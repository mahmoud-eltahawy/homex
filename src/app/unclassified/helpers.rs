//! Pure client-side helpers for the unclassified feature.

use leptos::prelude::*;

use crate::app::model::MediaKind;
use crate::app::types::FileExt;

use super::types::UnclassifiedEntry;

pub fn kind_for_ext(ext: Option<&FileExt>) -> Option<MediaKind> {
    MediaKind::from_extension(ext?)
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

pub fn encode_query_path(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

pub fn select_all(selection: RwSignal<Vec<UnclassifiedEntry>>, entries: Vec<UnclassifiedEntry>) {
    selection.update(|v| {
        let target_kind = v
            .first()
            .and_then(|e| kind_for_ext(e.ext.as_ref()))
            .or_else(|| entries.iter().find_map(|e| kind_for_ext(e.ext.as_ref())));
        v.clear();
        for e in entries {
            let k = kind_for_ext(e.ext.as_ref());
            if k.is_none() || (target_kind.is_some() && k != target_kind) {
                continue;
            }
            v.push(e);
        }
    });
}

pub fn toggle_selection(selection: RwSignal<Vec<UnclassifiedEntry>>, entry: UnclassifiedEntry) {
    selection.update(|v| {
        if let Some(pos) = v.iter().position(|e| e.rel_path == entry.rel_path) {
            v.remove(pos);
            return;
        }
        let new_kind = kind_for_ext(entry.ext.as_ref());
        let existing_kind = v.first().and_then(|e| kind_for_ext(e.ext.as_ref()));
        if existing_kind.is_some() && existing_kind != new_kind {
            v.clear();
        }
        v.push(entry);
    });
}

pub fn toggle_folder_selection(
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    entry: UnclassifiedEntry,
) {
    selection.update(|v| {
        if let Some(pos) = v.iter().position(|e| e.rel_path == entry.rel_path) {
            v.remove(pos);
        } else {
            v.push(entry);
        }
    });
}

pub fn parse_season(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() { None } else { s.parse().ok() }
}

pub fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
