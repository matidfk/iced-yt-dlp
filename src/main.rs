#![feature(fn_traits)]

use dirs::{download_dir, home_dir};
use iced::{
    alignment::Horizontal,
    executor,
    theme::{Button, Custom, Palette},
    widget::{button, column, row, text, text_input, Row, Space},
    Application, Color, Command, Length, Renderer, Theme,
};

fn main() {
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: (600, 300),
            position: iced::window::Position::Centered,
            always_on_top: true,
            ..Default::default()
        },
        default_font: Some(include_bytes!("../fonts/FiraSans-Medium.ttf")),
        ..Default::default()
    })
    .unwrap();
}

struct App {
    download_url: String,
    download_path: String,
    download_type: DownloadType,
    timestamp_start: String,
    timestamp_end: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            download_url: "".to_string(),
            download_path: download_dir()
                .unwrap_or_else(|| home_dir().unwrap_or("/".into()))
                .to_string_lossy()
                .to_string(),
            download_type: DownloadType::Video,
            timestamp_start: "0:00".to_string(),
            timestamp_end: "inf".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum DownloadType {
    #[default]
    Video,
    Audio,
}

impl ToString for DownloadType {
    fn to_string(&self) -> String {
        match self {
            DownloadType::Video => "Video".to_string(),
            DownloadType::Audio => "Audio".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    DownloadUrlChanged(String),
    DownloadTypeChanged(DownloadType),
    DownloadPathChanged(String),
    TimestampStartChanged(String),
    TimestampEndChanged(String),
    PickFolder,
    StartDownload,
}

impl Application for App {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Iced yt downloader".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::DownloadUrlChanged(v) => self.download_url = v,
            Message::DownloadTypeChanged(t) => self.download_type = t,
            Message::StartDownload => {
                let mut command = std::process::Command::new("yt-dlp");
                command.arg(self.download_url.clone());

                // output path
                command.args(["-P", &self.download_path]);

                // download type
                command.args([
                    "-f",
                    match self.download_type {
                        DownloadType::Video => "bestvideo",
                        DownloadType::Audio => "bestaudio",
                    },
                ]);
                command.args(match &self.download_type {
                    DownloadType::Video => vec!["-f", "bestvideo"],
                    DownloadType::Audio => vec![
                        "-f",
                        "bestaudio",
                        "--extract-audio",
                        "--audio-format",
                        "mp3",
                    ],
                });

                // time range
                if self.timestamp_start != "0:00" && self.timestamp_end != "inf" {
                    let start = if self.timestamp_start.is_empty() {
                        "0:00".to_string()
                    } else {
                        format!("*{}", self.timestamp_start)
                    };

                    let end = if self.timestamp_end.is_empty() {
                        "inf".to_string()
                    } else {
                        format!("{}", self.timestamp_start)
                    };

                    command.args(["--download-sections", &format!("{}-{}", start, end)]);
                }

                notify_rust::Notification::new()
                    .summary("Downloading video ...")
                    .body(&format!(
                        "{} into {}",
                        self.download_url, self.download_path
                    ))
                    .show()
                    .unwrap();

                let output = command.output().unwrap();

                notify_rust::Notification::new()
                    .body(&String::from_utf8(output.stdout).unwrap())
                    .show()
                    .unwrap();
            }
            Message::DownloadPathChanged(v) => self.download_path = v,
            Message::PickFolder => {
                let folder = rfd::FileDialog::new()
                    .set_directory(self.download_path.clone())
                    .pick_folder();

                if let Some(folder) = folder {
                    self.download_path = folder.to_string_lossy().to_string();
                }
            }
            Message::TimestampStartChanged(v) => self.timestamp_start = v,
            Message::TimestampEndChanged(v) => self.timestamp_end = v,
        }
        Command::none()
    }

    fn view(&self) -> Element {
        let path_row = row![
            text_input(
                "Download path",
                &self.download_path,
                Message::DownloadPathChanged
            ),
            button("Choose").on_press(Message::PickFolder)
        ]
        .spacing(10)
        .width(Length::Fill);

        let time_range = column![
            text("Download time range")
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center),
            Space::with_height(Length::Units(10)),
            row![
                text("From"),
                Space::with_width(Length::Units(30)),
                text_input("-", &self.timestamp_start, Message::TimestampStartChanged),
                Space::with_width(Length::Units(30)),
                text("To"),
                Space::with_width(Length::Units(30)),
                text_input("-", &self.timestamp_end, Message::TimestampEndChanged),
            ]
            .width(Length::Fill)
        ];

        let download_button = button(
            text("START DOWNLOAD")
                .size(30)
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .on_press(Message::StartDownload);

        column![
            text_input(
                "Enter YouTube URL",
                &self.download_url,
                Message::DownloadUrlChanged
            ),
            path_row,
            multi_toggle(
                vec![DownloadType::Audio, DownloadType::Video],
                self.download_type.clone(),
                |t| Message::DownloadTypeChanged(t.clone())
            ),
            time_range,
            Space::with_height(Length::Fill),
            download_button
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::Custom(Box::new(Custom::new(Palette {
            background: Color::new(0.1, 0.1, 0.1, 1.0),
            text: Color::new(0.9, 0.9, 0.9, 1.0),
            primary: Color::new(1.0, 0.04, 0.04, 1.0),
            success: Color::new(0.2, 1.0, 0.1, 1.0),
            danger: Color::new(0.17, 0.17, 0.17, 1.0),
        })))
    }
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::none()
    }
}
pub type Element<'a> = iced::Element<'a, Message, Renderer>;

pub fn multi_toggle<'a, T: ToString + PartialEq>(
    options: Vec<T>,
    selected: T,
    message: impl Fn(&T) -> Message,
) -> Element<'a> {
    options
        .iter()
        .fold(Row::new(), |row, option| {
            let text = text(option.to_string())
                .horizontal_alignment(Horizontal::Center)
                .width(Length::Fill);

            let button = button(text)
                .style(if *option == selected {
                    Button::Primary
                } else {
                    Button::Destructive
                })
                .width(Length::Fill)
                .on_press(message.call((option,)));

            row.push(button)
        })
        .spacing(0)
        .width(Length::Fill)
        .into()
}
