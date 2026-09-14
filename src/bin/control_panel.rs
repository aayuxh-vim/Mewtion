use iced::widget::{button, column, container, pick_list, row, scrollable, slider, text, text_input};
use iced::{Alignment, Color, Element, Length, Task};
use std::fs;

pub fn main() -> iced::Result {
    iced::application(SettingsApp::default, SettingsApp::update, SettingsApp::view)
        .title("Mewtion Settings")
        .window_size((380.0, 720.0))
        .centered()
        .run()
}

const UI_BG_MAIN: Color = Color::from_rgb(0.12, 0.12, 0.13);
const UI_BG_CARD: Color = Color::from_rgb(0.18, 0.18, 0.20);
const UI_FG_TEXT: Color = Color::from_rgb(0.95, 0.95, 0.96);
const UI_FG_MUTED: Color = Color::from_rgb(0.60, 0.60, 0.64);

const COLOR_WHITE: Color = Color::from_rgb(1.00, 1.00, 1.00);
const COLOR_BLUE: Color = Color::from_rgb(0.00, 0.48, 1.00);
const COLOR_GREEN: Color = Color::from_rgb(0.20, 0.78, 0.35);
const COLOR_RED: Color = Color::from_rgb(1.00, 0.23, 0.19);
const COLOR_YELLOW: Color = Color::from_rgb(1.00, 0.80, 0.00);
const COLOR_PURPLE: Color = Color::from_rgb(0.69, 0.32, 0.87);
const COLOR_CYAN: Color = Color::from_rgb(0.00, 0.78, 0.75);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DotSize { Small, Medium, Large }
impl DotSize {
    fn pixels(&self) -> f32 { match self { DotSize::Small => 8.0, DotSize::Medium => 16.0, DotSize::Large => 24.0 } }
    fn label(&self) -> &'static str { match self { DotSize::Small => "Small", DotSize::Medium => "Medium", DotSize::Large => "Large" } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DotColor { White, Blue, Green, Red, Yellow, Purple, Cyan }
impl DotColor {
    fn to_color(&self) -> Color {
        match self {
            DotColor::White => COLOR_WHITE, DotColor::Blue => COLOR_BLUE,
            DotColor::Green => COLOR_GREEN, DotColor::Red => COLOR_RED,
            DotColor::Yellow => COLOR_YELLOW, DotColor::Purple => COLOR_PURPLE,
            DotColor::Cyan => COLOR_CYAN,
        }
    }
    fn hex_string(&self) -> &'static str {
        match self {
            DotColor::White => "#ffffff", DotColor::Blue => "#007aff",
            DotColor::Green => "#34c759", DotColor::Red => "#ff3b30",
            DotColor::Yellow => "#ffcc00", DotColor::Purple => "#af52de",
            DotColor::Cyan => "#00c7be",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimMode { Fluid, Rigid }
impl std::fmt::Display for AnimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self { AnimMode::Fluid => "Fluid (Organic)", AnimMode::Rigid => "Rigid (Mechanical)" })
    }
}

struct SettingsApp {
    selected_size: DotSize,
    selected_color: DotColor,
    opacity: f32,
    margin_pct: f32,
    sensitivity: f32,
    anim_mode: AnimMode,
    phone_ip: String,
    status_message: String,
}

impl Default for SettingsApp {
    fn default() -> Self {
        let path = "mewtion_config.txt";
        let mut app = Self {
            selected_size: DotSize::Medium, selected_color: DotColor::White,
            opacity: 0.85, margin_pct: 0.03, sensitivity: 3.5,
            anim_mode: AnimMode::Fluid, phone_ip: "127.0.0.1".to_string(),
            status_message: String::new(),
        };

        if let Ok(contents) = fs::read_to_string(path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() != 2 { continue; }
                let (k, v) = (parts[0].trim(), parts[1].trim());
                match k {
                    "opacity" => if let Ok(val) = v.parse::<f32>() { app.opacity = val; }
                    "margin" => if let Ok(val) = v.parse::<f32>() { app.margin_pct = val; }
                    "sensitivity" => if let Ok(val) = v.parse::<f32>() { app.sensitivity = val; }
                    "ip" => app.phone_ip = v.to_string(),
                    _ => {}
                }
            }
        }
        app
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SizeSelected(DotSize), ColorSelected(DotColor), OpacityChanged(f32),
    MarginChanged(f32), SensitivityChanged(f32), AnimModeChanged(AnimMode),
    IpChanged(String), SetUsbMode, AutoDetectHotspot, SaveClicked,
}

impl SettingsApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SizeSelected(size) => self.selected_size = size,
            Message::ColorSelected(color) => self.selected_color = color,
            Message::OpacityChanged(val) => self.opacity = val,
            Message::MarginChanged(val) => self.margin_pct = val,
            Message::SensitivityChanged(val) => self.sensitivity = val,
            Message::AnimModeChanged(mode) => self.anim_mode = mode,
            Message::IpChanged(ip) => self.phone_ip = ip,
            Message::SetUsbMode => self.phone_ip = "127.0.0.1".to_string(),
            Message::AutoDetectHotspot => {
                if let Ok(output) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("ip route | awk '/default/ {print $3}'")
                    .output()
                {
                    let detected_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !detected_ip.is_empty() {
                        self.phone_ip = detected_ip;
                        self.status_message = "Hotspot IP detected!".to_string();
                    } else {
                        self.status_message = "Could not find hotspot gateway.".to_string();
                    }
                } else {
                    self.status_message = "Failed to run network command.".to_string();
                }
            }
            Message::SaveClicked => {
                let config_data = format!(
                    "size={}\ncolor={}\nopacity={}\nmargin={}\nsensitivity={}\nanimation={}\nip={}\n",
                    self.selected_size.pixels(), self.selected_color.hex_string(),
                    self.opacity, self.margin_pct, self.sensitivity,
                    match self.anim_mode { AnimMode::Fluid => "Fluid", AnimMode::Rigid => "Rigid" },
                    self.phone_ip
                );
                match fs::write("mewtion_config.txt", config_data) {
                    Ok(_) => self.status_message = "Settings applied live!".to_string(),
                    Err(_) => self.status_message = "Error saving config".to_string(),
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let title = text("Mewtion Settings").size(22).color(UI_FG_TEXT);

        let preview_size = self.selected_size.pixels();
        let preview_dot = container(text("")).width(Length::Fixed(preview_size)).height(Length::Fixed(preview_size))
            .style(move |_| container::Style { background: Some(Color { a: self.opacity, ..self.selected_color.to_color() }.into()), border: iced::Border { radius: (preview_size / 2.0).into(), ..Default::default() }, ..Default::default() });

        let preview_area = container(preview_dot).width(Length::Fill).height(Length::Fixed(70.0)).center_x(Length::Fill).center_y(Length::Fill)
            .style(|_| container::Style { background: Some(UI_BG_CARD.into()), border: iced::Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() });

        let sizes_row = row![self.size_button(DotSize::Small), self.size_button(DotSize::Medium), self.size_button(DotSize::Large)].spacing(10);
        let colors_row = row![
            self.color_button(DotColor::White), self.color_button(DotColor::Blue), self.color_button(DotColor::Green),
            self.color_button(DotColor::Red), self.color_button(DotColor::Yellow), self.color_button(DotColor::Purple), self.color_button(DotColor::Cyan)
        ].spacing(10);

        let ip_input = text_input("e.g. 192.168.43.1", &self.phone_ip).on_input(Message::IpChanged).padding(8)
            .style(move |_theme, _status| text_input::Style { background: UI_BG_CARD.into(), border: iced::Border { radius: 4.0.into(), ..Default::default() }, icon: UI_FG_TEXT, placeholder: UI_FG_MUTED, value: UI_FG_TEXT, selection: COLOR_BLUE });
        
        let usb_btn = button(text("USB Mode").size(12).color(UI_FG_TEXT)).on_press(Message::SetUsbMode).padding(8)
            .style(move |_theme, _status| button::Style { background: Some(UI_BG_CARD.into()), ..Default::default() });

        let auto_btn = button(text("Auto-Detect Hotspot").size(12).color(UI_FG_TEXT)).on_press(Message::AutoDetectHotspot).padding(8)
            .style(move |_theme, _status| button::Style { background: Some(UI_BG_CARD.into()), ..Default::default() });

        let save_btn = button(text("Save & Apply").color(Color::WHITE).width(Length::Fill).align_x(Alignment::Center))
            .on_press(Message::SaveClicked).padding(12).style(|_theme, status| button::Style {
                background: Some(if status == button::Status::Hovered { Color::from_rgb(0.05, 0.55, 1.00).into() } else { COLOR_BLUE.into() }),
                border: iced::Border { radius: 6.0.into(), ..Default::default() }, ..Default::default()
            });

        let content = column![
            title,
            text("Live Preview").size(13).color(UI_FG_MUTED), preview_area,
            text("Connection Target").size(13).color(UI_FG_MUTED), 
            row![ip_input, usb_btn, auto_btn].spacing(10),
            text("Dot Size").size(13).color(UI_FG_MUTED), sizes_row,
            text("Dot Color").size(13).color(UI_FG_MUTED), colors_row,
            text(format!("Opacity: {:.0}%", self.opacity * 100.0)).size(13).color(UI_FG_MUTED), slider(0.1..=1.0, self.opacity, Message::OpacityChanged).step(0.05),
            text(format!("Edge Margin: {:.0}%", self.margin_pct * 100.0)).size(13).color(UI_FG_MUTED), slider(0.0..=0.15, self.margin_pct, Message::MarginChanged).step(0.01),
            text(format!("Motion Sensitivity: {:.1}", self.sensitivity)).size(13).color(UI_FG_MUTED), slider(1.0..=10.0, self.sensitivity, Message::SensitivityChanged).step(0.5),
            text("Animation Behavior").size(13).color(UI_FG_MUTED), pick_list(&[AnimMode::Fluid, AnimMode::Rigid][..], Some(self.anim_mode), Message::AnimModeChanged).width(Length::Fill),
            save_btn, text(&self.status_message).size(12).color(COLOR_GREEN),
        ].spacing(16).padding(24).max_width(380);

        container(scrollable(content)).width(Length::Fill).height(Length::Fill).center_x(Length::Fill)
            .style(|_| container::Style { background: Some(UI_BG_MAIN.into()), text_color: Some(UI_FG_TEXT), ..Default::default() }).into()
    }

    fn size_button(&self, size: DotSize) -> Element<Message> {
        let is_selected = self.selected_size == size;
        button(text(size.label()).color(if is_selected { Color::WHITE } else { UI_FG_TEXT })).on_press(Message::SizeSelected(size)).padding([8, 16])
            .style(move |_theme, status| button::Style { background: Some(if is_selected { COLOR_BLUE.into() } else if status == button::Status::Hovered { UI_BG_CARD.into() } else { UI_BG_MAIN.into() }), border: iced::Border { radius: 6.0.into(), width: 1.0, color: if is_selected { COLOR_BLUE } else { UI_BG_CARD } }, ..Default::default() }).into()
    }
    
    fn color_button(&self, color: DotColor) -> Element<Message> {
        let is_selected = self.selected_color == color;
        button(text("")).on_press(Message::ColorSelected(color)).width(Length::Fixed(26.0)).height(Length::Fixed(26.0))
            .style(move |_theme, _status| button::Style { background: Some(color.to_color().into()), border: iced::Border { radius: 13.0.into(), width: if is_selected { 3.0 } else { 0.0 }, color: Color::WHITE }, ..Default::default() }).into()
    }
}
