


use iced::widget::{button, column, slider, text, Column};
use iced::Task;

pub fn main() -> iced::Result {
    iced::application(SettingsApp::default, SettingsApp::update, SettingsApp::view)
        .title("Mewtion Settings")
        .centered()
        .run()
}

#[derive(Default)]
struct SettingsApp {
    sensitivity: f32,
    dot_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SensitivityChanged(f32),
    DotSizeChanged(f32),
    SaveClicked,
}

impl SettingsApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SensitivityChanged(val) => self.sensitivity = val,
            Message::DotSizeChanged(val) => self.dot_size = val,
            Message::SaveClicked => {
                println!("Saved settings: Sensitivity={}, Size={}", self.sensitivity, self.dot_size);
            }
        }
        Task::none()
    }

    fn view(&self) -> Column<Message> {
        column![
            text("Mewtion Settings").size(24),
            
            text(format!("Acceleration Sensitivity: {:.1}", self.sensitivity)),
            slider(0.5..=10.0, self.sensitivity, Message::SensitivityChanged).step(0.1_f32),

            text(format!("Dot Size: {:.1}px", self.dot_size)),
            slider(4.0..=16.0, self.dot_size, Message::DotSizeChanged).step(1.0_f32),

            button("Save & Apply").on_press(Message::SaveClicked),
        ]
        .padding(20)
        .spacing(15)
    }
}
