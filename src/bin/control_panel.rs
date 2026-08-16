use iced::widget::{button, column, container, row, text, Container};
use iced::{Alignment, Color, Element, Length, Task, Theme};
use std::fs;

pub fn main() -> iced::Result {
    iced::application(SettingsApp::default, SettingsApp::update, SettingsApp::view)
        .title("Mewtion Settings")
        .window_size((350.0, 400.0))
        .centered()
        .run()
}

// Strict Gruvbox Dark Palette
const GB_BG0: Color = Color::from_rgb(0.157, 0.157, 0.157); // #282828
const GB_BG2: Color = Color::from_rgb(0.314, 0.286, 0.271); // #504945
const GB_FG: Color = Color::from_rgb(0.922, 0.859, 0.698);  // #ebdbb2
const GB_GRAY: Color = Color::from_rgb(0.573, 0.514, 0.451); // #928374

const GB_RED: Color = Color::from_rgb(0.800, 0.141, 0.114);    // #cc241d
const GB_GREEN: Color = Color::from_rgb(0.596, 0.592, 0.102);  // #98971a
const GB_YELLOW: Color = Color::from_rgb(0.843, 0.600, 0.129); // #d79921
const GB_BLUE: Color = Color::from_rgb(0.271, 0.522, 0.533);   // #458588
const GB_PURPLE: Color = Color::from_rgb(0.694, 0.384, 0.525); // #b16286
const GB_AQUA: Color = Color::from_rgb(0.408, 0.616, 0.416);   // #689d6a
const GB_WHITE: Color = Color::from_rgb(0.659, 0.600, 0.518);  // #a89984

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
    Red, Green, Yellow, Blue, Purple, Aqua, White,
}

impl DotColor {
    fn to_color(&self) -> Color {
        match self {
            DotColor::Red => GB_RED,
            DotColor::Green => GB_GREEN,
            DotColor::Yellow => GB_YELLOW,
            DotColor::Blue => GB_BLUE,
            DotColor::Purple => GB_PURPLE,
            DotColor::Aqua => GB_AQUA,
            DotColor::White => GB_WHITE,
        }
    }
    fn hex_string(&self) -> &'static str {
        match self {
            DotColor::Red => "#cc241d",
            DotColor::Green => "#98971a",
            DotColor::Yellow => "#d79921",
            DotColor::Blue => "#458588",
            DotColor::Purple => "#b16286",
            DotColor::Aqua => "#689d6a",
            DotColor::White => "#a89984",
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
            selected_color: DotColor::Green,
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
        let title = text("Mewtion Configuration")
            .size(22)
            .color(GB_FG)
            .style(|_| text::Style { ..Default::default() });

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
            .height(Length::Fixed(60.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(GB_BG2.into()),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let sizes_row = row![
            self.size_button(DotSize::Small),
            self.size_button(DotSize::Medium),
            self.size_button(DotSize::Large),
        ]
        .spacing(10);

        let colors_row = row![
            self.color_button(DotColor::Red),
            self.color_button(DotColor::Green),
            self.color_button(DotColor::Yellow),
            self.color_button(DotColor::Blue),
            self.color_button(DotColor::Purple),
            self.color_button(DotColor::Aqua),
            self.color_button(DotColor::White),
        ]
        .spacing(8);

        let save_btn = button(
            text("Save & Apply")
                .color(GB_BG0)
                .width(Length::Fill)
                .align_x(Alignment::Center)
        )
        .on_press(Message::SaveClicked)
        .padding(12)
        .style(|_theme, status| button::Style {
            background: Some(
                if status == button::Status::Hovered { GB_YELLOW.into() } else { GB_GREEN.into() }
            ),
            border: iced::Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        });

        let status = text(&self.status_message).size(12).color(GB_GRAY);

        let content = column![
            title,
            text("Live Preview").size(14).color(GB_GRAY),
            preview_area,
            text("Dot Size").size(14).color(GB_GRAY),
            sizes_row,
            text("Theme Color").size(14).color(GB_GRAY),
            colors_row,
            save_btn,
            status,
        ]
        .spacing(15)
        .padding(25)
        .max_width(350);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .style(|_| container::Style {
                background: Some(GB_BG0.into()),
                text_color: Some(GB_FG),
                ..Default::default()
            })
            .into()
    }

    fn size_button(&self, size: DotSize) -> Element<Message> {
        let is_selected = self.selected_size == size;
        button(text(size.label()).color(if is_selected { GB_BG0 } else { GB_FG }))
            .on_press(Message::SizeSelected(size))
            .padding([8, 16])
            .style(move |_theme, status| button::Style {
                background: Some(
                    if is_selected { GB_GRAY.into() } 
                    else if status == button::Status::Hovered { GB_BG2.into() } 
                    else { GB_BG0.into() }
                ),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: if is_selected { GB_GRAY } else { GB_BG2 },
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
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .style(move |_theme, _status| button::Style {
                background: Some(c.into()),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: if is_selected { 3.0 } else { 0.0 },
                    color: GB_FG,
                },
                ..Default::default()
            })
            .into()
    }
}
