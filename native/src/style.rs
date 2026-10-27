use iced::{
    Background, Border, Color, Theme, border,
    theme::Palette,
    widget::{button, container},
};

pub const INK: Color = Color::from_rgb8(39, 56, 60);
pub const MUTED: Color = Color::from_rgb8(112, 129, 132);
pub const PAPER: Color = Color::from_rgb8(249, 249, 245);
pub const SKY: Color = Color::from_rgb8(234, 241, 240);
pub const LINE: Color = Color::from_rgb8(216, 225, 222);
pub const WOOD: Color = Color::from_rgb8(166, 100, 58);

pub fn theme() -> Theme {
    Theme::custom(
        "Clouds".to_owned(),
        Palette {
            background: PAPER,
            text: INK,
            primary: WOOD,
            success: Color::from_rgb8(66, 117, 100),
            danger: Color::from_rgb8(169, 65, 51),
            warning: Color::from_rgb8(166, 100, 58),
        },
    )
}

pub fn sidebar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SKY.into()),
        border: Border {
            color: LINE,
            width: 1.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = border::Radius::from(5.0);
    style
}

pub fn quiet(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.text_color = if status == button::Status::Disabled {
        MUTED
    } else {
        INK
    };
    style.border.radius = border::Radius::from(4.0);
    style
}

pub fn nav(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = quiet(theme, status);
        style.text_color = if selected { INK } else { MUTED };
        if selected || status == button::Status::Hovered {
            style.background = Some(Background::Color(if selected { PAPER } else { LINE }));
        }
        style
    }
}

pub fn notice(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(244, 234, 218).into()),
        text_color: Some(INK),
        border: Border {
            radius: border::Radius::from(4.0),
            ..Border::default()
        },
        ..container::Style::default()
    }
}
