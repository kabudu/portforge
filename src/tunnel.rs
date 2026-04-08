use crate::models::TunnelInfo;
use tracing::debug;

/// Known tunnel processes to detect.
const TUNNEL_PROCESSES: &[(&str, &str)] = &[
    ("ngrok", "ngrok"),
    ("cloudflared", "cloudflared"),
    ("localtunnel", "lt"),
    ("tailscale", "tailscale"),
    ("serveo", "serveo"),
    ("ssh", "ssh"), // Could be SSH reverse tunnel
];

/// Detect if a process is a tunnel service based on command line.
pub fn detect_tunnel(process_name: &str, command: &str) -> Option<TunnelInfo> {
    let cmd_lower = command.to_lowercase();
    let name_lower = process_name.to_lowercase();

    for (process_keyword, kind) in TUNNEL_PROCESSES {
        if name_lower.contains(process_keyword) || cmd_lower.contains(process_keyword) {
            // Try to extract public URL from command
            let public_url = extract_tunnel_url(&cmd_lower, kind);

            debug!("Detected tunnel: {} with URL {:?}", kind, public_url);

            return Some(TunnelInfo {
                kind: kind.to_string(),
                public_url,
            });
        }
    }

    None
}

/// Extract public URL from tunnel command line arguments.
fn extract_tunnel_url(command: &str, kind: &str) -> Option<String> {
    match kind {
        "ngrok" => {
            // ngrok http <port> or ngrok http --url <url>
            if let Some(idx) = command.find("--url") {
                let after = &command[idx + 5..];
                let url = after.trim_start_matches([' ', '=']);
                // Get until next space or end
                url.split_whitespace().next().map(String::from)
            } else if command.contains("http ") {
                // Just showing it tunnels http, no specific URL without API call
                None
            } else {
                None
            }
        }
        "cloudflared" => {
            // cloudflared tunnel --url <url>
            if let Some(idx) = command.find("--url") {
                let after = &command[idx + 5..];
                let url = after.trim_start_matches([' ', '=']);
                url.split_whitespace().next().map(String::from)
            } else {
                None
            }
        }
        "lt" | "localtunnel" => {
            // lt --port <port> --subdomain <name>
            if let Some(idx) = command.find("--subdomain") {
                let after = &command[idx + 11..];
                let subdomain = after.trim_start_matches([' ', '=']);
                subdomain
                    .split_whitespace()
                    .next()
                    .map(|sub| format!("https://{}.loca.lt", sub))
            } else {
                None
            }
        }
        "tailscale" => {
            if command.contains("funnel") || command.contains("serve") {
                command
                    .split_whitespace()
                    .find(|part| part.ends_with(".ts.net") || part.ends_with(".tailscale.net"))
                    .map(String::from)
                    .or_else(|| Some("Tailscale Funnel".to_string()))
            } else {
                None
            }
        }
        "serveo" => {
            // ssh -R 80:localhost:3000 serveo.net
            if command.contains("serveo.net") {
                Some("serveo.net".to_string())
            } else {
                None
            }
        }
        "ssh" => {
            // SSH reverse tunnel: ssh -R <remote>:<host>:<port> <user>@<server>
            if command.contains("-R") && command.contains("localhost") {
                Some("SSH reverse tunnel".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ngrok() {
        let tunnel = detect_tunnel("ngrok", "ngrok http 3000 --url myapp.ngrok.io");
        assert!(tunnel.is_some());
        let info = tunnel.unwrap();
        assert_eq!(info.kind, "ngrok");
        assert_eq!(info.public_url, Some("myapp.ngrok.io".to_string()));
    }

    #[test]
    fn test_detect_cloudflared() {
        let tunnel = detect_tunnel(
            "cloudflared",
            "cloudflared tunnel --url http://localhost:3000",
        );
        assert!(tunnel.is_some());
        let info = tunnel.unwrap();
        assert_eq!(info.kind, "cloudflared");
    }

    #[test]
    fn test_detect_non_tunnel() {
        let tunnel = detect_tunnel("node", "node server.js");
        assert!(tunnel.is_none());
    }

    #[test]
    fn test_detect_ssh_tunnel() {
        let tunnel = detect_tunnel("ssh", "ssh -R 8080:localhost:3000 user@remote.server.com");
        assert!(tunnel.is_some());
        let info = tunnel.unwrap();
        assert_eq!(info.kind, "ssh");
    }

    #[test]
    fn test_detect_localtunnel() {
        let tunnel = detect_tunnel("localtunnel", "lt --port 3000 --subdomain mytest");
        assert!(tunnel.is_some());
        let info = tunnel.unwrap();
        assert_eq!(info.kind, "lt");
        assert_eq!(info.public_url, Some("https://mytest.loca.lt".to_string()));
    }

    #[test]
    fn test_detect_tunnel_with_command_case_insensitive() {
        let tunnel = detect_tunnel("NGROK", "NGROK HTTP 3000");
        assert!(tunnel.is_some());
        assert_eq!(tunnel.unwrap().kind, "ngrok");
    }

    #[test]
    fn test_extract_url_from_ngrok_custom_url() {
        let tunnel = detect_tunnel("ngrok", "ngrok http --url https://abc123.ngrok.io 3000");
        assert!(tunnel.is_some());
        assert_eq!(
            tunnel.unwrap().public_url,
            Some("https://abc123.ngrok.io".to_string())
        );
    }

    #[test]
    fn test_detect_tailscale_funnel() {
        let tunnel = detect_tunnel("tailscale", "tailscale funnel 3000");
        assert!(tunnel.is_some());
        let info = tunnel.unwrap();
        assert_eq!(info.kind, "tailscale");
        assert_eq!(info.public_url, Some("Tailscale Funnel".to_string()));
    }
}
