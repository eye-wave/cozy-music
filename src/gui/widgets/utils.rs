use iced::widget::svg::Handle;
use iced::widget::{Svg, svg};

pub fn gen_svg_icon(bytes: &'static [u8]) -> Svg<'static> {
    let handle = Handle::from_memory(bytes);
    svg(handle)
}
