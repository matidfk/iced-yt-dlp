use std::{
    io::{BufRead, BufReader},
    process::{ChildStdout, Command, Stdio},
};

use iced::{
    alignment::Horizontal,
    widget::{column, progress_bar, row, text, Rule},
    Subscription,
};

use crate::{DownloaderMessage, Element, Message};

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub url: String,
    pub download_path: String,
    pub download_type: DownloadType,
    pub timestamp_start: Option<String>,
    pub timestamp_end: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Download {
    pub id: u32,

    pub options: DownloadOptions,
    pub name: Option<String>,
    pub progress: DownloadProgress,
}

enum YtDlp {
    Starting(Download),
    Running {
        id: u32,
        reader: BufReader<ChildStdout>,
        reader_state: ReaderState,
    },
    Finished,
}

enum ReaderState {
    WaitingForName,
    WaitingForProgress,
}

impl Download {
    pub fn new(options: DownloadOptions, id: u32) -> Self {
        Self {
            options,
            id,
            name: None,
            progress: DownloadProgress::Queued,
        }
    }

    // TODO: clean this up wtf
    pub fn run(&self) -> Subscription<Message> {
        iced::subscription::unfold(
            self.id,
            YtDlp::Starting(self.clone()),
            move |state| async move {
                match state {
                    YtDlp::Starting(download) => {
                        let id = download.id;

                        let child = download
                            .options
                            .clone()
                            .parse_command()
                            .stdout(Stdio::piped())
                            .spawn()
                            .unwrap();

                        let reader = BufReader::new(child.stdout.unwrap());

                        (
                            None,
                            YtDlp::Running {
                                id,
                                reader,
                                reader_state: ReaderState::WaitingForName,
                            },
                        )
                    }
                    YtDlp::Running {
                        id,
                        mut reader,
                        reader_state,
                    } => match reader_state {
                        ReaderState::WaitingForName => {
                            let mut buf = String::new();
                            while !buf.contains("Destination") {
                                reader.read_line(&mut buf).unwrap();
                            }
                            let name = buf.split('/').last().unwrap().to_string();

                            let mut trash = vec![];
                            reader.read_until(b'\r', &mut trash).unwrap();

                            (
                                Some(Message::Downloader(
                                    id,
                                    DownloaderMessage::NameObtained(name),
                                )),
                                YtDlp::Running {
                                    id,
                                    reader,
                                    reader_state: ReaderState::WaitingForProgress,
                                },
                            )
                        }
                        ReaderState::WaitingForProgress => {
                            let mut buf = vec![];
                            reader.read_until(b'\r', &mut buf).unwrap();
                            let buf = String::from_utf8_lossy(&buf);

                            // if done
                            if !buf.contains('%') {
                                return (
                                    Some(Message::Downloader(
                                        id,
                                        DownloaderMessage::DownloadFinished,
                                    )),
                                    YtDlp::Finished,
                                );
                            }

                            let progress =
                                buf.split_once('%').unwrap().0.split(' ').last().unwrap();

                            (
                                Some(Message::Downloader(
                                    id,
                                    DownloaderMessage::DownloadProgressed(
                                        progress.parse().unwrap(),
                                    ),
                                )),
                                YtDlp::Running {
                                    id,
                                    reader,
                                    reader_state,
                                },
                            )
                        }
                    },
                    YtDlp::Finished => (None, YtDlp::Finished),
                }
            },
        )
    }
}

#[derive(Debug, Clone, Default)]
pub enum DownloadProgress {
    #[default]
    Queued,
    Running(f32),
    Finished,
}

impl DownloadProgress {
    pub fn view(&self) -> Element {
        match self {
            DownloadProgress::Queued => text("Queued")
                .horizontal_alignment(Horizontal::Center)
                .into(),
            DownloadProgress::Running(progress) => row![
                progress_bar(0.0..=100.0, *progress),
                text(format!("{progress}%"))
            ]
            .spacing(10)
            .into(),
            DownloadProgress::Finished => text("Finished")
                .horizontal_alignment(Horizontal::Center)
                .into(),
        }
    }
}

impl Download {
    pub fn view(&self) -> Element {
        column![
            text(self.name.as_deref().unwrap_or(&self.options.url)),
            self.progress.view(),
            Rule::horizontal(1)
        ]
        .spacing(3)
        .into()
    }
}

impl DownloadOptions {
    pub fn parse_command(self) -> Command {
        let mut command = std::process::Command::new("yt-dlp");
        // url
        command.arg(&self.url);

        // output path
        command.args(["-P", &self.download_path]);

        // download type
        command.args(match self.download_type {
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
        if self.timestamp_start.is_some() || self.timestamp_end.is_some() {
            let start = self.timestamp_start.unwrap_or("0:00".to_string());
            let end = self.timestamp_end.unwrap_or("inf".to_string());

            command.args([
                "--download-sections",
                &format!("*{start}-{end}").to_string(),
            ]);
        }

        command
    }
}

#[derive(Debug, Clone, Default, PartialEq, Hash)]
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
