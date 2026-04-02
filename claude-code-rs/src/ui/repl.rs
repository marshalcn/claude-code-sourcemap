use super::{app::{App, AppState}, tui::Tui};
use crate::agent::Agent;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::time::Duration;
use tokio::sync::mpsc;
use tui_textarea::Input;

pub async fn start_repl(agent: &mut Agent) -> Result<()> {
    let mut tui = Tui::init()?;
    let mut app = App::new();

    // Channel for async agent responses
    let (tx, mut rx) = mpsc::channel(32);

    let res = run_loop(&mut tui, &mut app, agent, tx, &mut rx).await;
    
    Tui::restore()?;
    res
}

async fn run_loop(
    tui: &mut Tui,
    app: &mut App<'_>,
    _agent: &mut Agent, // Agent will be used here later
    tx: mpsc::Sender<String>,
    rx: &mut mpsc::Receiver<String>,
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
        if let Ok(msg) = rx.try_recv() {
            app.add_message("Claude", &msg);
            app.state = AppState::Input;
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

                                    // TODO: Actually spawn the agent run task here
                                    // For now, we simulate a response
                                    let tx_clone = tx.clone();
                                    tokio::spawn(async move {
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                        let _ = tx_clone.send("I received your message! API integration coming soon.".to_string()).await;
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
