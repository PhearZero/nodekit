use crate::ui::viewport::ViewportComponent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::app::App;

pub mod theme;
pub mod modals;
pub mod pages;
pub mod protocol;
pub mod status;
pub mod viewport;

impl Widget for &App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let viewport = ViewportComponent::new(self);
        viewport.render(area, buf);
    }
}
