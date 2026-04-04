use crate::models::GitInfo;
use std::path::Path;
use tracing::debug;

/// Get git branch and dirty status for a directory.
pub fn get_git_info(dir: &Path) -> Option<GitInfo> {
    let repo = git2::Repository::discover(dir).ok()?;

    let branch = get_branch_name(&repo).unwrap_or_else(|| "HEAD".to_string());
    let dirty = is_dirty(&repo);

    debug!("Git info for {}: branch={}, dirty={}", dir.display(), branch, dirty);

    Some(GitInfo { branch, dirty })
}

/// Get the current branch name.
fn get_branch_name(repo: &git2::Repository) -> Option<String> {
    let head = repo.head().ok()?;

    if head.is_branch() {
        head.shorthand().map(|s| s.to_string())
    } else {
        // Detached HEAD — return short OID
        head.target()
            .map(|oid| format!("{:.7}", oid))
    }
}

/// Check if the working tree has uncommitted changes.
fn is_dirty(repo: &git2::Repository) -> bool {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(false)
        .include_ignored(false);

    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => !statuses.is_empty(),
        Err(_) => false,
    }
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
