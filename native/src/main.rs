#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod game;
mod preferences;
mod style;

fn main() -> iced::Result {
    iced::application(app::App::new, app::App::update, app::App::view)
        .title("Rockoon")
        .theme(style::theme())
        .subscription(app::App::subscription)
        .default_font(iced::Font::with_name("Microsoft YaHei UI"))
        .window(iced::window::Settings {
            size: iced::Size::new(1040.0, 700.0),
            min_size: Some(iced::Size::new(840.0, 580.0)),
            ..iced::window::Settings::default()
        })
        .centered()
        .run()
}
