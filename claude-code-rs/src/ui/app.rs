use ratatui::widgets::ListState;
use tui_textarea::TextArea;

pub enum AppState {
    Input,
    Thinking,
    ExecutingTool,
}

pub struct Message {
    pub sender: String,
    pub text: String,
}

pub struct App<'a> {
    pub state: AppState,
    pub messages: Vec<Message>,
    pub input_area: TextArea<'a>,
    pub scroll_state: ListState,
    pub should_quit: bool,
    pub current_tool: Option<String>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(" Ask Claude (Press Enter to send, Esc to quit) ")
        );

        Self {
            state: AppState::Input,
            messages: vec![Message {
                sender: "System".to_string(),
                text: "Welcome to Claude Code (Rust Edition)!".to_string(),
            }],
            input_area,
            scroll_state: ListState::default(),
            should_quit: false,
            current_tool: None,
        }
    }

    pub fn add_message(&mut self, sender: &str, text: &str) {
        self.messages.push(Message {
            sender: sender.to_string(),
            text: text.to_string(),
        });
        // Auto scroll to bottom
        self.scroll_state.select(Some(self.messages.len().saturating_sub(1)));
    }
}
