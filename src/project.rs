use crate::models::ProjectInfo;
use std::path::Path;
use tracing::debug;

/// Detect project kind and framework from the given directory.
/// Walks up the directory tree to find project manifest files.
pub fn detect_project(dir: &Path) -> Option<ProjectInfo> {
    let mut current = dir.to_path_buf();

    for _ in 0..10 {
        // Limit traversal depth
        if let Some(info) = detect_in_dir(&current) {
            return Some(info);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Check a single directory for project manifest files.
fn detect_in_dir(dir: &Path) -> Option<ProjectInfo> {
    // Check in priority order (most specific first)
    let detectors: Vec<(&str, &str, &[&str])> = vec![
        // Rust
        (
            "Cargo.toml",
            "Rust",
            &["actix", "axum", "rocket", "warp", "tide"],
        ),
        // Node.js / JavaScript / TypeScript
        (
            "package.json",
            "Node.js",
            &[
                "next", "nuxt", "remix", "express", "fastify", "nest", "vite", "react", "vue",
                "angular", "svelte", "astro",
            ],
        ),
        // Python
        (
            "pyproject.toml",
            "Python",
            &[
                "django",
                "flask",
                "fastapi",
                "starlette",
                "tornado",
                "sanic",
            ],
        ),
        (
            "requirements.txt",
            "Python",
            &["django", "flask", "fastapi", "starlette"],
        ),
        ("Pipfile", "Python", &["django", "flask", "fastapi"]),
        // Go
        ("go.mod", "Go", &["gin", "echo", "fiber", "chi", "mux"]),
        // Ruby
        ("Gemfile", "Ruby", &["rails", "sinatra", "hanami"]),
        // Java / Kotlin
        ("pom.xml", "Java", &["spring", "quarkus", "micronaut"]),
        (
            "build.gradle",
            "Java/Kotlin",
            &["spring", "ktor", "quarkus"],
        ),
        ("build.gradle.kts", "Kotlin", &["spring", "ktor"]),
        // PHP
        ("composer.json", "PHP", &["laravel", "symfony", "slim"]),
        // Elixir
        ("mix.exs", "Elixir", &["phoenix"]),
        // Swift
        ("Package.swift", "Swift", &["vapor"]),
        // Dart/Flutter
        ("pubspec.yaml", "Dart", &["flutter"]),
        // Zig
        ("build.zig", "Zig", &[]),
        // Docker Compose
        ("docker-compose.yml", "Docker Compose", &[]),
        ("docker-compose.yaml", "Docker Compose", &[]),
        ("compose.yml", "Docker Compose", &[]),
        ("compose.yaml", "Docker Compose", &[]),
    ];

    for (manifest, kind, frameworks) in &detectors {
        let manifest_path = if manifest.starts_with('*') {
            // Glob pattern — check if any matching file exists
            match std::fs::read_dir(dir) { Ok(entries) => {
                let ext = manifest.trim_start_matches('*');
                let found = entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name().to_string_lossy().ends_with(ext));
                found.map(|e| e.path())
            } _ => {
                None
            }}
        } else {
            let path = dir.join(manifest);
            if path.exists() {
                Some(path)
            } else {
                None
            }
        };

        if let Some(found_path) = manifest_path {
            let framework = detect_framework(&found_path, kind, frameworks);
            let framework_str = framework.unwrap_or_default();
            debug!(
                "Detected {} project ({}) at {}",
                kind,
                framework_str,
                dir.display()
            );

            return Some(ProjectInfo {
                kind: kind.to_string(),
                framework: framework_str,
                version: detect_version(&found_path, kind),
                detected_file: found_path,
            });
        }
    }

    None
}

/// Try to detect the specific framework from the manifest file contents.
fn detect_framework(manifest: &Path, kind: &str, frameworks: &[&str]) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;
    let content_lower = content.to_lowercase();

    match kind {
        "Node.js" => {
            // Check package.json dependencies
            for fw in frameworks {
                if content_lower.contains(&format!("\"{}\"", fw))
                    || content_lower.contains(&format!("\"@{}/", fw))
                {
                    return Some(capitalize_framework(fw));
                }
            }
            None
        }
        "Rust" => {
            for fw in frameworks {
                if content_lower.contains(&format!("{} =", fw))
                    || content_lower.contains(&format!("\"{}\"", fw))
                {
                    return Some(capitalize_framework(fw));
                }
            }
            None
        }
        "Python" => {
            for fw in frameworks {
                if content_lower.contains(fw) {
                    return Some(capitalize_framework(fw));
                }
            }
            None
        }
        "Go" => {
            for fw in frameworks {
                if content_lower.contains(fw) {
                    return Some(capitalize_framework(fw));
                }
            }
            None
        }
        "Ruby" => {
            for fw in frameworks {
                if content_lower.contains(fw) {
                    return Some(capitalize_framework(fw));
                }
            }
            None
        }
        _ => {
            // Generic check
            for fw in frameworks {
                if content_lower.contains(fw) {
                    return Some(capitalize_framework(fw));
                }
            }
            None
        }
    }
}

/// Extract version from manifest if possible.
fn detect_version(manifest: &Path, kind: &str) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;

    match kind {
        "Node.js" => {
            // Parse "version": "x.y.z" from package.json
            let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
            parsed
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }
        "Rust" => {
            // Simple regex-free version extraction from Cargo.toml
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("version") && trimmed.contains('=') {
                    let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        return Some(
                            parts[1]
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string(),
                        );
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Capitalize a framework name nicely.
fn capitalize_framework(name: &str) -> String {
    match name {
        "next" => "Next.js".to_string(),
        "nuxt" => "Nuxt.js".to_string(),
        "react" => "React".to_string(),
        "vue" => "Vue.js".to_string(),
        "angular" => "Angular".to_string(),
        "svelte" => "Svelte".to_string(),
        "express" => "Express".to_string(),
        "fastify" => "Fastify".to_string(),
        "nest" => "NestJS".to_string(),
        "remix" => "Remix".to_string(),
        "vite" => "Vite".to_string(),
        "astro" => "Astro".to_string(),
        "actix" => "Actix Web".to_string(),
        "axum" => "Axum".to_string(),
        "rocket" => "Rocket".to_string(),
        "warp" => "Warp".to_string(),
        "tide" => "Tide".to_string(),
        "django" => "Django".to_string(),
        "flask" => "Flask".to_string(),
        "fastapi" => "FastAPI".to_string(),
        "starlette" => "Starlette".to_string(),
        "tornado" => "Tornado".to_string(),
        "sanic" => "Sanic".to_string(),
        "gin" => "Gin".to_string(),
        "echo" => "Echo".to_string(),
        "fiber" => "Fiber".to_string(),
        "chi" => "Chi".to_string(),
        "mux" => "Gorilla Mux".to_string(),
        "rails" => "Rails".to_string(),
        "sinatra" => "Sinatra".to_string(),
        "spring" => "Spring Boot".to_string(),
        "quarkus" => "Quarkus".to_string(),
        "micronaut" => "Micronaut".to_string(),
        "ktor" => "Ktor".to_string(),
        "laravel" => "Laravel".to_string(),
        "symfony" => "Symfony".to_string(),
        "phoenix" => "Phoenix".to_string(),
        "vapor" => "Vapor".to_string(),
        "flutter" => "Flutter".to_string(),
        "aspnet" => "ASP.NET".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_framework() {
        assert_eq!(capitalize_framework("next"), "Next.js");
        assert_eq!(capitalize_framework("express"), "Express");
        assert_eq!(capitalize_framework("actix"), "Actix Web");
        assert_eq!(capitalize_framework("spring"), "Spring Boot");
        assert_eq!(capitalize_framework("unknown"), "Unknown");
    }

    #[test]
    fn test_detect_project_in_rust_project() {
        // The portforge project itself should be detected as Rust
        let project = detect_project(Path::new(env!("CARGO_MANIFEST_DIR")));
        assert!(project.is_some());
        let info = project.unwrap();
        assert_eq!(info.kind, "Rust");
    }
}
