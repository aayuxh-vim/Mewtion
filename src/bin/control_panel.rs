use iced::widget::{button, column, container, row, text, Container};
use iced::{Alignment, Color, Element, Length, Task};
use std::fs;

pub fn main() -> iced::Result {
    iced::application(SettingsApp::default, SettingsApp::update, SettingsApp::view)
        .title("Mewtion Settings")
        .window_size((360.0, 420.0))
        .centered()
        .run()
}

// Background UI Colors (Dark Neutral Frame)
const UI_BG_MAIN: Color = Color::from_rgb(0.12, 0.12, 0.13); // #1f1f21
const UI_BG_CARD: Color = Color::from_rgb(0.18, 0.18, 0.20); // #2e2e33
const UI_FG_TEXT: Color = Color::from_rgb(0.95, 0.95, 0.96); // #f2f2f5
const UI_FG_MUTED: Color = Color::from_rgb(0.60, 0.60, 0.64); // #9999a3

// Vibrant Modern Dot Color Palette
const COLOR_WHITE: Color  = Color::from_rgb(1.00, 1.00, 1.00); // #ffffff (Apple Default)
const COLOR_BLUE: Color   = Color::from_rgb(0.00, 0.48, 1.00); // #007aff (iOS Blue)
const COLOR_GREEN: Color  = Color::from_rgb(0.20, 0.78, 0.35); // #34c759 (Emerald)
const COLOR_RED: Color    = Color::from_rgb(1.00, 0.23, 0.19); // #ff3b30 (Vivid Red)
const COLOR_YELLOW: Color = Color::from_rgb(1.00, 0.80, 0.00); // #ffcc00 (Bright Gold)
const COLOR_PURPLE: Color = Color::from_rgb(0.69, 0.32, 0.87); // #af52de (Neon Purple)
const COLOR_CYAN: Color   = Color::from_rgb(0.00, 0.78, 0.75); // #00c7be (Cyan/Teal)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DotSize {
    Small,
    Medium,
    Large,
}

impl DotSize {
    fn pixels(&self) -> f32 {
        match self {
            DotSize::Small => 8.0,
            DotSize::Medium => 16.0,
            DotSize::Large => 24.0,
        }
    }
    fn label(&self) -> &'static str {
        match self {
            DotSize::Small => "Small",
            DotSize::Medium => "Medium",
            DotSize::Large => "Large",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DotColor {
    White,
    Blue,
    Green,
    Red,
    Yellow,
    Purple,
    Cyan,
}

impl DotColor {
    fn to_color(&self) -> Color {
        match self {
            DotColor::White => COLOR_WHITE,
            DotColor::Blue => COLOR_BLUE,
            DotColor::Green => COLOR_GREEN,
            DotColor::Red => COLOR_RED,
            DotColor::Yellow => COLOR_YELLOW,
            DotColor::Purple => COLOR_PURPLE,
            DotColor::Cyan => COLOR_CYAN,
        }
    }

    fn hex_string(&self) -> &'static str {
        match self {
            DotColor::White => "#ffffff",
            DotColor::Blue => "#007aff",
            DotColor::Green => "#34c759",
            DotColor::Red => "#ff3b30",
            DotColor::Yellow => "#ffcc00",
            DotColor::Purple => "#af52de",
            DotColor::Cyan => "#00c7be",
        }
    }
}

struct SettingsApp {
    selected_size: DotSize,
    selected_color: DotColor,
    status_message: String,
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {
            selected_size: DotSize::Medium,
            selected_color: DotColor::White,
            status_message: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SizeSelected(DotSize),
    ColorSelected(DotColor),
    SaveClicked,
}

impl SettingsApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SizeSelected(size) => self.selected_size = size,
            Message::ColorSelected(color) => self.selected_color = color,
            Message::SaveClicked => {
                let config_data = format!(
                    "size={}\ncolor={}\n",
                    self.selected_size.pixels(),
                    self.selected_color.hex_string()
                );

                match fs::write("mewtion_config.txt", config_data) {
                    Ok(_) => self.status_message = "Saved to mewtion_config.txt".to_string(),
                    Err(_) => self.status_message = "Error saving config".to_string(),
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let title = text("Mewtion Settings")
            .size(20)
            .color(UI_FG_TEXT);

        // Preview box
        let preview_size = self.selected_size.pixels();
        let preview_dot = container(text(""))
            .width(Length::Fixed(preview_size))
            .height(Length::Fixed(preview_size))
            .style(move |_| container::Style {
                background: Some(self.selected_color.to_color().into()),
                border: iced::Border {
                    radius: (preview_size / 2.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let preview_area = container(preview_dot)
            .width(Length::Fill)
            .height(Length::Fixed(70.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(UI_BG_CARD.into()),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        // Size toggles
        let sizes_row = row![
            self.size_button(DotSize::Small),
            self.size_button(DotSize::Medium),
            self.size_button(DotSize::Large),
        ]
        .spacing(10);

        // Color swatches
        let colors_row = row![
            self.color_button(DotColor::White),
            self.color_button(DotColor::Blue),
            self.color_button(DotColor::Green),
            self.color_button(DotColor::Red),
            self.color_button(DotColor::Yellow),
            self.color_button(DotColor::Purple),
            self.color_button(DotColor::Cyan),
        ]
        .spacing(10);

        // Apply Button
        let save_btn = button(
            text("Save & Apply")
                .color(Color::WHITE)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .on_press(Message::SaveClicked)
        .padding(12)
        .style(|_theme, status| button::Style {
            background: Some(
                if status == button::Status::Hovered {
                    Color::from_rgb(0.05, 0.55, 1.00).into()
                } else {
                    COLOR_BLUE.into()
                },
            ),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let status = text(&self.status_message).size(12).color(COLOR_GREEN);

        let content = column![
            title,
            text("Live Preview").size(13).color(UI_FG_MUTED),
            preview_area,
            text("Dot Size").size(13).color(UI_FG_MUTED),
            sizes_row,
            text("Dot Color").size(13).color(UI_FG_MUTED),
            colors_row,
            save_btn,
            status,
        ]
        .spacing(14)
        .padding(24)
        .max_width(360);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .style(|_| container::Style {
                background: Some(UI_BG_MAIN.into()),
                text_color: Some(UI_FG_TEXT),
                ..Default::default()
            })
            .into()
    }

    fn size_button(&self, size: DotSize) -> Element<Message> {
        let is_selected = self.selected_size == size;
        button(text(size.label()).color(if is_selected { Color::WHITE } else { UI_FG_TEXT }))
            .on_press(Message::SizeSelected(size))
            .padding([8, 16])
            .style(move |_theme, status| button::Style {
                background: Some(
                    if is_selected {
                        COLOR_BLUE.into()
                    } else if status == button::Status::Hovered {
                        UI_BG_CARD.into()
                    } else {
                        UI_BG_MAIN.into()
                    },
                ),
                border: iced::Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: if is_selected { COLOR_BLUE } else { UI_BG_CARD },
                },
                ..Default::default()
            })
            .into()
    }

    fn color_button(&self, color: DotColor) -> Element<Message> {
        let is_selected = self.selected_color == color;
        let c = color.to_color();

        button(text(""))
            .on_press(Message::ColorSelected(color))
            .width(Length::Fixed(26.0))
            .height(Length::Fixed(26.0))
            .style(move |_theme, _status| button::Style {
                background: Some(c.into()),
                border: iced::Border {
                    radius: 13.0.into(),
                    width: if is_selected { 3.0 } else { 0.0 },
                    color: Color::WHITE,
                },
                ..Default::default()
            })
            .into()
    }
}
