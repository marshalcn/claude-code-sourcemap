use anyhow::Result;

pub mod repl {
    use super::*;
    use crate::agent::Agent;
    use std::io::{self, Write};
    
    pub async fn start_repl(agent: &mut Agent) -> Result<()> {
        println!("Welcome to Claude Code (Rust Edition)!");
        println!("Type '/help' for commands, or just start typing.");
        
        loop {
            print!("> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            if input == "/quit" || input == "/exit" {
                println!("Goodbye!");
                break;
            }
            
            // Delegate to the agent
            if let Err(e) = agent.run_single(input).await {
                eprintln!("Error executing prompt: {}", e);
            }
        }
        
        Ok(())
    }
}
