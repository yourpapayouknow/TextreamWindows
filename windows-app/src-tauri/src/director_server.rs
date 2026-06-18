use crate::models::{AppSettings, DirectorCommand, ReadingState};
use crate::reading::{director_state, rebuild_reading_content};
use crate::AppState;
use rand::{distributions::Alphanumeric, Rng};
use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::State;
use tiny_http::{Header, Response, Server};
use tungstenite::{Error as WsError, Message};

#[derive(Default)]
pub struct DirectorServer {
    running: Option<Arc<AtomicBool>>,
    handles: Vec<JoinHandle<()>>,
    token: Option<String>,
}

impl DirectorServer {
    fn stop(&mut self) {
        if let Some(running) = &self.running {
            running.store(false, Ordering::SeqCst);
        }
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
        self.running = None;
        self.token = None;
    }
}

impl Drop for DirectorServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tauri::command]
pub fn start_director_server(app_state: State<'_, AppState>) -> Result<(u16, String), String> {
    let settings = app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())?
        .clone();
    let http_port = settings.director_server_port;
    let ws_port = http_port + 1;
    let token = generate_token();

    let http = Server::http(("0.0.0.0", http_port)).map_err(|err| err.to_string())?;
    let ws = TcpListener::bind(("0.0.0.0", ws_port)).map_err(|err| err.to_string())?;
    ws.set_nonblocking(true).map_err(|err| err.to_string())?;

    let running = Arc::new(AtomicBool::new(true));
    let mut server = app_state
        .director
        .lock()
        .map_err(|_| "Director server lock failed".to_string())?;
    server.stop();

    let http_running = running.clone();
    let http_token = token.clone();
    let http_handle = thread::spawn(move || run_http(http, http_running, ws_port, http_token));

    let ws_running = running.clone();
    let ws_token = token.clone();
    let reading = app_state.reading.clone();
    let settings_state = app_state.settings.clone();
    let ws_handle = thread::spawn(move || run_ws(ws, ws_running, reading, settings_state, ws_token));

    server.running = Some(running);
    server.handles = vec![http_handle, ws_handle];
    server.token = Some(token.clone());
    Ok((http_port, token))
}

#[tauri::command]
pub fn stop_director_server(app_state: State<'_, AppState>) -> Result<(), String> {
    app_state
        .director
        .lock()
        .map_err(|_| "Director server lock failed".to_string())?
        .stop();
    Ok(())
}

fn run_http(server: Server, running: Arc<AtomicBool>, ws_port: u16, token: String) {
    while running.load(Ordering::SeqCst) {
        match server.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(request)) => {
                let body = generate_director_html(ws_port, &token);
                let mut response = Response::from_string(body);
                if let Ok(header) = Header::from_bytes("Content-Type", "text/html; charset=utf-8") {
                    response.add_header(header);
                }
                if let Ok(header) = Header::from_bytes("Cache-Control", "no-store") {
                    response.add_header(header);
                }
                let _ = request.respond(response);
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }
}

fn run_ws(
    listener: TcpListener,
    running: Arc<AtomicBool>,
    reading: Arc<Mutex<ReadingState>>,
    settings: Arc<Mutex<AppSettings>>,
    token: String,
) {
    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let connection_running = running.clone();
                let reading = reading.clone();
                let settings = settings.clone();
                let token = token.clone();
                thread::spawn(move || {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(20)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
                    let Ok(mut socket) = tungstenite::accept(stream) else {
                        return;
                    };
                    let mut authenticated = false;
                    while connection_running.load(Ordering::SeqCst) {
                        match socket.read() {
                            Ok(message) => {
                                if let Ok(text) = message.to_text() {
                                    authenticated = handle_command(text, authenticated, &token, &reading);
                                }
                            }
                            Err(WsError::Io(err))
                                if err.kind() == std::io::ErrorKind::WouldBlock
                                    || err.kind() == std::io::ErrorKind::TimedOut => {}
                            Err(_) => break,
                        }

                        if authenticated {
                            let state = {
                                let Ok(reading) = reading.lock() else { break };
                                let Ok(settings) = settings.lock() else { break };
                                director_state(&reading, &settings)
                            };
                            if let Ok(data) = serde_json::to_string(&state) {
                                if socket.send(Message::Text(data.into())).is_err() {
                                    break;
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }
}

fn handle_command(
    text: &str,
    authenticated: bool,
    token: &str,
    reading: &Arc<Mutex<ReadingState>>,
) -> bool {
    let Ok(command) = serde_json::from_str::<DirectorCommand>(text) else {
        return authenticated;
    };

    if !authenticated {
        return command.r#type == "auth" && command.text.as_deref() == Some(token);
    }

    let Ok(mut state) = reading.lock() else {
        return authenticated;
    };

    match command.r#type.as_str() {
        "setText" => {
            if let Some(text) = command.text {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    state.pages = vec![trimmed.to_string()];
                    state.current_page_index = 0;
                    state.read_pages = vec![0];
                    state.is_running = true;
                    state.recognized_char_count = 0;
                    state.last_spoken_text.clear();
                    rebuild_reading_content(&mut state);
                }
            }
        }
        "updateText" => {
            if let Some(text) = command.text {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    state.pages = vec![trimmed.to_string()];
                    state.current_page_index = 0;
                    state.is_running = true;
                    state.recognized_char_count = command
                        .read_char_count
                        .unwrap_or(state.recognized_char_count)
                        .min(trimmed.chars().count());
                    rebuild_reading_content(&mut state);
                }
            }
        }
        "stop" => {
            state.is_running = false;
        }
        _ => {}
    }

    authenticated
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn generate_director_html(ws_port: u16, token: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Textream Director</title>
<style>
body{{margin:0;background:#09090b;color:white;font:500 16px/1.5 system-ui,-apple-system,Segoe UI,sans-serif;display:flex;flex-direction:column;height:100vh}}
header,footer{{padding:12px 16px;border-color:rgba(255,255,255,.1);background:#111114}}header{{border-bottom:1px solid rgba(255,255,255,.1)}}footer{{border-top:1px solid rgba(255,255,255,.1);display:flex;gap:10px;align-items:center}}
#editor{{flex:1;padding:18px;outline:none;overflow:auto;white-space:pre-wrap;font-size:18px;line-height:1.7}}button{{border:0;border-radius:8px;padding:10px 18px;font-weight:700;color:white;background:#2563eb}}button.stop{{background:#dc2626}}#status{{opacity:.55}}
</style></head><body>
<header><strong>Textream Director</strong> <span id="status">Connecting...</span></header>
<div id="editor" contenteditable="true" data-placeholder="Type or paste script here"></div>
<footer><button id="go">Go</button><button class="stop" id="stop">Stop</button><span id="progress"></span></footer>
<script>
const token='{token}',wsPort={ws_port},editor=document.getElementById('editor'),status=document.getElementById('status'),progress=document.getElementById('progress');
let ws,active=false,readCount=0;
function send(o){{if(ws&&ws.readyState===1)ws.send(JSON.stringify(o));}}
function connect(){{ws=new WebSocket('ws://'+location.hostname+':'+wsPort);ws.onopen=()=>{{status.textContent='Connected';send({{type:'auth',text:token}})}};ws.onclose=()=>{{status.textContent='Reconnecting...';setTimeout(connect,1200)}};ws.onmessage=e=>{{const s=JSON.parse(e.data);active=s.isActive;readCount=s.highlightedCharCount||0;progress.textContent=s.totalCharCount?Math.round(readCount/s.totalCharCount*100)+'%':'';}}}}
document.getElementById('go').onclick=()=>send({{type:'setText',text:editor.innerText}});
document.getElementById('stop').onclick=()=>send({{type:'stop'}});
editor.addEventListener('input',()=>{{if(active)send({{type:'updateText',text:editor.innerText,readCharCount:readCount}})}});
connect();
</script></body></html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_command_must_match_token() {
        let reading = Arc::new(Mutex::new(ReadingState::default()));
        assert!(!handle_command(
            r#"{"type":"auth","text":"bad"}"#,
            false,
            "token",
            &reading
        ));
        assert!(handle_command(
            r#"{"type":"auth","text":"token"}"#,
            false,
            "token",
            &reading
        ));
    }

    #[test]
    fn set_text_command_starts_reading() {
        let reading = Arc::new(Mutex::new(ReadingState::default()));
        assert!(handle_command(
            r#"{"type":"auth","text":"token"}"#,
            false,
            "token",
            &reading
        ));
        handle_command(
            r#"{"type":"setText","text":"Hello world"}"#,
            true,
            "token",
            &reading,
        );
        let state = reading.lock().unwrap();
        assert!(state.is_running);
        assert_eq!(state.words, vec!["Hello", "world"]);
    }
}

