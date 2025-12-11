//! Game region and version detection.

use std::fmt;

/// Target platform for the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Nintendo GameCube
    GameCube,
    /// Sega Dreamcast (not yet supported)
    Dreamcast,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::GameCube => write!(f, "GameCube"),
            Platform::Dreamcast => write!(f, "Dreamcast"),
        }
    }
}

/// Game region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// US/NTSC-U
    Us,
    /// Japan/NTSC-J
    Jp,
    /// Europe/PAL
    Eu,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Region::Us => write!(f, "US"),
            Region::Jp => write!(f, "JP"),
            Region::Eu => write!(f, "EU"),
        }
    }
}

/// Specific game version identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameVersion {
    pub platform: Platform,
    pub region: Region,
    pub product_id: String,
    pub build_date: Option<String>,
}

impl GameVersion {
    /// Create a new game version.
    pub fn new(platform: Platform, region: Region, product_id: String) -> Self {
        Self {
            platform,
            region,
            product_id,
            build_date: None,
        }
    }

    /// Detect the game version from a game ID string.
    ///
    /// GameCube game IDs are 6 characters:
    /// - Bytes 0-3: Game code (e.g., "GEAE" for Skies of Arcadia Legends US)
    /// - Byte 4: Region code (E=US, J=JP, P=EU)
    /// - Byte 5: Maker code (usually '8' for Sega, 'P' for ?)
    pub fn from_game_id(game_id: &str) -> Option<Self> {
        if game_id.len() < 6 {
            return None;
        }

        let game_code = &game_id[0..3];
        let region_code = game_id.chars().nth(3)?;

        // Check if this is Skies of Arcadia / Eternal Arcadia
        if game_code != "GEA" {
            return None;
        }

        let region = match region_code {
            'E' => Region::Us,
            'J' => Region::Jp,
            'P' => Region::Eu,
            _ => return None,
        };

        Some(Self::new(Platform::GameCube, region, game_id.to_string()))
    }

    /// Get a version key for offset lookups.
    pub fn version_key(&self) -> String {
        format!("GC-{}-GEA", self.region)
    }

    /// Check if this is the US GameCube version.
    pub fn is_gc_us(&self) -> bool {
        self.platform == Platform::GameCube && self.region == Region::Us
    }

    /// Check if this is the JP GameCube version.
    pub fn is_gc_jp(&self) -> bool {
        self.platform == Platform::GameCube && self.region == Region::Jp
    }

    /// Check if this is the EU GameCube version.
    pub fn is_gc_eu(&self) -> bool {
        self.platform == Platform::GameCube && self.region == Region::Eu
    }

    /// Check if this is a GameCube version (any region).
    pub fn is_gc(&self) -> bool {
        self.platform == Platform::GameCube
    }

    /// Check if this is an EU version (any platform).
    pub fn is_eu(&self) -> bool {
        self.region == Region::Eu
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{} ({})", self.platform, self.region, self.product_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gc_us() {
        let version = GameVersion::from_game_id("GEAE8P").unwrap();
        assert_eq!(version.platform, Platform::GameCube);
        assert_eq!(version.region, Region::Us);
        assert!(version.is_gc_us());
    }

    #[test]
    fn test_detect_gc_jp() {
        let version = GameVersion::from_game_id("GEAJ8P").unwrap();
        assert_eq!(version.platform, Platform::GameCube);
        assert_eq!(version.region, Region::Jp);
        assert!(version.is_gc_jp());
    }

    #[test]
    fn test_detect_gc_eu() {
        let version = GameVersion::from_game_id("GEAP8P").unwrap();
        assert_eq!(version.platform, Platform::GameCube);
        assert_eq!(version.region, Region::Eu);
        assert!(version.is_gc_eu());
    }

    #[test]
    fn test_reject_invalid() {
        assert!(GameVersion::from_game_id("XXXX8P").is_none());
        assert!(GameVersion::from_game_id("GEA").is_none());
    }
}
