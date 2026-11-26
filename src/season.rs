//! Season facts from data/season.yaml: tribes per lobby, tribes rotated out of the pool, and card tiers
//! and texts changed by server-side hotfixes that HearthstoneJSON's card file does not show.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use crate::data;

#[derive(Debug, Clone, Deserialize)]
pub struct Season {
    #[serde(default = "five")]
    pub lobby_tribes: usize,
    #[serde(default)]
    pub rotated_out: Vec<String>,
    /// Card id (normal version) -> real tavern tier, for tier changes made by a server-side hotfix.
    #[serde(default)]
    pub tier_fixes: HashMap<String, i64>,
    /// Exact card id (golden versions listed apart) -> real card text, for hotfix text changes.
    #[serde(default)]
    pub text_fixes: HashMap<String, String>,
    /// Card pictures saved before this day ("YYYY-MM-DD") are downloaded again once (new renders after a
    /// hotfix or patch). Empty = keep every saved picture.
    #[serde(default)]
    pub art_since: String,
}

fn five() -> usize {
    5
}

impl Default for Season {
    fn default() -> Self {
        Season { lobby_tribes: 5, rotated_out: vec!["NAGA".into()], tier_fixes: HashMap::new(), text_fixes: HashMap::new(), art_since: String::new() }
    }
}

/// The season facts (reloaded when data/season.yaml changes).
pub fn get() -> Arc<Season> {
    data::season()
}

impl Season {
    /// The real tier of a card: the hotfix tier when there is one (also for the golden "_G" version).
    pub fn fixed_tier(&self, card_id: &str, tier: Option<i64>) -> Option<i64> {
        let base = card_id.strip_suffix("_G").unwrap_or(card_id);
        self.tier_fixes.get(base).copied().or(tier)
    }

    /// The real text of a card: the hotfix text when there is one.
    pub fn fixed_text(&self, card_id: &str, text: String) -> String {
        self.text_fixes.get(card_id).cloned().unwrap_or(text)
    }

    /// Is a picture saved at `saved` older than `art_since` (so worth downloading again)?
    pub fn art_outdated(&self, saved: std::time::SystemTime) -> bool {
        let Ok(day) = chrono::NaiveDate::parse_from_str(self.art_since.trim(), "%Y-%m-%d") else { return false };
        let since = day.and_hms_opt(0, 0, 0).map(|t| t.and_utc().timestamp()).unwrap_or(i64::MAX);
        let saved = saved.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        saved < since
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_fixes_cover_golden_cards() {
        let s: Season = serde_yaml_ng::from_str("tier_fixes: {BG36_206: 5}").unwrap();
        assert_eq!(s.fixed_tier("BG36_206", Some(4)), Some(5));
        assert_eq!(s.fixed_tier("BG36_206_G", Some(4)), Some(5));
        assert_eq!(s.fixed_tier("BG28_633", Some(5)), Some(5));
        assert_eq!(Season::default().fixed_tier("BG36_206", Some(4)), Some(4));
    }

    #[test]
    fn text_fixes_and_art_date() {
        let s: Season = serde_yaml_ng::from_str("text_fixes: {BG28_633: new}\nart_since: 2026-11-18").unwrap();
        assert_eq!(s.fixed_text("BG28_633", "old".into()), "new");
        assert_eq!(s.fixed_text("BG28_633_G", "old".into()), "old");
        let day = |secs: u64| std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
        assert!(s.art_outdated(day(1_763_337_600))); // 2025-11-17
        assert!(!s.art_outdated(day(1_795_046_400))); // 2026-11-19
        assert!(!Season::default().art_outdated(day(0)));
    }
}
