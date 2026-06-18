use crate::models::{AppSettings, ReadingState};
use crate::reading::browser_state;
use crate::AppState;
use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::State;
use tiny_http::{Header, Response, Server};
use tungstenite::Message;

#[derive(Default)]
pub struct RemoteServer {
    running: Option<Arc<AtomicBool>>,
    handles: Vec<JoinHandle<()>>,
}

impl RemoteServer {
    fn stop(&mut self) {
        if let Some(running) = &self.running {
            running.store(false, Ordering::SeqCst);
        }
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
        self.running = None;
    }
}

impl Drop for RemoteServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tauri::command]
pub fn start_remote_server(app_state: State<'_, AppState>) -> Result<u16, String> {
    let settings = app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())?
        .clone();
    let http_port = settings.browser_server_port;
    let ws_port = http_port + 1;

    let http = Server::http(("0.0.0.0", http_port)).map_err(|err| err.to_string())?;
    let ws = TcpListener::bind(("0.0.0.0", ws_port)).map_err(|err| err.to_string())?;
    ws.set_nonblocking(true).map_err(|err| err.to_string())?;

    let running = Arc::new(AtomicBool::new(true));
    let mut server = app_state
        .remote
        .lock()
        .map_err(|_| "Remote server lock failed".to_string())?;
    server.stop();

    let http_running = running.clone();
    let http_handle = thread::spawn(move || run_http(http, http_running, ws_port));

    let ws_running = running.clone();
    let reading = app_state.reading.clone();
    let settings_state = app_state.settings.clone();
    let ws_handle = thread::spawn(move || run_ws(ws, ws_running, reading, settings_state));

    server.running = Some(running);
    server.handles = vec![http_handle, ws_handle];
    Ok(http_port)
}

#[tauri::command]
pub fn stop_remote_server(app_state: State<'_, AppState>) -> Result<(), String> {
    app_state
        .remote
        .lock()
        .map_err(|_| "Remote server lock failed".to_string())?
        .stop();
    Ok(())
}

fn run_http(server: Server, running: Arc<AtomicBool>, ws_port: u16) {
    while running.load(Ordering::SeqCst) {
        match server.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(request)) => {
                let body = generate_remote_html(ws_port);
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
) {
    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let connection_running = running.clone();
                let reading = reading.clone();
                let settings = settings.clone();
                thread::spawn(move || {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(20)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
                    let Ok(mut socket) = tungstenite::accept(stream) else {
                        return;
                    };
                    while connection_running.load(Ordering::SeqCst) {
                        let state = {
                            let Ok(reading) = reading.lock() else { break };
                            let Ok(settings) = settings.lock() else { break };
                            browser_state(&reading, &settings)
                        };
                        if let Ok(data) = serde_json::to_string(&state) {
                            if socket.send(Message::Text(data.into())).is_err() {
                                break;
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

fn generate_remote_html(ws_port: u16) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Textream Remote</title>
<style>
body{{margin:0;background:#050506;color:white;font:600 22px/1.7 system-ui,-apple-system,Segoe UI,sans-serif}}
#app{{min-height:100vh;padding:28px;display:flex;align-items:center;justify-content:center}}
.word{{opacity:.25;margin-right:.35em}}.word.read{{opacity:1;color:var(--font-color,#fff)}}.cue{{color:var(--cue-color,#fff)}}
.empty{{opacity:.45;font-size:15px}}
</style>
</head>
<body><main id="app"><div class="empty">Connecting...</div></main>
<script>
const app=document.getElementById('app');
const ws=new WebSocket('ws://'+location.hostname+':{ws_port}');
ws.onmessage=(event)=>{{
  const s=JSON.parse(event.data);
  if(!s.isActive){{app.innerHTML='<div class="empty">Waiting for Textream...</div>';return;}}
  document.documentElement.style.setProperty('--font-color',s.fontColor);
  document.documentElement.style.setProperty('--cue-color',s.cueColor);
  let offset=0;
  app.innerHTML='<div>'+s.words.map(w=>{{
    const read=offset<s.highlightedCharCount;
    const cue=/^\\[.*\\]$/.test(w);
    const html='<span class="word '+(read?'read ':'')+(cue?'cue':'')+'">'+w+'</span>';
    offset+=w.length+1;
    return html;
  }}).join('')+'</div>';
}};
ws.onclose=()=>app.innerHTML='<div class="empty">Disconnected</div>';
</script></body></html>"#
    )
}

