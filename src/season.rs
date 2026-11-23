//! Season facts from data/season.yaml: tribes per lobby, tribes rotated out of the pool and tiers changed
//! by server-side hotfixes that HearthstoneJSON's card file does not show.

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
}

fn five() -> usize {
    5
}

impl Default for Season {
    fn default() -> Self {
        Season { lobby_tribes: 5, rotated_out: vec!["NAGA".into()], tier_fixes: HashMap::new() }
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
}
