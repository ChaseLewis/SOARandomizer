//! Integration tests for ISO loading and game detection.

mod common;

use alx::game::{Platform, Region};

#[test]
fn test_open_iso() {
    skip_if_no_iso!();

    let game = common::try_load_game();
    assert!(game.is_ok(), "Failed to open ISO: {:?}", game.err());

    let game = game.unwrap();
    let version = game.version();

    println!("Detected game version: {}", version);
    assert_eq!(version.platform, Platform::GameCube);
    assert_eq!(version.region, Region::Us);
    assert!(version.is_gc_us());
}

#[test]
fn test_game_version_detection() {
    skip_if_no_iso!();

    let game = common::load_game();
    let version = game.version();

    assert_eq!(version.product_id, "GEAE8P");
    assert!(version.is_gc_us());
    assert!(!version.is_gc_jp());
    assert!(!version.is_gc_eu());
}
