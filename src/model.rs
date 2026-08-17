use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Representation {
    pub item: u32,
    pub format: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub id: Uuid,
    pub origin: Uuid,
    pub created_millis: u64,
    pub representations: Vec<Representation>,
}

impl Clip {
    #[must_use]
    pub fn new(origin: Uuid, representations: Vec<Representation>) -> Self {
        Self {
            id: Uuid::new_v4(),
            origin,
            created_millis: now_millis(),
            representations,
        }
    }

    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.representations
            .iter()
            .map(|representation| u64::try_from(representation.data.len()).unwrap_or(u64::MAX))
            .sum()
    }

    #[must_use]
    pub fn preview(&self, limit: usize) -> String {
        let text = self
            .representations
            .iter()
            .find(|representation| is_plain_text(&representation.format))
            .map_or_else(
                || {
                    let formats = self
                        .representations
                        .iter()
                        .map(|representation| representation.format.as_str())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<{formats}>")
                },
                |representation| String::from_utf8_lossy(&representation.data).into_owned(),
            );
        truncate_clean(&text, limit)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepresentationInfo {
    pub item: u32,
    pub format: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonitorEvent {
    pub timestamp_millis: u64,
    pub direction: Direction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    pub clip_id: Uuid,
    pub origin: Uuid,
    pub preview: String,
    pub representations: Vec<RepresentationInfo>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Local,
    Send,
    Receive,
}

impl MonitorEvent {
    #[must_use]
    pub fn from_clip(direction: Direction, peer: Option<String>, clip: &Clip) -> Self {
        Self {
            timestamp_millis: now_millis(),
            direction,
            peer,
            clip_id: clip.id,
            origin: clip.origin,
            preview: clip.preview(400),
            representations: clip
                .representations
                .iter()
                .map(|representation| RepresentationInfo {
                    item: representation.item,
                    format: representation.format.clone(),
                    bytes: u64::try_from(representation.data.len()).unwrap_or(u64::MAX),
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.representations
            .iter()
            .map(|representation| representation.bytes)
            .sum()
    }
}

#[must_use]
pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

#[must_use]
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn is_plain_text(format: &str) -> bool {
    matches!(
        format.to_ascii_lowercase().as_str(),
        "text/plain" | "text/plain;charset=utf-8" | "public.utf8-plain-text" | "utf8_string"
    )
}

fn truncate_clean(text: &str, limit: usize) -> String {
    let cleaned = text
        .chars()
        .filter_map(|character| match character {
            '\n' | '\r' | '\t' => Some(' '),
            character if character.is_control() => None,
            character => Some(character),
        })
        .collect::<String>();
    if cleaned.chars().count() <= limit {
        return cleaned;
    }
    let mut shortened = cleaned.chars().take(limit.saturating_sub(1)).collect::<String>();
    shortened.push('…');
    shortened
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_is_unicode_safe_and_strips_terminal_controls() {
        let clip = Clip::new(
            Uuid::nil(),
            vec![Representation {
                item: 0,
                format: "text/plain".into(),
                data: "hello\n\u{1b}[31m世界".as_bytes().to_vec(),
            }],
        );
        assert_eq!(clip.preview(10), "hello [31…");
    }

    #[test]
    fn formats_have_a_useful_fallback_preview() {
        let clip = Clip::new(
            Uuid::nil(),
            vec![Representation {
                item: 0,
                format: "image/heic".into(),
                data: vec![1, 2, 3],
            }],
        );
        assert_eq!(clip.preview(80), "<image/heic>");
    }
}
