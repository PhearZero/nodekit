pub mod catchup;
pub mod exception;
pub mod hybrid;
pub mod partkey;
pub mod key_info;
pub mod lagging;
pub mod generate;
pub mod delete_confirm;
pub mod deleting;

pub trait ModalMetadata {
    fn title(&self) -> String;
    fn border_color(&self) -> ratatui::style::Color;
    fn controls(&self) -> String {
        String::new()
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        60
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        10
    }
}
