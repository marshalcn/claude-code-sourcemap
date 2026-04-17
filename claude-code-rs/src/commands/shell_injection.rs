use anyhow::Result;
use tokio::process::Command;

/// Parses and executes `!\`command\`` blocks inside a prompt string, replacing them
/// with the standard output of the executed shell command.
pub async fn execute_shell_commands_in_prompt(prompt: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = prompt.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '!' {
            if let Some(&'`') = chars.peek() {
                chars.next(); // consume '`'
                let mut cmd_str = String::new();
                
                // Read until the closing backtick
                while let Some(inner_c) = chars.next() {
                    if inner_c == '`' {
                        break;
                    }
                    cmd_str.push(inner_c);
                }
                
                // Execute the command
                if !cmd_str.is_empty() {
                    match Command::new("sh")
                        .arg("-c")
                        .arg(&cmd_str)
                        .output()
                        .await 
                    {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            
                            if !stdout.is_empty() {
                                result.push_str(&stdout);
                            }
                            if !stderr.is_empty() {
                                result.push_str("\n[stderr]:\n");
                                result.push_str(&stderr);
                            }
                        }
                        Err(e) => {
                            result.push_str(&format!("[Failed to execute '{}': {}]", cmd_str, e));
                        }
                    }
                }
                continue;
            }
        }
        result.push(c);
    }
    
    Ok(result)
}
