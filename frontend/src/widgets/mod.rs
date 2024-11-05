use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub mod chat;
pub mod input;

pub fn centered_rect(cx: Constraint, cy: Constraint, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::vertical([Constraint::Fill(1), cx, Constraint::Fill(1)]).split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), cy, Constraint::Fill(1)])
        .split(popup_layout[1])[1] // Return the middle chunk
}
