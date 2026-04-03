use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;
use taffy::TaffyTree;

/// Represents a UI node in our React-like TUI framework.
pub struct UiNode {
    pub id: NodeId,
    pub style: Style,
    pub children: Vec<Rc<RefCell<UiNode>>>,
    pub content: Option<String>,
    pub show_borders: bool,
}

pub struct FlexboxTuiEngine {
    taffy: TaffyTree,
    root_node: Rc<RefCell<UiNode>>,
}

impl FlexboxTuiEngine {
    pub fn new() -> Self {
        let mut taffy = TaffyTree::new();

        // Create a root node spanning the full terminal size
        let root_style = taffy::style::Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::Stretch),
            justify_content: Some(JustifyContent::FlexStart),
            ..Default::default()
        };

        let root_id = taffy.new_leaf(root_style).unwrap();

        let root_node = Rc::new(RefCell::new(UiNode {
            id: root_id,
            style: Style::default(),
            children: Vec::new(),
            content: None,
            show_borders: false,
        }));

        Self { taffy, root_node }
    }

    /// Helper to create a Box component (like Ink's `<Box>`)
    pub fn create_box(
        &mut self,
        taffy_style: taffy::style::Style,
        tui_style: Style,
        show_borders: bool,
    ) -> Rc<RefCell<UiNode>> {
        let node_id = self.taffy.new_leaf(taffy_style).unwrap();
        Rc::new(RefCell::new(UiNode {
            id: node_id,
            style: tui_style,
            children: Vec::new(),
            content: None,
            show_borders,
        }))
    }

    /// Helper to create a Text component (like Ink's `<Text>`)
    pub fn create_text(
        &mut self,
        content: String,
        tui_style: Style,
    ) -> Rc<RefCell<UiNode>> {
        let taffy_style = taffy::style::Style {
            // Text nodes tightly wrap their content (simplified)
            size: Size {
                width: Dimension::length(content.chars().count() as f32),
                height: Dimension::length(1.0), // Simplified to single line
            },
            ..Default::default()
        };

        let node_id = self.taffy.new_leaf(taffy_style).unwrap();
        Rc::new(RefCell::new(UiNode {
            id: node_id,
            style: tui_style,
            children: Vec::new(),
            content: Some(content),
            show_borders: false,
        }))
    }

    pub fn append_child(&mut self, parent: &Rc<RefCell<UiNode>>, child: Rc<RefCell<UiNode>>) {
        let parent_id = parent.borrow().id;
        let child_id = child.borrow().id;

        self.taffy.add_child(parent_id, child_id).unwrap();
        parent.borrow_mut().children.push(child);
    }

    pub fn set_root_children(&mut self, children: Vec<Rc<RefCell<UiNode>>>) {
        let root_id = self.root_node.borrow().id;
        let child_ids: Vec<NodeId> = children.iter().map(|c| c.borrow().id).collect();
        
        self.taffy.set_children(root_id, &child_ids).unwrap();
        self.root_node.borrow_mut().children = children;
    }

    /// Computes layout and renders to the Ratatui frame
    pub fn render(&mut self, f: &mut Frame) {
        let size = f.area();
        
        // 1. Compute Taffy Layout based on terminal size
        let available_space = Size {
            width: AvailableSpace::Definite(size.width as f32),
            height: AvailableSpace::Definite(size.height as f32),
        };

        self.taffy
            .compute_layout(self.root_node.borrow().id, available_space)
            .unwrap();

        // 2. Recursively paint nodes
        self.paint_node(f, &self.root_node.borrow(), size.x, size.y);
    }

    fn paint_node(&self, f: &mut Frame, node: &UiNode, offset_x: u16, offset_y: u16) {
        let layout = self.taffy.layout(node.id).unwrap();

        let x = offset_x + layout.location.x as u16;
        let y = offset_y + layout.location.y as u16;
        let width = layout.size.width as u16;
        let height = layout.size.height as u16;

        let rect = Rect::new(x, y, width, height);

        // Paint background / borders
        if node.show_borders {
            let block = Block::default()
                .borders(Borders::ALL)
                .style(node.style);
            f.render_widget(block, rect);
        }

        // Paint text content
        if let Some(ref text) = node.content {
            let paragraph = Paragraph::new(vec![Line::from(vec![Span::styled(
                text.clone(),
                node.style,
            )])]);
            
            // Render inside borders if any
            let inner_rect = if node.show_borders {
                Rect::new(x + 1, y + 1, width.saturating_sub(2), height.saturating_sub(2))
            } else {
                rect
            };
            
            f.render_widget(paragraph, inner_rect);
        }

        // Recurse to children
        for child in &node.children {
            self.paint_node(f, &child.borrow(), x, y);
        }
    }
}
