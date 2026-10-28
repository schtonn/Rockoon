use crate::preferences::ColorMode;
use iced::{
    Border, Color, Theme,
    theme::Palette,
    widget::{
        button, checkbox, container, overlay::menu, pick_list, scrollable, slider, text_input,
    },
};

#[derive(Clone, Copy)]
pub struct Colors {
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub muted: Color,
    pub line: Color,
    pub hover: Color,
    pub selected: Color,
    pub focus: Color,
    pub solid: Color,
    pub on_solid: Color,
}
fn palette(mode: ColorMode) -> Colors {
    if mode == ColorMode::Light {
        Colors {
            background: Color::from_rgb8(246, 247, 248),
            surface: Color::WHITE,
            text: Color::from_rgb8(34, 38, 43),
            muted: Color::from_rgb8(114, 121, 130),
            line: Color::from_rgb8(217, 222, 228),
            hover: Color::from_rgb8(235, 238, 242),
            selected: Color::from_rgb8(223, 228, 234),
            focus: Color::from_rgb8(105, 117, 132),
            solid: Color::from_rgb8(67, 77, 90),
            on_solid: Color::WHITE,
        }
    } else {
        Colors {
            background: Color::from_rgb8(27, 29, 32),
            surface: Color::from_rgb8(22, 24, 27),
            text: Color::from_rgb8(229, 233, 238),
            muted: Color::from_rgb8(146, 154, 165),
            line: Color::from_rgb8(56, 62, 70),
            hover: Color::from_rgb8(40, 45, 52),
            selected: Color::from_rgb8(51, 58, 68),
            focus: Color::from_rgb8(139, 154, 174),
            solid: Color::from_rgb8(189, 199, 213),
            on_solid: Color::from_rgb8(25, 29, 35),
        }
    }
}
pub fn colors(theme: &Theme) -> Colors {
    palette(if theme.palette().background.r > 0.5 {
        ColorMode::Light
    } else {
        ColorMode::Dark
    })
}
pub fn theme(mode: ColorMode) -> Theme {
    let c = palette(mode);
    Theme::custom(
        format!("Ballance {mode:?}"),
        Palette {
            background: c.background,
            text: c.text,
            primary: c.solid,
            success: Color::from_rgb8(78, 148, 107),
            danger: Color::from_rgb8(188, 91, 98),
            warning: c.focus,
        },
    )
}
fn outline(color: Color) -> Border {
    Border {
        color,
        width: 1.0,
        radius: 4.0.into(),
    }
}
pub fn sidebar(theme: &Theme) -> container::Style {
    let c = colors(theme);
    container::Style {
        background: Some(c.surface.into()),
        text_color: Some(c.text),
        border: Border {
            color: c.line,
            width: 1.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}
pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let c = colors(theme);
    button::Style {
        background: Some(
            match status {
                button::Status::Hovered => c.focus,
                button::Status::Disabled => c.selected,
                _ => c.solid,
            }
            .into(),
        ),
        text_color: if status == button::Status::Disabled {
            c.muted
        } else {
            c.on_solid
        },
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
/// The source texture and its bitmap caption retain their original colors in either theme.
pub fn original_button(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        text_color: Color::WHITE,
        ..button::Style::default()
    }
}
pub fn quiet(theme: &Theme, status: button::Status) -> button::Style {
    let c = colors(theme);
    button::Style {
        text_color: if status == button::Status::Disabled {
            c.muted
        } else {
            c.text
        },
        background: match status {
            button::Status::Hovered => Some(c.hover.into()),
            button::Status::Pressed => Some(c.selected.into()),
            _ => None,
        },
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = quiet(theme, status);
    style.border = outline(colors(theme).line);
    style
}
pub fn sky_button(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: Color::WHITE,
        background: match status {
            button::Status::Hovered => Some(Color::from_rgba8(0, 0, 0, 0.16).into()),
            button::Status::Pressed => Some(Color::from_rgba8(0, 0, 0, 0.28).into()),
            _ => None,
        },
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
pub fn nav(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = quiet(theme, status);
        if selected && status == button::Status::Active {
            style.background = Some(colors(theme).selected.into());
        }
        style
    }
}
pub fn tab(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = nav(selected)(theme, status);
        if let Some(iced::Background::Color(color)) = style.background {
            style.background = Some(color.scale_alpha(0.76).into());
        }
        style
    }
}
pub fn tabs(theme: &Theme) -> container::Style {
    let mut style = menu(theme);
    style.border.radius = 6.0.into();
    style.background = Some(colors(theme).surface.scale_alpha(0.32).into());
    style
}
pub fn menu(theme: &Theme) -> container::Style {
    let c = colors(theme);
    container::Style {
        background: Some(c.surface.into()),
        text_color: Some(c.text),
        border: outline(c.line),
        ..container::Style::default()
    }
}
pub fn toast(theme: &Theme, alpha: f32) -> container::Style {
    let c = colors(theme);
    container::Style {
        background: Some(c.surface.scale_alpha(alpha).into()),
        text_color: Some(c.text.scale_alpha(alpha)),
        border: outline(c.focus.scale_alpha(alpha)),
        ..container::Style::default()
    }
}
pub fn shade(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba8(0, 0, 0, 0.5).into()),
        ..container::Style::default()
    }
}
pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let c = colors(theme);
    let color = match status {
        text_input::Status::Focused { .. } => c.focus,
        text_input::Status::Hovered => c.muted,
        _ => c.line,
    };
    text_input::Style {
        background: c
            .surface
            .scale_alpha(match status {
                text_input::Status::Focused { .. } => 0.58,
                text_input::Status::Hovered => 0.48,
                _ => 0.36,
            })
            .into(),
        border: outline(color),
        icon: c.muted,
        placeholder: c.muted,
        value: if matches!(status, text_input::Status::Disabled) {
            c.muted
        } else {
            c.text
        },
        selection: c.selected,
    }
}
pub fn select(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let c = colors(theme);
    let (color, background) = match status {
        pick_list::Status::Opened { .. } => (c.focus, c.hover),
        pick_list::Status::Hovered => (c.muted, c.hover),
        pick_list::Status::Active => (c.line, c.surface),
    };
    pick_list::Style {
        text_color: c.text,
        placeholder_color: c.muted,
        handle_color: c.muted,
        background: background
            .scale_alpha(match status {
                pick_list::Status::Opened { .. } => 0.76,
                pick_list::Status::Hovered => 0.58,
                pick_list::Status::Active => 0.36,
            })
            .into(),
        border: outline(color),
    }
}
pub fn select_menu(theme: &Theme) -> menu::Style {
    let c = colors(theme);
    menu::Style {
        background: c.surface.into(),
        border: outline(c.line),
        text_color: c.text,
        selected_text_color: c.text,
        selected_background: c.selected.into(),
        shadow: iced::Shadow::default(),
    }
}
pub fn check(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let c = colors(theme);
    let (checked, hovered, disabled) = match status {
        checkbox::Status::Active { is_checked } => (is_checked, false, false),
        checkbox::Status::Hovered { is_checked } => (is_checked, true, false),
        checkbox::Status::Disabled { is_checked } => (is_checked, false, true),
    };
    let accent = if disabled { c.muted } else { c.solid };
    checkbox::Style {
        background: if checked {
            accent.into()
        } else {
            c.surface.into()
        },
        icon_color: c.on_solid,
        border: Border {
            color: if checked || hovered { accent } else { c.muted },
            width: 1.0,
            radius: 3.0.into(),
        },
        text_color: Some(if disabled { c.muted } else { c.text }),
    }
}
pub fn scroll(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let c = colors(theme);
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
        rail.scroller.background = if hovered { c.focus } else { c.line }.into();
        rail.scroller.border = Border {
            radius: 4.0.into(),
            ..Border::default()
        };
    }
    style
}
pub fn range(theme: &Theme, status: slider::Status) -> slider::Style {
    let c = colors(theme);
    let mut style = slider::default(theme, status);
    style.rail.backgrounds = (c.focus.into(), c.line.into());
    style.rail.width = 3.0;
    style.handle.background = c.solid.into();
    style.handle.border_color = c.solid;
    style.with_circular_handle(6.0)
}

pub fn catalog_card(theme: &Theme) -> container::Style {
    let c = colors(theme);
    container::Style {
        background: Some(c.surface.scale_alpha(0.60).into()),
        border: Border {
            color: c.line,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
pub fn catalog_placeholder(theme: &Theme) -> container::Style {
    let c = colors(theme);
    container::Style {
        background: Some(c.hover.into()),
        ..Default::default()
    }
}

pub fn toggle(
    theme: &Theme,
    status: iced::widget::toggler::Status,
) -> iced::widget::toggler::Style {
    use iced::widget::toggler::{self, Status};
    let c = colors(theme);
    let mut style = toggler::default(theme, status);
    let active = matches!(
        status,
        Status::Active { is_toggled: true } | Status::Hovered { is_toggled: true }
    );
    style.background = if active { c.solid } else { c.line }.into();
    style.foreground = if active { c.on_solid } else { c.text }.into();
    style
}

pub fn background_choice(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = quiet(theme, status);
        style.border = outline(if selected {
            colors(theme).focus
        } else {
            colors(theme).line
        });
        style.border.width = if selected { 2.0 } else { 1.0 };
        style
    }
}
