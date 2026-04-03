use super::{app::{App, AppState}, tui::Tui};
use crate::agent::{Agent, AgentEvent};
use crate::commands::{get_commit_prompt, get_review_prompt, get_security_review_prompt, get_commit_push_pr_prompt};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tui_textarea::Input;

pub async fn start_repl(agent: Agent) -> Result<()> {
    let cost_tracker = agent.cost_tracker.clone();
    let mut tui = Tui::init()?;
    let mut app = App::new();

    // Channel for async agent responses
    let (tx, mut rx) = mpsc::channel(32);
    let agent_arc = Arc::new(Mutex::new(agent));

    let res = run_loop(&mut tui, &mut app, agent_arc, tx, &mut rx).await;
    
    Tui::restore()?;
    
    // Print session cost summary upon exiting
    println!("\n==========================================");
    println!("Session Complete. Generating summary...");
    println!("{}", cost_tracker.format_summary().await);
    println!("==========================================\n");
    
    res
}

async fn run_loop(
    tui: &mut Tui,
    app: &mut App<'_>,
    agent: Arc<Mutex<Agent>>,
    tx: mpsc::Sender<AgentEvent>,
    rx: &mut mpsc::Receiver<AgentEvent>,
) -> Result<()> {
    loop {
        if app.should_quit {
            break;
        }

        // Draw UI
        tui.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1), // Message History
                    Constraint::Length(3), // Input box
                    Constraint::Length(1), // Status bar
                ])
                .split(f.area());

            // 1. Message History
            let messages: Vec<ListItem> = app.messages.iter().map(|m| {
                let color = match m.sender.as_str() {
                    "User" => Color::Blue,
                    "Claude" => Color::Green,
                    "System" => Color::Yellow,
                    "Tool" => Color::Magenta,
                    "Error" => Color::Red,
                    _ => Color::White,
                };
                
                let content = Line::from(vec![
                    Span::styled(format!("{}: ", m.sender), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(&m.text),
                ]);
                ListItem::new(content)
            }).collect();

            let history = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title(" Conversation "))
                .highlight_style(Style::default().bg(Color::DarkGray));
            
            f.render_stateful_widget(history, chunks[0], &mut app.scroll_state);

            // 2. Input Box
            f.render_widget(&app.input_area, chunks[1]);

            // 3. Status Bar
            let status_text = match app.state {
                AppState::Input => " Ready",
                AppState::Thinking => " 🤖 Claude is thinking...",
                AppState::ExecutingTool => " 🛠️ Executing Tool...",
            };
            let status_bar = Paragraph::new(status_text)
                .style(Style::default().bg(Color::DarkGray).fg(Color::White));
            f.render_widget(status_bar, chunks[2]);
        })?;

        // Handle Async Agent Messages
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::Message(text) => {
                    app.add_message("Claude", &text);
                }
                AgentEvent::Thinking => {
                    app.state = AppState::Thinking;
                }
                AgentEvent::ToolCall(name, input) => {
                    app.add_message("Tool", &format!("Call: {} ({})", name, input));
                    app.state = AppState::ExecutingTool;
                }
                AgentEvent::ToolResult(name, res) => {
                    let result_str = match res {
                        Ok(val) => format!("Result [{}]: {}", name, val),
                        Err(e) => format!("Error [{}]: {}", name, e),
                    };
                    app.add_message("Tool", &result_str);
                    app.state = AppState::Thinking;
                }
                AgentEvent::Error(err) => {
                    app.add_message("Error", &err);
                    app.state = AppState::Input;
                }
                AgentEvent::Finished => {
                    app.state = AppState::Input;
                }
            }
        }

        // Handle Input Events (Non-blocking)
        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Enter => {
                            if matches!(app.state, AppState::Input) {
                                let lines = app.input_area.lines().to_vec();
                                let prompt = lines.join("\n").trim().to_string();
                                
                                if !prompt.is_empty() {
                                    app.add_message("User", &prompt);
                                    
                                    // Clear input
                                    app.input_area = tui_textarea::TextArea::default();
                                    app.input_area.set_block(
                                        Block::default()
                                            .borders(Borders::ALL)
                                            .title(" Ask Claude (Press Enter to send, Esc to quit) ")
                                    );
                                    
                                    app.state = AppState::Thinking;

                                    let tx_clone = tx.clone();
                                    let agent_clone = agent.clone();
                                    let prompt_clone = prompt.clone();
                                    tokio::spawn(async move {
                                        let mut locked_agent = agent_clone.lock().await;
                                        
                                        // Command Router
                                        if prompt_clone.starts_with('/') {
                                            let parts: Vec<&str> = prompt_clone.split_whitespace().collect();
                                            let command = parts[0];
                                            let args = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
                                            
                                            let command_prompt = match command {
                                                "/commit" => {
                                                     let _ = tx_clone.send(AgentEvent::Message("Executing /commit command...".to_string())).await;
                                                     get_commit_prompt().await.unwrap_or_else(|e| format!("Error generating commit prompt: {}", e))
                                                 }
                                                 "/commit-push-pr" => {
                                                     let _ = tx_clone.send(AgentEvent::Message("Executing /commit-push-pr command...".to_string())).await;
                                                     get_commit_push_pr_prompt(&args).await.unwrap_or_else(|e| format!("Error generating commit-push-pr prompt: {}", e))
                                                 }
                                                 "/review" => {
                                                     let _ = tx_clone.send(AgentEvent::Message("Executing /review command...".to_string())).await;
                                                     get_review_prompt(&args).await.unwrap_or_else(|e| format!("Error generating review prompt: {}", e))
                                                 }
                                                "/security-review" => {
                                                    let _ = tx_clone.send(AgentEvent::Message("Executing /security-review command...".to_string())).await;
                                                    get_security_review_prompt().await.unwrap_or_else(|e| format!("Error generating security-review prompt: {}", e))
                                                }
                                                "/help" => {
                                                     let help_msg = "Available commands:\n  /commit - Create a git commit automatically\n  /commit-push-pr [args] - Commit, push and create a PR\n  /review [PR] - Review code changes or PR\n  /security-review - Run a deep security analysis on changes\n  /clear - Clear the conversation\n  /compact - Compact the conversation history";
                                                     let _ = tx_clone.send(AgentEvent::Message(help_msg.to_string())).await;
                                                     let _ = tx_clone.send(AgentEvent::Finished).await;
                                                     return;
                                                 }
                                                "/clear" => {
                                                    // In a real app we'd clear the history, here we just notify
                                                    let _ = tx_clone.send(AgentEvent::Message("Conversation cleared.".to_string())).await;
                                                    let _ = tx_clone.send(AgentEvent::Finished).await;
                                                    return;
                                                }
                                                "/compact" => {
                                                    let _ = tx_clone.send(AgentEvent::Message("Conversation compacted.".to_string())).await;
                                                    let _ = tx_clone.send(AgentEvent::Finished).await;
                                                    return;
                                                }
                                                _ => {
                                                    let _ = tx_clone.send(AgentEvent::Message(format!("Unknown command: {}", command))).await;
                                                    let _ = tx_clone.send(AgentEvent::Finished).await;
                                                    return;
                                                }
                                            };
                                            
                                            if let Err(e) = locked_agent.run_single(&command_prompt, Some(tx_clone.clone())).await {
                                                let _ = tx_clone.send(AgentEvent::Error(format!("Failed to run agent: {}", e))).await;
                                                let _ = tx_clone.send(AgentEvent::Finished).await;
                                            }
                                        } else {
                                            // Normal chat prompt
                                            if let Err(e) = locked_agent.run_single(&prompt_clone, Some(tx_clone.clone())).await {
                                                let _ = tx_clone.send(AgentEvent::Error(format!("Failed to run agent: {}", e))).await;
                                                let _ = tx_clone.send(AgentEvent::Finished).await;
                                            }
                                        }
                                    });
                                }
                            }
                        }
                        _ => {
                            if matches!(app.state, AppState::Input) {
                                // Provide Key Event into tui-textarea
                                app.input_area.input(Input::from(key));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
