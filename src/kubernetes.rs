use crate::models::KubernetesInfo;
use std::path::Path;
use tracing::debug;

/// Detects local listeners created by `kubectl port-forward`.
pub fn detect_port_forward(
    listener_port: u16,
    process_name: &str,
    command: &str,
) -> Option<KubernetesInfo> {
    let tokens = tokenize_command(command);
    if !looks_like_kubectl(process_name, &tokens) {
        return None;
    }

    let port_forward_idx = tokens.iter().position(|token| token == "port-forward")?;
    let namespace = find_flag_value(&tokens, &["-n", "--namespace"]);
    let context = find_flag_value(&tokens, &["--context"]);
    let bind_address = find_flag_value(&tokens, &["--address"]);

    let (target_idx, resource_kind, resource_name) =
        find_resource_target(&tokens, port_forward_idx + 1)?;
    let (local_port, remote_port) = select_port_spec(listener_port, &tokens, target_idx + 1)?;

    debug!(
        "Detected kubectl port-forward: {}/{} {}->{}",
        resource_kind,
        resource_name,
        local_port,
        remote_port
            .map(|port| port.to_string())
            .unwrap_or_else(|| "?".to_string())
    );

    Some(KubernetesInfo {
        tool: "kubectl".to_string(),
        resource_kind,
        resource_name,
        namespace,
        context,
        local_port,
        remote_port,
        bind_address,
    })
}

fn tokenize_command(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .map(|token| token.trim_matches(['\'', '"']).to_string())
        .collect()
}

fn looks_like_kubectl(process_name: &str, tokens: &[String]) -> bool {
    is_kubectl_executable(process_name)
        || tokens
            .iter()
            .any(|token| is_kubectl_executable(token.as_str()))
}

fn is_kubectl_executable(value: &str) -> bool {
    let name = Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .to_ascii_lowercase();

    name == "kubectl" || name == "kubectl.exe"
}

fn find_resource_target(tokens: &[String], mut index: usize) -> Option<(usize, String, String)> {
    while index < tokens.len() {
        let token = &tokens[index];

        if flag_consumes_next_value(token) {
            index += 2;
            continue;
        }

        if token.starts_with('-') {
            index += 1;
            continue;
        }

        if let Some((kind, name)) = parse_resource_target(token) {
            return Some((index, kind, name));
        }

        index += 1;
    }

    None
}

fn parse_resource_target(token: &str) -> Option<(String, String)> {
    let (kind, name) = token.split_once('/')?;
    let normalized_kind = normalize_resource_kind(kind)?;
    if name.trim().is_empty() {
        return None;
    }
    Some((normalized_kind.to_string(), name.to_string()))
}

fn normalize_resource_kind(kind: &str) -> Option<&'static str> {
    match kind.to_ascii_lowercase().as_str() {
        "po" | "pod" | "pods" => Some("pod"),
        "svc" | "service" | "services" => Some("service"),
        "deploy" | "deployment" | "deployments" => Some("deployment"),
        _ => None,
    }
}

fn find_flag_value(tokens: &[String], names: &[&str]) -> Option<String> {
    for (idx, token) in tokens.iter().enumerate() {
        for name in names {
            if token == name {
                return tokens
                    .get(idx + 1)
                    .filter(|value| !value.starts_with('-'))
                    .cloned();
            }

            if let Some(value) = token.strip_prefix(&format!("{name}=")) {
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

fn flag_consumes_next_value(token: &str) -> bool {
    matches!(
        token,
        "-n" | "--namespace" | "--context" | "--address" | "--kubeconfig"
    )
}

fn select_port_spec(
    listener_port: u16,
    tokens: &[String],
    start: usize,
) -> Option<(u16, Option<u16>)> {
    let mut wildcard_remote = None;
    let mut index = start;

    while index < tokens.len() {
        let token = &tokens[index];

        if flag_consumes_next_value(token) {
            index += 2;
            continue;
        }

        if token.starts_with('-') {
            index += 1;
            continue;
        }

        if let Some(spec) = parse_port_spec(token) {
            match spec.local_port {
                Some(local_port) if local_port == listener_port => {
                    return Some((local_port, spec.remote_port));
                }
                Some(_) => {}
                None => wildcard_remote = wildcard_remote.or(spec.remote_port),
            }
        }

        index += 1;
    }

    wildcard_remote.map(|remote_port| (listener_port, Some(remote_port)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PortSpec {
    local_port: Option<u16>,
    remote_port: Option<u16>,
}

fn parse_port_spec(token: &str) -> Option<PortSpec> {
    if token.is_empty() || token.starts_with('-') {
        return None;
    }

    if let Some((local, remote)) = token.split_once(':') {
        if remote.contains(':') {
            return None;
        }

        let local_port = if local.is_empty() {
            None
        } else {
            Some(parse_port(local)?)
        };
        let remote_port = Some(parse_port(remote)?);
        return Some(PortSpec {
            local_port,
            remote_port,
        });
    }

    let port = parse_port(token)?;
    Some(PortSpec {
        local_port: Some(port),
        remote_port: Some(port),
    })
}

fn parse_port(value: &str) -> Option<u16> {
    let port: u16 = value.parse().ok()?;
    if port == 0 { None } else { Some(port) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pod_port_forward() {
        let info =
            detect_port_forward(8080, "kubectl", "kubectl port-forward pod/api 8080:80").unwrap();

        assert_eq!(info.resource_kind, "pod");
        assert_eq!(info.resource_name, "api");
        assert_eq!(info.local_port, 8080);
        assert_eq!(info.remote_port, Some(80));
    }

    #[test]
    fn detects_windows_kubectl_executable() {
        let info = detect_port_forward(
            8080,
            "kubectl.exe",
            "kubectl.exe port-forward pod/api 8080:80",
        )
        .unwrap();

        assert_eq!(info.resource_kind, "pod");
    }

    #[test]
    fn normalizes_pods_resource_kind() {
        let info =
            detect_port_forward(8080, "kubectl", "kubectl port-forward pods/api 8080:80").unwrap();

        assert_eq!(info.resource_kind, "pod");
    }

    #[test]
    fn detects_service_port_forward() {
        let info =
            detect_port_forward(3000, "kubectl", "kubectl port-forward svc/api 3000:3000").unwrap();

        assert_eq!(info.resource_kind, "service");
        assert_eq!(info.resource_name, "api");
        assert_eq!(info.local_port, 3000);
        assert_eq!(info.remote_port, Some(3000));
    }

    #[test]
    fn detects_bare_service_port_spec() {
        let info =
            detect_port_forward(3000, "kubectl", "kubectl port-forward service/api 3000").unwrap();

        assert_eq!(info.resource_kind, "service");
        assert_eq!(info.remote_port, Some(3000));
    }

    #[test]
    fn detects_wildcard_local_port_spec() {
        let info = detect_port_forward(
            18080,
            "kubectl",
            "kubectl port-forward deployment/web :8080",
        )
        .unwrap();

        assert_eq!(info.resource_kind, "deployment");
        assert_eq!(info.local_port, 18080);
        assert_eq!(info.remote_port, Some(8080));
    }

    #[test]
    fn parses_namespace_short_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 -n dev",
        )
        .unwrap();

        assert_eq!(info.namespace.as_deref(), Some("dev"));
    }

    #[test]
    fn parses_namespace_long_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 --namespace dev",
        )
        .unwrap();

        assert_eq!(info.namespace.as_deref(), Some("dev"));
    }

    #[test]
    fn parses_namespace_equals_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 --namespace=dev",
        )
        .unwrap();

        assert_eq!(info.namespace.as_deref(), Some("dev"));
    }

    #[test]
    fn parses_context_split_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 --context staging",
        )
        .unwrap();

        assert_eq!(info.context.as_deref(), Some("staging"));
    }

    #[test]
    fn parses_context_equals_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 --context=staging",
        )
        .unwrap();

        assert_eq!(info.context.as_deref(), Some("staging"));
    }

    #[test]
    fn parses_address_split_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 --address 127.0.0.1",
        )
        .unwrap();

        assert_eq!(info.bind_address.as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn parses_address_equals_flag() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward pod/api 8080:80 --address=0.0.0.0",
        )
        .unwrap();

        assert_eq!(info.bind_address.as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn handles_flags_before_port_forward() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl --context staging -n dev port-forward svc/api 8080:80",
        )
        .unwrap();

        assert_eq!(info.namespace.as_deref(), Some("dev"));
        assert_eq!(info.context.as_deref(), Some("staging"));
    }

    #[test]
    fn handles_flags_after_resource_target() {
        let info = detect_port_forward(
            8080,
            "kubectl",
            "kubectl port-forward svc/api --namespace dev 8080:80",
        )
        .unwrap();

        assert_eq!(info.namespace.as_deref(), Some("dev"));
        assert_eq!(info.remote_port, Some(80));
    }

    #[test]
    fn selects_matching_port_from_multiple_specs() {
        let info = detect_port_forward(
            9090,
            "kubectl",
            "kubectl port-forward svc/api 8080:80 9090:90",
        )
        .unwrap();

        assert_eq!(info.local_port, 9090);
        assert_eq!(info.remote_port, Some(90));
    }

    #[test]
    fn returns_none_when_no_port_matches_listener() {
        let info = detect_port_forward(
            7070,
            "kubectl",
            "kubectl port-forward svc/api 8080:80 9090:90",
        );

        assert!(info.is_none());
    }

    #[test]
    fn returns_none_for_non_kubectl_commands() {
        let info = detect_port_forward(8080, "node", "node server.js");

        assert!(info.is_none());
    }

    #[test]
    fn returns_none_without_port_forward_subcommand() {
        let info = detect_port_forward(8080, "kubectl", "kubectl get pods");

        assert!(info.is_none());
    }

    #[test]
    fn returns_none_for_malformed_resource_targets() {
        let info = detect_port_forward(8080, "kubectl", "kubectl port-forward pod/ 8080:80");

        assert!(info.is_none());
    }

    #[test]
    fn handles_long_irrelevant_commands() {
        let command = format!(
            "kubectl {} port-forward svc/api 8080:80",
            "--v=0 ".repeat(256)
        );
        let info = detect_port_forward(8080, "kubectl", &command).unwrap();

        assert_eq!(info.resource_name, "api");
    }
}
