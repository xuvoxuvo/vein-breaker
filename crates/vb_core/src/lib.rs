//! Pure gameplay rules. No Bevy, no rendering, no IO.
//!
//! Everything here must be testable in milliseconds so the game crate can stay thin.

/// What a mine block is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrickKind {
    Rock,
    Copper,
    Iron,
    Bedrock,
}

impl BrickKind {
    /// Hit points before the block breaks.
    pub const fn max_hp(self) -> i32 {
        match self {
            Self::Rock => 2,
            Self::Copper => 2,
            Self::Iron => 3,
            Self::Bedrock => i32::MAX,
        }
    }

    /// Ore awarded when the block breaks.
    pub const fn ore_value(self) -> u32 {
        match self {
            Self::Rock => 1,
            Self::Copper => 2,
            Self::Iron => 3,
            Self::Bedrock => 0,
        }
    }

    pub const fn destructible(self) -> bool {
        !matches!(self, Self::Bedrock)
    }

    /// Level file legend.
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            '.' => None,
            '#' => Some(Self::Bedrock),
            'C' | 'c' => Some(Self::Copper),
            'I' | 'i' => Some(Self::Iron),
            'R' | 'r' => Some(Self::Rock),
            _ => None,
        }
    }
}

/// Clamped damage application. Returns the remaining HP (never negative).
pub fn apply_damage(hp: i32, damage: i32) -> i32 {
    hp.saturating_sub(damage.max(0)).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_is_clamped_at_zero() {
        assert_eq!(apply_damage(2, 1), 1);
        assert_eq!(apply_damage(2, 5), 0);
        assert_eq!(apply_damage(3, 0), 3);
        assert_eq!(apply_damage(3, -4), 3);
    }

    #[test]
    fn bedrock_never_breaks() {
        assert_eq!(apply_damage(BrickKind::Bedrock.max_hp(), i32::MAX), 0);
        assert!(!BrickKind::Bedrock.destructible());
        assert_eq!(BrickKind::Bedrock.ore_value(), 0);
    }

    #[test]
    fn legend_maps_to_kinds() {
        assert_eq!(BrickKind::from_char('C'), Some(BrickKind::Copper));
        assert_eq!(BrickKind::from_char('#'), Some(BrickKind::Bedrock));
        assert_eq!(BrickKind::from_char('.'), None);
        assert_eq!(BrickKind::from_char('?'), None);
    }

    #[test]
    fn richer_ore_is_tougher() {
        assert!(BrickKind::Iron.max_hp() >= BrickKind::Copper.max_hp());
        assert!(BrickKind::Iron.ore_value() > BrickKind::Rock.ore_value());
    }
}
