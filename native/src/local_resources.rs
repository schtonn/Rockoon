use crate::workspace::{self, Entry, Kind};
use std::{collections::BTreeMap, path::PathBuf};

pub struct SkyGroup<'a> {
    pub key: PathBuf,
    pub name: String,
    pub faces: Vec<&'a Entry>,
}
impl SkyGroup<'_> {
    pub fn preview(&self) -> &Entry {
        self.faces[0]
    }
    pub fn size(&self) -> u64 {
        self.faces.iter().map(|e| e.size).sum()
    }
}
pub enum LocalRow<'a> {
    File(&'a Entry),
    Sky(SkyGroup<'a>),
}
fn sky_face(entry: &Entry) -> Option<(String, usize)> {
    if entry.kind != Kind::Skies || entry.directory {
        return None;
    }
    if !entry.path.extension()?.eq_ignore_ascii_case("bmp") {
        return None;
    }
    let stem = entry.path.file_stem()?.to_str()?;
    let (prefix, side) = stem.rsplit_once('_')?;
    if !prefix.to_ascii_lowercase().starts_with("sky_") || prefix.len() <= 4 {
        return None;
    }
    let order = ["Front", "Back", "Left", "Right", "Down"]
        .iter()
        .position(|s| side.eq_ignore_ascii_case(s))?;
    Some((prefix.into(), order))
}
pub fn is_sky_face(entry: &Entry) -> bool {
    sky_face(entry).is_some()
}
/// Group only recognized faces from the same folder, preserving other files and list order.
pub fn rows<'a>(entries: &'a [Entry], search: &str) -> Vec<LocalRow<'a>> {
    let mut rows = Vec::new();
    let mut groups = BTreeMap::new();
    for entry in entries {
        if let Some((prefix, _)) = sky_face(entry) {
            let parent = entry.path.parent().unwrap_or(&entry.path);
            let key = parent.join(prefix.to_ascii_lowercase());
            if let Some(&index) = groups.get(&key) {
                if let LocalRow::Sky(group) = &mut rows[index] {
                    group.faces.push(entry);
                }
            } else {
                groups.insert(key.clone(), rows.len());
                rows.push(LocalRow::Sky(SkyGroup {
                    key,
                    name: entry
                        .name
                        .rsplit_once('_')
                        .map(|(name, _)| name)
                        .unwrap_or(&prefix)
                        .into(),
                    faces: vec![entry],
                }));
            }
        } else {
            rows.push(LocalRow::File(entry));
        }
    }
    for row in &mut rows {
        if let LocalRow::Sky(group) = row {
            group
                .faces
                .sort_by_key(|entry| sky_face(entry).map(|(_, order)| order));
        }
    }
    rows.retain(|row| match row {
        LocalRow::File(entry) => workspace::matches(&entry.name, search),
        LocalRow::Sky(group) => {
            workspace::matches(&group.name, search)
                || group
                    .faces
                    .iter()
                    .any(|e| workspace::matches(&e.name, search))
        }
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    fn face(path: &str) -> Entry {
        Entry {
            kind: Kind::Skies,
            path: path.into(),
            name: path.into(),
            directory: false,
            size: 10,
            disabled: false,
        }
    }
    #[test]
    fn groups_known_faces_case_insensitively_without_mixing_directories() {
        let entries = vec![
            face("one/Sky_A_Left.bmp"),
            face("one/sky_a_FRONT.BMP"),
            face("two/Sky_A_Front.bmp"),
            face("one/unrelated.bmp"),
        ];
        let rows = rows(&entries, "");
        assert_eq!(rows.len(), 3);
        let LocalRow::Sky(group) = &rows[0] else {
            panic!("expected sky group")
        };
        assert_eq!(group.faces.len(), 2);
        assert!(group.preview().path.ends_with("sky_a_FRONT.BMP"));
        assert_eq!(group.size(), 20);
        assert!(matches!(&rows[2], LocalRow::File(_)));
    }
    #[test]
    fn searching_a_face_keeps_the_whole_group() {
        let entries = vec![
            face("Sky_A_Front.bmp"),
            face("Sky_A_Down.bmp"),
            face("Sky_B_Front.bmp"),
        ];
        let rows = rows(&entries, "A Down");
        assert_eq!(rows.len(), 1);
        let LocalRow::Sky(group) = &rows[0] else {
            panic!("expected sky group")
        };
        assert_eq!(group.faces.len(), 2);
    }
}
