use ratatui::widgets::BorderType;
use ratatui::symbols::border;

pub fn get_border_type() -> BorderType {
    #[cfg(any(target_arch = "xtensa", target_arch = "riscv32", feature = "simulator"))]
    {
        BorderType::Plain
    }
    #[cfg(not(any(target_arch = "xtensa", target_arch = "riscv32", feature = "simulator")))]
    {
        BorderType::Rounded
    }
}

pub fn get_border_set() -> border::Set<'static> {
    #[cfg(any(target_arch = "xtensa", target_arch = "riscv32", feature = "simulator"))]
    {
        border::Set {
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        }
    }
    #[cfg(not(any(target_arch = "xtensa", target_arch = "riscv32", feature = "simulator")))]
    {
        border::ROUNDED
    }
}
