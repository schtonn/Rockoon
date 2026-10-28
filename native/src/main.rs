#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod assets;
mod blender;
mod bounds;
mod components;
mod config;
mod cover;
mod game;
mod hint;
mod home_balls;
mod hub;
mod icons;
mod jobs;
mod keys;
mod l10n;
mod launch;
mod media;
mod popup;
mod preferences;
mod process_guard;
mod profiles;
mod sky;
mod style;
mod system;
mod tas;
mod updates;
mod vanilla;
mod workspace;

fn main() -> iced::Result {
    let _instance = match system::start() {
        Ok(Some(guard)) => Some(guard),
        Ok(None) => return Ok(()),
        Err(_) => None,
    };
    #[cfg(target_os = "macos")]
    system::install_url_handler();
    iced::application(app::App::new, app::App::update, app::App::view)
        .title(if std::env::args().any(|a| a == "--tas-editor") {
            "Rockoon · TAS"
        } else {
            "Rockoon"
        })
        .theme(app::App::theme)
        .subscription(app::App::subscription)
        .font(include_bytes!("../resources/fonts/SourceHanSansCN-Regular.otf").as_slice())
        .font(include_bytes!("../resources/fonts/SourceHanSansCN-Medium.otf").as_slice())
        .default_font(iced::Font::with_name("Source Han Sans CN"))
        .window(iced::window::Settings {
            size: if std::env::args().any(|a| a == "--tas-editor") {
                iced::Size::new(1280.0, 960.0)
            } else {
                iced::Size::new(1040.0, 700.0)
            },
            exit_on_close_request: false,
            min_size: Some(iced::Size::new(840.0, 580.0)),
            ..iced::window::Settings::default()
        })
        .centered()
        .run()
}
