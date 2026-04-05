use crate::models::GitInfo;
use std::path::Path;
use std::process::Command;
use tracing::debug;

/// Get git branch and dirty status for a directory.
pub fn get_git_info(dir: &Path) -> Option<GitInfo> {
    // Check if in git repo and get branch name
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;

    if !branch_output.status.success() {
        return None;
    }

    let branch = String::from_utf8(branch_output.stdout)
        .unwrap_or_default()
        .trim()
        .to_string();

    let branch = if branch == "HEAD" {
        // Detached HEAD, get short hash
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(dir)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "HEAD".to_string())
    } else {
        branch
    };

    // Check if dirty
    let dirty_output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .current_dir(dir)
        .output()
        .ok()?;

    let dirty = !dirty_output.stdout.is_empty();

    debug!(
        "Git info for {}: branch={}, dirty={}",
        dir.display(),
        branch,
        dirty
    );

    Some(GitInfo { branch, dirty })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_info_for_portforge() {
        // The portforge repo itself should have git info
        let info = get_git_info(Path::new(env!("CARGO_MANIFEST_DIR")));
        assert!(info.is_some());
        let git = info.unwrap();
        assert!(!git.branch.is_empty());
    }

    #[test]
    fn test_git_info_nonexistent_dir() {
        let info = get_git_info(Path::new("/tmp/definitely-not-a-git-repo-xyz123"));
        assert!(info.is_none());
    }
}
