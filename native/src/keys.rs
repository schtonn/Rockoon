//! Ballance table indices and CKKEYBOARD/DirectInput scan codes are separate formats.
use iced::keyboard::key::{Code, Physical};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Ballance,
    ScanCode,
}
struct Binding {
    code: Code,
    game: Option<i32>,
    scan: i32,
    name: &'static str,
}
const BINDINGS: &[Binding] = &[
    Binding {
        code: Code::Digit1,
        game: Some(0),
        scan: 2,
        name: "1",
    },
    Binding {
        code: Code::Digit2,
        game: Some(1),
        scan: 3,
        name: "2",
    },
    Binding {
        code: Code::Digit3,
        game: Some(2),
        scan: 4,
        name: "3",
    },
    Binding {
        code: Code::Digit4,
        game: Some(3),
        scan: 5,
        name: "4",
    },
    Binding {
        code: Code::Digit5,
        game: Some(4),
        scan: 6,
        name: "5",
    },
    Binding {
        code: Code::Digit6,
        game: Some(5),
        scan: 7,
        name: "6",
    },
    Binding {
        code: Code::Digit7,
        game: Some(6),
        scan: 8,
        name: "7",
    },
    Binding {
        code: Code::Digit8,
        game: Some(7),
        scan: 9,
        name: "8",
    },
    Binding {
        code: Code::Digit9,
        game: Some(8),
        scan: 10,
        name: "9",
    },
    Binding {
        code: Code::Digit0,
        game: Some(9),
        scan: 11,
        name: "0",
    },
    Binding {
        code: Code::KeyQ,
        game: Some(14),
        scan: 16,
        name: "Q",
    },
    Binding {
        code: Code::KeyW,
        game: Some(15),
        scan: 17,
        name: "W",
    },
    Binding {
        code: Code::KeyE,
        game: Some(16),
        scan: 18,
        name: "E",
    },
    Binding {
        code: Code::KeyR,
        game: Some(17),
        scan: 19,
        name: "R",
    },
    Binding {
        code: Code::KeyT,
        game: Some(18),
        scan: 20,
        name: "T",
    },
    Binding {
        code: Code::KeyY,
        game: Some(19),
        scan: 21,
        name: "Y",
    },
    Binding {
        code: Code::KeyU,
        game: Some(20),
        scan: 22,
        name: "U",
    },
    Binding {
        code: Code::KeyI,
        game: Some(21),
        scan: 23,
        name: "I",
    },
    Binding {
        code: Code::KeyO,
        game: Some(22),
        scan: 24,
        name: "O",
    },
    Binding {
        code: Code::KeyP,
        game: Some(23),
        scan: 25,
        name: "P",
    },
    Binding {
        code: Code::KeyA,
        game: Some(27),
        scan: 30,
        name: "A",
    },
    Binding {
        code: Code::KeyS,
        game: Some(28),
        scan: 31,
        name: "S",
    },
    Binding {
        code: Code::KeyD,
        game: Some(29),
        scan: 32,
        name: "D",
    },
    Binding {
        code: Code::KeyF,
        game: Some(30),
        scan: 33,
        name: "F",
    },
    Binding {
        code: Code::KeyG,
        game: Some(31),
        scan: 34,
        name: "G",
    },
    Binding {
        code: Code::KeyH,
        game: Some(32),
        scan: 35,
        name: "H",
    },
    Binding {
        code: Code::KeyJ,
        game: Some(33),
        scan: 36,
        name: "J",
    },
    Binding {
        code: Code::KeyK,
        game: Some(34),
        scan: 37,
        name: "K",
    },
    Binding {
        code: Code::KeyL,
        game: Some(35),
        scan: 38,
        name: "L",
    },
    Binding {
        code: Code::KeyZ,
        game: Some(41),
        scan: 44,
        name: "Z",
    },
    Binding {
        code: Code::KeyX,
        game: Some(42),
        scan: 45,
        name: "X",
    },
    Binding {
        code: Code::KeyC,
        game: Some(43),
        scan: 46,
        name: "C",
    },
    Binding {
        code: Code::KeyV,
        game: Some(44),
        scan: 47,
        name: "V",
    },
    Binding {
        code: Code::KeyB,
        game: Some(45),
        scan: 48,
        name: "B",
    },
    Binding {
        code: Code::KeyN,
        game: Some(46),
        scan: 49,
        name: "N",
    },
    Binding {
        code: Code::KeyM,
        game: Some(47),
        scan: 50,
        name: "M",
    },
    Binding {
        code: Code::Escape,
        game: None,
        scan: 1,
        name: "Esc",
    },
    Binding {
        code: Code::Minus,
        game: Some(10),
        scan: 12,
        name: "-",
    },
    Binding {
        code: Code::Equal,
        game: Some(11),
        scan: 13,
        name: "=",
    },
    Binding {
        code: Code::Backspace,
        game: Some(12),
        scan: 14,
        name: "退格",
    },
    Binding {
        code: Code::Tab,
        game: Some(13),
        scan: 15,
        name: "Tab",
    },
    Binding {
        code: Code::BracketLeft,
        game: Some(24),
        scan: 26,
        name: "[",
    },
    Binding {
        code: Code::BracketRight,
        game: Some(25),
        scan: 27,
        name: "]",
    },
    Binding {
        code: Code::Enter,
        game: None,
        scan: 28,
        name: "Enter",
    },
    Binding {
        code: Code::ControlLeft,
        game: Some(26),
        scan: 29,
        name: "左 Ctrl",
    },
    Binding {
        code: Code::ControlRight,
        game: None,
        scan: 157,
        name: "右 Ctrl",
    },
    Binding {
        code: Code::Semicolon,
        game: Some(36),
        scan: 39,
        name: ";",
    },
    Binding {
        code: Code::Quote,
        game: Some(37),
        scan: 40,
        name: "'",
    },
    Binding {
        code: Code::Backquote,
        game: Some(38),
        scan: 41,
        name: "`",
    },
    Binding {
        code: Code::ShiftLeft,
        game: Some(39),
        scan: 42,
        name: "左 Shift",
    },
    Binding {
        code: Code::ShiftRight,
        game: Some(51),
        scan: 54,
        name: "右 Shift",
    },
    Binding {
        code: Code::Backslash,
        game: Some(40),
        scan: 43,
        name: "\\",
    },
    Binding {
        code: Code::Comma,
        game: Some(48),
        scan: 51,
        name: ",",
    },
    Binding {
        code: Code::Period,
        game: Some(49),
        scan: 52,
        name: ".",
    },
    Binding {
        code: Code::Slash,
        game: Some(50),
        scan: 53,
        name: "/",
    },
    Binding {
        code: Code::AltLeft,
        game: Some(52),
        scan: 56,
        name: "左 Alt",
    },
    Binding {
        code: Code::AltRight,
        game: None,
        scan: 184,
        name: "右 Alt",
    },
    Binding {
        code: Code::Space,
        game: Some(53),
        scan: 57,
        name: "空格",
    },
    Binding {
        code: Code::CapsLock,
        game: None,
        scan: 58,
        name: "Caps Lock",
    },
    Binding {
        code: Code::NumLock,
        game: None,
        scan: 69,
        name: "Num Lock",
    },
    Binding {
        code: Code::ScrollLock,
        game: None,
        scan: 70,
        name: "Scroll Lock",
    },
    Binding {
        code: Code::NumpadSubtract,
        game: Some(57),
        scan: 74,
        name: "小键盘 -",
    },
    Binding {
        code: Code::NumpadAdd,
        game: Some(61),
        scan: 78,
        name: "小键盘 +",
    },
    Binding {
        code: Code::NumpadDecimal,
        game: Some(66),
        scan: 83,
        name: "小键盘 .",
    },
    Binding {
        code: Code::NumpadMultiply,
        game: None,
        scan: 55,
        name: "小键盘 *",
    },
    Binding {
        code: Code::NumpadDivide,
        game: None,
        scan: 181,
        name: "小键盘 /",
    },
    Binding {
        code: Code::NumpadEnter,
        game: None,
        scan: 156,
        name: "小键盘 Enter",
    },
    Binding {
        code: Code::Home,
        game: None,
        scan: 199,
        name: "Home",
    },
    Binding {
        code: Code::End,
        game: None,
        scan: 207,
        name: "End",
    },
    Binding {
        code: Code::PageUp,
        game: None,
        scan: 201,
        name: "Page Up",
    },
    Binding {
        code: Code::PageDown,
        game: None,
        scan: 209,
        name: "Page Down",
    },
    Binding {
        code: Code::ArrowUp,
        game: Some(68),
        scan: 200,
        name: "↑",
    },
    Binding {
        code: Code::ArrowDown,
        game: Some(69),
        scan: 208,
        name: "↓",
    },
    Binding {
        code: Code::ArrowLeft,
        game: Some(70),
        scan: 203,
        name: "←",
    },
    Binding {
        code: Code::ArrowRight,
        game: Some(71),
        scan: 205,
        name: "→",
    },
    Binding {
        code: Code::Insert,
        game: None,
        scan: 210,
        name: "Insert",
    },
    Binding {
        code: Code::Delete,
        game: None,
        scan: 211,
        name: "Delete",
    },
    Binding {
        code: Code::SuperLeft,
        game: None,
        scan: 219,
        name: "左 Win",
    },
    Binding {
        code: Code::SuperRight,
        game: None,
        scan: 220,
        name: "右 Win",
    },
    Binding {
        code: Code::ContextMenu,
        game: None,
        scan: 221,
        name: "菜单键",
    },
    Binding {
        code: Code::PrintScreen,
        game: None,
        scan: 183,
        name: "Print Screen",
    },
    Binding {
        code: Code::IntlYen,
        game: None,
        scan: 125,
        name: "¥",
    },
    Binding {
        code: Code::Convert,
        game: None,
        scan: 121,
        name: "转换",
    },
    Binding {
        code: Code::NonConvert,
        game: None,
        scan: 123,
        name: "不转换",
    },
    Binding {
        code: Code::KanaMode,
        game: None,
        scan: 112,
        name: "假名",
    },
    Binding {
        code: Code::NumpadEqual,
        game: None,
        scan: 141,
        name: "小键盘 =",
    },
    Binding {
        code: Code::NumpadComma,
        game: None,
        scan: 179,
        name: "小键盘 ,",
    },
    Binding {
        code: Code::Numpad0,
        game: Some(65),
        scan: 82,
        name: "小键盘 0",
    },
    Binding {
        code: Code::Numpad1,
        game: Some(62),
        scan: 79,
        name: "小键盘 1",
    },
    Binding {
        code: Code::Numpad2,
        game: Some(63),
        scan: 80,
        name: "小键盘 2",
    },
    Binding {
        code: Code::Numpad3,
        game: Some(64),
        scan: 81,
        name: "小键盘 3",
    },
    Binding {
        code: Code::Numpad4,
        game: Some(58),
        scan: 75,
        name: "小键盘 4",
    },
    Binding {
        code: Code::Numpad5,
        game: Some(59),
        scan: 76,
        name: "小键盘 5",
    },
    Binding {
        code: Code::Numpad6,
        game: Some(60),
        scan: 77,
        name: "小键盘 6",
    },
    Binding {
        code: Code::Numpad7,
        game: Some(54),
        scan: 71,
        name: "小键盘 7",
    },
    Binding {
        code: Code::Numpad8,
        game: Some(55),
        scan: 72,
        name: "小键盘 8",
    },
    Binding {
        code: Code::Numpad9,
        game: Some(56),
        scan: 73,
        name: "小键盘 9",
    },
    Binding {
        code: Code::F1,
        game: None,
        scan: 59,
        name: "F1",
    },
    Binding {
        code: Code::F2,
        game: None,
        scan: 60,
        name: "F2",
    },
    Binding {
        code: Code::F3,
        game: None,
        scan: 61,
        name: "F3",
    },
    Binding {
        code: Code::F4,
        game: None,
        scan: 62,
        name: "F4",
    },
    Binding {
        code: Code::F5,
        game: None,
        scan: 63,
        name: "F5",
    },
    Binding {
        code: Code::F6,
        game: None,
        scan: 64,
        name: "F6",
    },
    Binding {
        code: Code::F7,
        game: None,
        scan: 65,
        name: "F7",
    },
    Binding {
        code: Code::F8,
        game: None,
        scan: 66,
        name: "F8",
    },
    Binding {
        code: Code::F9,
        game: None,
        scan: 67,
        name: "F9",
    },
    Binding {
        code: Code::F10,
        game: None,
        scan: 68,
        name: "F10",
    },
    Binding {
        code: Code::F11,
        game: None,
        scan: 87,
        name: "F11",
    },
    Binding {
        code: Code::F12,
        game: None,
        scan: 88,
        name: "F12",
    },
    Binding {
        code: Code::F13,
        game: None,
        scan: 100,
        name: "F13",
    },
    Binding {
        code: Code::F14,
        game: None,
        scan: 101,
        name: "F14",
    },
    Binding {
        code: Code::F15,
        game: None,
        scan: 102,
        name: "F15",
    },
    Binding {
        code: Code::IntlBackslash,
        game: Some(67),
        scan: 86,
        name: "<",
    },
];
pub fn encode(encoding: Encoding, physical: Physical) -> Option<i32> {
    let Physical::Code(code) = physical else {
        return None;
    };
    let binding = BINDINGS.iter().find(|b| b.code == code)?;
    match encoding {
        Encoding::Ballance => binding.game,
        Encoding::ScanCode => Some(binding.scan),
    }
}
pub fn name(encoding: Encoding, value: &str) -> String {
    let Ok(id) = value.parse::<i32>() else {
        return "未识别按键".into();
    };
    if encoding == Encoding::ScanCode && id == 0 {
        return "未设置".into();
    }
    BINDINGS
        .iter()
        .find(|b| match encoding {
            Encoding::Ballance => b.game == Some(id),
            Encoding::ScanCode => b.scan == id,
        })
        .map(|b| b.name)
        .unwrap_or("未识别按键")
        .into()
}

use crate::app::Message;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{self, Tree},
};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};
pub fn recorder(content: Element<'_, Message>, active: bool) -> Element<'_, Message> {
    Element::new(Recorder(content, active))
}
struct Recorder<'a>(Element<'a, Message>, bool);
impl Widget<Message, Theme, Renderer> for Recorder<'_> {
    fn size(&self) -> Size<Length> {
        self.0.as_widget().size()
    }
    fn size_hint(&self) -> Size<Length> {
        self.0.as_widget().size_hint()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.0)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.0));
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.0
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.0.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.1 && matches!(event, Event::Keyboard(_)) {
            if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key,
                physical_key,
                repeat,
                ..
            }) = event
                && !repeat
            {
                if *key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                    shell.publish(Message::CancelKeyCapture);
                } else {
                    shell.publish(Message::RecordedKey(*physical_key));
                }
            }
            shell.capture_event();
            return;
        }
        self.0.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.0
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.0
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.0.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_indices_and_scan_codes_do_not_cross_formats() {
        for (key, game, scan) in [
            (Code::Space, 53, 57),
            (Code::KeyW, 15, 17),
            (Code::ArrowUp, 68, 200),
            (Code::ShiftRight, 51, 54),
            (Code::Numpad8, 55, 72),
            (Code::Backslash, 40, 43),
        ] {
            assert_eq!(encode(Encoding::Ballance, Physical::Code(key)), Some(game));
            assert_eq!(encode(Encoding::ScanCode, Physical::Code(key)), Some(scan));
            assert_eq!(
                name(Encoding::Ballance, &game.to_string()),
                name(Encoding::ScanCode, &scan.to_string())
            );
        }
        assert_eq!(encode(Encoding::Ballance, Physical::Code(Code::F1)), None);
        assert_eq!(
            encode(Encoding::ScanCode, Physical::Code(Code::F1)),
            Some(59)
        );
        assert_eq!(
            encode(Encoding::ScanCode, Physical::Code(Code::ControlRight)),
            Some(157)
        );
        assert_eq!(name(Encoding::ScanCode, "0"), "未设置");
    }
    #[test]
    fn complete_game_table_and_physical_keys_have_no_duplicate_entries() {
        let ids: std::collections::HashSet<_> = BINDINGS.iter().filter_map(|b| b.game).collect();
        assert_eq!(ids, (0..72).collect());
        let codes: std::collections::HashSet<_> = BINDINGS.iter().map(|b| b.code).collect();
        let scans: std::collections::HashSet<_> = BINDINGS.iter().map(|b| b.scan).collect();
        assert_eq!(codes.len(), BINDINGS.len());
        assert_eq!(scans.len(), BINDINGS.len());
    }
}
