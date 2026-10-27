#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub id: i32,
    pub name: &'static str,
}
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}
pub const ALL: &[Key] = &[
    Key { id: 0, name: "1" },
    Key { id: 1, name: "2" },
    Key { id: 2, name: "3" },
    Key { id: 3, name: "4" },
    Key { id: 4, name: "5" },
    Key { id: 5, name: "6" },
    Key { id: 6, name: "7" },
    Key { id: 7, name: "8" },
    Key { id: 8, name: "9" },
    Key { id: 9, name: "0" },
    Key { id: 10, name: "-" },
    Key { id: 11, name: "=" },
    Key {
        id: 12,
        name: "BackSpace",
    },
    Key {
        id: 13,
        name: "Tab",
    },
    Key { id: 14, name: "Q" },
    Key { id: 15, name: "W" },
    Key { id: 16, name: "E" },
    Key { id: 17, name: "R" },
    Key { id: 18, name: "T" },
    Key { id: 19, name: "Y" },
    Key { id: 20, name: "U" },
    Key { id: 21, name: "I" },
    Key { id: 22, name: "O" },
    Key { id: 23, name: "P" },
    Key { id: 24, name: "[" },
    Key { id: 25, name: "]" },
    Key {
        id: 26,
        name: "Ctrl",
    },
    Key { id: 27, name: "A" },
    Key { id: 28, name: "S" },
    Key { id: 29, name: "D" },
    Key { id: 30, name: "F" },
    Key { id: 31, name: "G" },
    Key { id: 32, name: "H" },
    Key { id: 33, name: "J" },
    Key { id: 34, name: "K" },
    Key { id: 35, name: "L" },
    Key { id: 36, name: ";" },
    Key { id: 37, name: "'" },
    Key { id: 38, name: "`" },
    Key {
        id: 39,
        name: "Shift",
    },
    Key {
        id: 40,
        name: "\\\\",
    },
    Key { id: 41, name: "Z" },
    Key { id: 42, name: "X" },
    Key { id: 43, name: "C" },
    Key { id: 44, name: "V" },
    Key { id: 45, name: "B" },
    Key { id: 46, name: "N" },
    Key { id: 47, name: "M" },
    Key { id: 48, name: "," },
    Key { id: 49, name: "." },
    Key { id: 50, name: "/" },
    Key {
        id: 51,
        name: "Right Shift",
    },
    Key {
        id: 52,
        name: "Alt",
    },
    Key {
        id: 53,
        name: "Space",
    },
    Key {
        id: 54,
        name: "Num 7",
    },
    Key {
        id: 55,
        name: "Num 8",
    },
    Key {
        id: 56,
        name: "Num 9",
    },
    Key {
        id: 57,
        name: "Num -",
    },
    Key {
        id: 58,
        name: "Num 4",
    },
    Key {
        id: 59,
        name: "Num 5",
    },
    Key {
        id: 60,
        name: "Num 6",
    },
    Key {
        id: 61,
        name: "Num +",
    },
    Key {
        id: 62,
        name: "Num 1",
    },
    Key {
        id: 63,
        name: "Num 2",
    },
    Key {
        id: 64,
        name: "Num 3",
    },
    Key {
        id: 65,
        name: "Num 0",
    },
    Key {
        id: 66,
        name: "Num Del",
    },
    Key { id: 68, name: "Up" },
    Key {
        id: 69,
        name: "Down",
    },
    Key {
        id: 70,
        name: "Left",
    },
    Key {
        id: 71,
        name: "Right",
    },
];
