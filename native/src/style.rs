use iced::{
    Background, Border, Color, Theme, border,
    theme::Palette,
    widget::{button, container},
};

pub const INK: Color = Color::from_rgb8(231, 228, 218);
pub const MUTED: Color = Color::from_rgb8(152, 151, 146);
pub const PAPER: Color = Color::from_rgb8(29, 30, 31);
pub const SKY: Color = Color::from_rgb8(23, 24, 25);
pub const LINE: Color = Color::from_rgb8(55, 56, 57);
pub const WOOD: Color = Color::from_rgb8(172, 149, 108);

pub fn theme() -> Theme {
    Theme::custom(
        "Ballance".to_owned(),
        Palette {
            background: PAPER,
            text: INK,
            primary: WOOD,
            success: Color::from_rgb8(121, 166, 126),
            danger: Color::from_rgb8(210, 134, 126),
            warning: WOOD,
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
    style.border.radius = border::Radius::from(4.0);
    style
}

/// Transparent hit target: the actual game's texture supplies the button appearance.
pub fn original_button(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        text_color: Color::WHITE,
        ..button::Style::default()
    }
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
            style.background = Some(Background::Color(LINE));
        }
        style
    }
}

pub fn notice(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(58, 49, 39).into()),
        text_color: Some(INK),
        border: Border {
            radius: border::Radius::from(4.0),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn shade(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba8(0, 0, 0, 0.65).into()),
        ..container::Style::default()
    }
}
