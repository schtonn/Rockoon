use iced::{
    Border, Color, Theme, border,
    theme::Palette,
    widget::{
        button, checkbox, container, overlay::menu, pick_list, scrollable, slider, text_input,
    },
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

pub fn quiet(_: &Theme, status: button::Status) -> button::Style {
    let mut style = button::Style {
        text_color: INK,
        border: Border {
            radius: border::Radius::from(4.0),
            ..Border::default()
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => {
            style.background = Some(Color::from_rgb8(58, 51, 40).into());
            style.text_color = Color::from_rgb8(243, 226, 193);
            style.border.color = Color::from_rgb8(110, 94, 65);
            style.border.width = 1.0;
        }
        button::Status::Pressed => {
            style.background = Some(Color::from_rgb8(76, 63, 44).into());
            style.text_color = Color::WHITE;
            style.border.color = WOOD;
            style.border.width = 1.0;
        }
        button::Status::Disabled => style.text_color = MUTED,
        button::Status::Active => {}
    }
    style
}
pub fn nav(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = quiet(theme, status);
        if status == button::Status::Active {
            style.text_color = if selected {
                Color::from_rgb8(228, 207, 168)
            } else {
                INK
            };
            if selected {
                style.background = Some(Color::from_rgb8(48, 43, 35).into());
                style.border.color = Color::from_rgb8(99, 84, 59);
                style.border.width = 1.0;
            }
        }
        style
    }
}
pub fn tab(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = quiet(theme, status);
        if selected && status == button::Status::Active {
            style.text_color = Color::from_rgb8(228, 207, 168);
            style.background = Some(Color::from_rgb8(48, 43, 35).into());
            style.border.color = Color::from_rgb8(99, 84, 59);
            style.border.width = 1.0;
        }
        style
    }
}
pub fn menu(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(32, 33, 34).into()),
        text_color: Some(INK),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: border::Radius::from(5.0),
        },
        ..container::Style::default()
    }
}
pub fn toast(alpha: f32) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(49, 43, 34).scale_alpha(alpha).into()),
        text_color: Some(INK.scale_alpha(alpha)),
        border: Border {
            color: WOOD.scale_alpha(alpha * 0.6),
            width: 1.0,
            radius: border::Radius::from(5.0),
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

pub fn input(_: &Theme, status: text_input::Status) -> text_input::Style {
    let disabled = matches!(status, text_input::Status::Disabled);
    let color = match status {
        text_input::Status::Focused { .. } => WOOD,
        text_input::Status::Hovered => Color::from_rgb8(104, 94, 76),
        _ => LINE,
    };
    text_input::Style {
        background: SKY.into(),
        border: Border {
            color,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: MUTED,
        placeholder: MUTED,
        value: if disabled { MUTED } else { INK },
        selection: Color::from_rgb8(93, 78, 54),
    }
}

pub fn select(_: &Theme, status: pick_list::Status) -> pick_list::Style {
    let (color, background) = match status {
        pick_list::Status::Opened { .. } => (WOOD, Color::from_rgb8(48, 43, 35)),
        pick_list::Status::Hovered => (Color::from_rgb8(104, 94, 76), Color::from_rgb8(38, 37, 34)),
        pick_list::Status::Active => (LINE, SKY),
    };
    pick_list::Style {
        text_color: INK,
        placeholder_color: MUTED,
        handle_color: WOOD,
        background: background.into(),
        border: Border {
            color,
            width: 1.0,
            radius: 4.0.into(),
        },
    }
}

pub fn select_menu(_: &Theme) -> menu::Style {
    menu::Style {
        background: Color::from_rgb8(32, 33, 34).into(),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: INK,
        selected_text_color: Color::from_rgb8(243, 226, 193),
        selected_background: Color::from_rgb8(58, 51, 40).into(),
        shadow: iced::Shadow::default(),
    }
}

pub fn check(_: &Theme, status: checkbox::Status) -> checkbox::Style {
    let (checked, hovered, disabled) = match status {
        checkbox::Status::Active { is_checked } => (is_checked, false, false),
        checkbox::Status::Hovered { is_checked } => (is_checked, true, false),
        checkbox::Status::Disabled { is_checked } => (is_checked, false, true),
    };
    let accent = if disabled { MUTED } else { WOOD };
    checkbox::Style {
        background: if checked { accent.into() } else { SKY.into() },
        icon_color: SKY,
        border: Border {
            color: if checked || hovered {
                accent
            } else {
                Color::from_rgb8(98, 99, 96)
            },
            width: 1.0,
            radius: 3.0.into(),
        },
        text_color: Some(if disabled { MUTED } else { INK }),
    }
}

pub fn scroll(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    let (horizontal, vertical) = match status {
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => (
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        ),
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => (
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        ),
        _ => (false, false),
    };
    for (rail, hovered) in [
        (&mut style.horizontal_rail, horizontal),
        (&mut style.vertical_rail, vertical),
    ] {
        rail.background = None;
        rail.border = Border::default();
        rail.scroller.background = if hovered {
            WOOD
        } else {
            Color::from_rgb8(83, 82, 77)
        }
        .into();
        rail.scroller.border = Border {
            radius: 4.0.into(),
            ..Border::default()
        };
    }
    style
}

pub fn range(theme: &Theme, status: slider::Status) -> slider::Style {
    let mut style = slider::default(theme, status);
    style.rail.backgrounds = (WOOD.into(), LINE.into());
    style.rail.width = 3.0;
    style.handle.background = WOOD.into();
    style.handle.border_color = WOOD;
    style.with_circular_handle(6.0)
}
