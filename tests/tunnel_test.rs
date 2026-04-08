use portforge::tunnel::detect_tunnel;

#[test]
fn test_detect_ngrok_basic() {
    let tunnel = detect_tunnel("ngrok", "ngrok http 3000");
    assert!(tunnel.is_some());
    assert_eq!(tunnel.unwrap().kind, "ngrok");
}

#[test]
fn test_detect_cloudflared_basic() {
    let tunnel = detect_tunnel("cloudflared", "cloudflared tunnel run");
    assert!(tunnel.is_some());
    assert_eq!(tunnel.unwrap().kind, "cloudflared");
}

#[test]
fn test_detect_no_tunnel() {
    let tunnel = detect_tunnel("node", "node server.js");
    assert!(tunnel.is_none());

    let tunnel2 = detect_tunnel("python", "python app.py");
    assert!(tunnel2.is_none());
}

#[test]
fn test_ssh_reverse_tunnel_detection() {
    let tunnel = detect_tunnel("ssh", "ssh -R 8080:localhost:3000 user@server.com");
    assert!(tunnel.is_some());
    assert_eq!(tunnel.unwrap().kind, "ssh");
}

#[test]
fn test_tailscale_funnel_detection() {
    let tunnel = detect_tunnel("tailscale", "tailscale funnel 3000");
    assert!(tunnel.is_some());
    let info = tunnel.unwrap();
    assert_eq!(info.kind, "tailscale");
    assert_eq!(info.public_url.as_deref(), Some("Tailscale Funnel"));
}
