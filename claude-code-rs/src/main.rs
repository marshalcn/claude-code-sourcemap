pub mod agent;
pub mod api;
pub mod mcp;
pub mod memory;
pub mod tools;
pub mod ui;
pub mod utils;

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "claude-code-rs")]
#[command(about = "Rust implementation of Anthropic Claude Code CLI", long_about = None)]
struct Cli {
    /// Initial prompt or command
    #[arg(short, long)]
    prompt: Option<String>,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.debug { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting claude-code-rs...");

    // Setup Agent Context and Tool Registry
    let mut agent = agent::Agent::new();
    
    // Register basic tools
    agent.register_tool(Box::new(tools::BashTool::new()));
    agent.register_tool(Box::new(tools::FileReadTool::new()));
    agent.register_tool(Box::new(tools::FileWriteTool::new()));
    agent.register_tool(Box::new(tools::FileEditTool::new()));
    agent.register_tool(Box::new(tools::GlobTool::new()));
    agent.register_tool(Box::new(tools::GrepTool::new()));
    agent.register_tool(Box::new(tools::AskUserQuestionTool::new()));
    agent.register_tool(Box::new(tools::WebFetchTool::new()));
    agent.register_tool(Box::new(tools::WebSearchTool::new()));
    agent.register_tool(Box::new(tools::NotebookEditTool::new()));
    agent.register_tool(Box::new(tools::ListMcpResourcesTool::new()));
    agent.register_tool(Box::new(tools::ReadMcpResourceTool::new()));
    
    // Register Advanced Tools
    agent.register_tool(Box::new(tools::LspTool::new()));
    agent.register_tool(Box::new(tools::AgentTool::new()));
    agent.register_tool(Box::new(tools::ConfigTool::new()));
    agent.register_tool(Box::new(tools::BriefTool::new()));
    agent.register_tool(Box::new(tools::SleepTool::new()));
    agent.register_tool(Box::new(tools::ToolSearchTool::new()));
    
    // Register Task and Plan Tools
    agent.register_tool(Box::new(tools::TodoWriteTool::new()));
    agent.register_tool(Box::new(tools::TaskCreateTool::new()));
    agent.register_tool(Box::new(tools::TaskListTool::new()));
    agent.register_tool(Box::new(tools::TaskUpdateTool::new()));
    agent.register_tool(Box::new(tools::EnterPlanModeTool::new()));
    agent.register_tool(Box::new(tools::ExitPlanModeTool::new()));

    // Run the main interactive loop or process single prompt
    if let Some(prompt) = cli.prompt {
        info!("Running single prompt: {}", prompt);
        agent.run_single(&prompt, None).await?;
    } else {
        info!("Entering interactive REPL mode...");
        ui::repl::start_repl(agent).await?;
    }

    Ok(())
}
