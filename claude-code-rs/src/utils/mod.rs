// Common utilities: config parser, crypto helpers, path sanitization
pub mod config;
pub mod sys;
pub mod git;
pub mod chrome_host;
pub mod computer_use;

pub use git::{find_git_root, resolve_canonical_root, find_canonical_git_root, is_current_directory_bare_git_repo};
pub use chrome_host::{ChromeNativeHost, send_chrome_message, read_chrome_messages};
pub use computer_use::ComputerExecutor;
