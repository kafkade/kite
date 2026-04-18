use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogChoice {
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DialogResult {
    Confirm,
    Cancel,
    Pending,
}

pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
    pub selected: DialogChoice,
}

#[allow(dead_code)]
impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "Yes".into(),
            cancel_label: "No".into(),
            selected: DialogChoice::Cancel,
        }
    }

    pub fn with_labels(mut self, confirm: impl Into<String>, cancel: impl Into<String>) -> Self {
        self.confirm_label = confirm.into();
        self.cancel_label = cancel.into();
        self
    }

    pub fn toggle_selection(&mut self) {
        self.selected = match self.selected {
            DialogChoice::Confirm => DialogChoice::Cancel,
            DialogChoice::Cancel => DialogChoice::Confirm,
        };
    }

    pub fn confirm(&self) -> DialogResult {
        match self.selected {
            DialogChoice::Confirm => DialogResult::Confirm,
            DialogChoice::Cancel => DialogResult::Cancel,
        }
    }

    pub fn selected(&self) -> DialogChoice {
        self.selected
    }
}

/// Center a rectangle of `width` x `height` within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .flex(Flex::Center)
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .flex(Flex::Center)
        .split(vertical[1]);

    horizontal[1]
}

pub fn render(frame: &mut Frame, dialog: &ConfirmDialog, theme: &Theme) {
    let area = frame.area();

    // Determine dialog dimensions based on content
    let min_width = dialog.message.len() as u16 + 6;
    let buttons_width = dialog.cancel_label.len() as u16 + dialog.confirm_label.len() as u16 + 13; // "[ X ] [ Y ]" + padding
    let title_width = dialog.title.len() as u16 + 6;
    let width = min_width.max(buttons_width).max(title_width).clamp(30, 60);
    let height: u16 = 7;

    let dialog_area = centered_rect(width, height, area);

    // Clear the area behind the dialog for a semi-transparent effect
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(format!(" {} ", dialog.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dialog_border));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Split inner area: message + spacing + buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top padding
            Constraint::Length(1), // message
            Constraint::Length(1), // spacing
            Constraint::Length(1), // buttons
            Constraint::Min(0),    // bottom padding
        ])
        .split(inner);

    // Render message centered
    let message = Paragraph::new(Line::from(dialog.message.as_str()))
        .style(Style::default().fg(theme.text_primary))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(message, chunks[1]);

    // Build button spans
    let (cancel_style, confirm_style) = match dialog.selected {
        DialogChoice::Cancel => (
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(theme.text_secondary),
        ),
        DialogChoice::Confirm => (
            Style::default().fg(theme.text_secondary),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let buttons = Line::from(vec![
        Span::styled(format!("[ {} ]", dialog.cancel_label), cancel_style),
        Span::raw("  "),
        Span::styled(format!("[ {} ]", dialog.confirm_label), confirm_style),
    ]);

    let buttons_paragraph = Paragraph::new(buttons).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(buttons_paragraph, chunks[3]);
}
