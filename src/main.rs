// disable terminal on windows
#![windows_subsystem = "windows"]

mod download_options;
mod utils;

use download_options::{Download, DownloadOptions, DownloadProgress, DownloadType};

use iced::theme::{Custom, Palette};
use iced::{
    alignment::Horizontal,
    executor,
    widget::{button, column, row, scrollable, text, text_input, Column, Space},
    Application, Color, Command, Length, Renderer, Theme,
};
use utils::{get_default_dir, multi_toggle};

fn main() -> iced::Result {
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: (600, 300),
            position: iced::window::Position::Centered,
            always_on_top: true,
            resizable: false,
            ..Default::default()
        },
        default_font: Some(include_bytes!("../fonts/FiraSans-Medium.ttf")),
        ..Default::default()
    })
}

pub type Element<'a> = iced::Element<'a, Message, Renderer>;

struct App {
    url: String,
    download_path: String,
    download_type: DownloadType,
    timestamp_start: String,
    timestamp_end: String,

    downloads: Vec<Download>,
    last_id: u32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            download_path: get_default_dir(),
            download_type: DownloadType::Video,
            timestamp_start: "".to_string(),
            timestamp_end: "".to_string(),

            downloads: vec![],
            last_id: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    UrlChanged(String),
    DownloadTypeChanged(DownloadType),
    DownloadPathChanged(String),
    TimestampStartChanged(String),
    TimestampEndChanged(String),

    Downloader(u32, DownloaderMessage),

    Browse,
    StartDownload,
}

#[derive(Debug, Clone)]
pub enum DownloaderMessage {
    NameObtained(String),
    DownloadProgressed(f32),
    DownloadFinished,
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
            Message::UrlChanged(url) => self.url = url,
            Message::DownloadPathChanged(download_path) => self.download_path = download_path,
            Message::DownloadTypeChanged(download_type) => self.download_type = download_type,
            Message::TimestampStartChanged(timestamp_start) => {
                self.timestamp_start = timestamp_start
            }
            Message::TimestampEndChanged(timestamp_end) => self.timestamp_end = timestamp_end,

            Message::StartDownload => {
                self.downloads.push(Download::new(
                    DownloadOptions {
                        url: self.url.clone(),
                        download_path: self.download_path.clone(),
                        download_type: self.download_type.clone(),
                        timestamp_start: if !self.timestamp_start.is_empty() {
                            Some(self.timestamp_start.clone())
                        } else {
                            None
                        },
                        timestamp_end: if !self.timestamp_end.is_empty() {
                            Some(self.timestamp_end.clone())
                        } else {
                            None
                        },
                    },
                    self.last_id,
                ));
                self.last_id += 1;

                // reset fields
                self.url = "".to_string();
                self.timestamp_start = "".to_string();
                self.timestamp_end = "".to_string();
            }
            Message::Browse => {
                let folder = rfd::FileDialog::new()
                    .set_directory(&self.download_path)
                    .pick_folder();

                if let Some(folder) = folder {
                    self.download_path = folder.to_string_lossy().to_string();
                }
            }
            Message::Downloader(id, message) => {
                let mut downloader = self.downloads.iter_mut().find(|d| d.id == id).unwrap();

                match message {
                    DownloaderMessage::NameObtained(name) => downloader.name = Some(name),
                    DownloaderMessage::DownloadProgressed(new_progress) => {
                        downloader.progress = DownloadProgress::Running(new_progress);
                    }
                    DownloaderMessage::DownloadFinished => {
                        downloader.progress = DownloadProgress::Finished;
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element {
        let url = text_input("Enter YouTube URL", &self.url, Message::UrlChanged);

        let download_path = row![
            text_input(
                "Download path",
                &self.download_path,
                Message::DownloadPathChanged
            ),
            button("Browse").on_press(Message::Browse)
        ]
        .spacing(10)
        .width(Length::Fill);

        let download_type = multi_toggle(
            vec![DownloadType::Audio, DownloadType::Video],
            &self.download_type,
            Message::DownloadTypeChanged,
        );

        let time_range = column![
            text("Download time range")
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center),
            Space::with_height(Length::Units(10)),
            row![
                text("From"),
                text_input(
                    "0:00",
                    &self.timestamp_start,
                    Message::TimestampStartChanged
                ),
                text("To"),
                text_input("inf", &self.timestamp_end, Message::TimestampEndChanged),
            ]
            .spacing(30)
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

        let children = scrollable(
            self.downloads
                .iter()
                .fold(Column::new(), |column, download| {
                    column.push(download.view())
                }),
        )
        .height(Length::Fill);

        column![
            url,
            download_path,
            download_type,
            time_range,
            // Space::with_height(Length::Fill),
            download_button,
            children
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
        iced::Subscription::batch(self.downloads.iter().map(Download::run))
    }
}
