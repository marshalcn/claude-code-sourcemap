use std::path::{Path, PathBuf};
use std::fs;

/// Checks if a directory contains a `.git` folder or file (like submodules/worktrees)
fn has_git_marker(path: &Path) -> bool {
    let git_path = path.join(".git");
    if let Ok(metadata) = fs::metadata(&git_path) {
        return metadata.is_dir() || metadata.is_file();
    }
    false
}

/// Find the git root by walking up the directory tree.
/// Looks for a .git directory or file.
/// Returns the directory containing .git, or None if not found.
pub fn find_git_root<P: AsRef<Path>>(start_path: P) -> Option<PathBuf> {
    let mut current = start_path.as_ref().to_path_buf();
    
    // Convert to absolute path if possible
    if let Ok(abs) = current.canonicalize() {
        current = abs;
    }

    loop {
        if has_git_marker(&current) {
            return Some(current);
        }

        let parent = current.parent().map(|p| p.to_path_buf());
        match parent {
            Some(p) if p != current => current = p,
            _ => break,
        }
    }

    None
}

/// Resolve a git root to the canonical main repository root.
/// For a regular repo this is a no-op. For a worktree, it follows the
/// `.git` file -> `gitdir:` -> `commondir` chain to find the main repo's
/// working directory.
pub fn resolve_canonical_root(git_root: &Path) -> PathBuf {
    let git_path = git_root.join(".git");
    
    // If it's a normal repository (.git is a directory), just return the root
    if let Ok(metadata) = fs::metadata(&git_path) {
        if metadata.is_dir() {
            return git_root.to_path_buf();
        }
    }
    
    // If .git is a file, it might be a worktree or submodule
    if let Ok(content) = fs::read_to_string(&git_path) {
        let content = content.trim();
        if content.starts_with("gitdir:") {
            let gitdir_relative = content["gitdir:".len()..].trim();
            let worktree_git_dir = git_root.join(gitdir_relative);
            
            // Try to read commondir to find the main repository
            let commondir_path = worktree_git_dir.join("commondir");
            if let Ok(commondir_content) = fs::read_to_string(&commondir_path) {
                let common_dir = worktree_git_dir.join(commondir_content.trim());
                
                // Security check: ensure it matches the worktree structure
                if let Some(parent) = common_dir.parent() {
                    return parent.to_path_buf();
                }
            }
        }
    }
    
    // Fallback
    git_root.to_path_buf()
}

/// Find the canonical git repository root, resolving through worktrees.
pub fn find_canonical_git_root<P: AsRef<Path>>(start_path: P) -> Option<PathBuf> {
    find_git_root(start_path).map(|root| resolve_canonical_root(&root))
}

/// Checks if the current working directory appears to be a bare git repository
/// or has been manipulated to look like one (sandbox escape attack vector).
pub fn is_current_directory_bare_git_repo<P: AsRef<Path>>(cwd: P) -> bool {
    let cwd = cwd.as_ref();
    let git_path = cwd.join(".git");
    
    if let Ok(stats) = fs::metadata(&git_path) {
        if stats.is_file() {
            return false;
        }
        if stats.is_dir() {
            let git_head_path = git_path.join("HEAD");
            if let Ok(head_stats) = fs::metadata(&git_head_path) {
                if head_stats.is_file() {
                    return false; // normal repo
                }
            }
        }
    }

    // No valid .git/HEAD found. Check if cwd has bare git repo indicators.
    let head_exists = fs::metadata(cwd.join("HEAD")).map(|s| s.is_file()).unwrap_or(false);
    let objects_exists = fs::metadata(cwd.join("objects")).map(|s| s.is_dir()).unwrap_or(false);
    let refs_exists = fs::metadata(cwd.join("refs")).map(|s| s.is_dir()).unwrap_or(false);

    head_exists || objects_exists || refs_exists
}
