use crate::models::UpdateStatus;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

#[tauri::command]
pub fn check_for_updates() -> UpdateStatus {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    match fetch_latest_release() {
        Ok(release) => {
            let latest = release.tag_name.trim_start_matches('v').to_string();
            let is_update_available = is_version_newer(&latest, &current_version);
            UpdateStatus {
                current_version,
                latest_version: Some(latest),
                release_url: Some(release.html_url),
                is_update_available,
                error: None,
            }
        }
        Err(error) => UpdateStatus {
            current_version,
            latest_version: None,
            release_url: None,
            is_update_available: false,
            error: Some(error),
        },
    }
}

fn fetch_latest_release() -> Result<GithubRelease, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Textream Windows")
        .build()
        .map_err(|err| err.to_string())?;
    client
        .get("https://api.github.com/repos/f/textream/releases/latest")
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?
        .json::<GithubRelease>()
        .map_err(|err| err.to_string())
}

fn is_version_newer(remote: &str, local: &str) -> bool {
    let parse = |version: &str| {
        version
            .split('.')
            .map(|part| part.parse::<u32>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let remote = parse(remote);
    let local = parse(local);
    for index in 0..remote.len().max(local.len()) {
        let remote_part = *remote.get(index).unwrap_or(&0);
        let local_part = *local.get(index).unwrap_or(&0);
        if remote_part != local_part {
            return remote_part > local_part;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_semver_parts() {
        assert!(is_version_newer("1.2.0", "1.1.9"));
        assert!(is_version_newer("1.2.1", "1.2.0"));
        assert!(!is_version_newer("1.2.0", "1.2.0"));
        assert!(!is_version_newer("1.1.9", "1.2.0"));
    }
}

