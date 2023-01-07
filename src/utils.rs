use dirs::{download_dir, home_dir};
use iced::{
    alignment::Horizontal,
    theme::Button,
    widget::{button, container, text, Row},
    Length,
};

use crate::{Element, Message};

pub fn get_default_dir() -> String {
    download_dir()
        .unwrap_or_else(|| home_dir().unwrap_or("/".into()))
        .to_string_lossy()
        .to_string()
}

pub fn multi_toggle<'a, T: ToString + PartialEq + Clone>(
    options: Vec<T>,
    selected: &T,
    on_change: fn(T) -> Message,
) -> Element<'a> {
    container(
        options
            .iter()
            .fold(Row::new(), |row, option| {
                let text = text(option.to_string())
                    .horizontal_alignment(Horizontal::Center)
                    .width(Length::Fill);

                let button = button(text)
                    .style(if option == selected {
                        Button::Primary
                    } else {
                        Button::Text
                    })
                    .width(Length::Fill)
                    .on_press(on_change(option.clone()));

                row.push(button)
            })
            .spacing(0)
            .width(Length::Fill),
    )
    .style(iced::theme::Container::Box)
    .into()
}
