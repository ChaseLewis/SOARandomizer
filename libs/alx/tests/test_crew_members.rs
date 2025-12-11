//! Integration tests for CrewMember entries.

mod common;

#[test]
fn test_read_crew_members() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let crew = game.read_crew_members().unwrap();
    
    // Should have 22 crew members (IDs 0-21)
    assert_eq!(crew.len(), 22, "Expected 22 crew members");
    
    // First crew member should be Lawrence
    let first = &crew[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.name, "Lawrence");
    assert_eq!(first.position_id, 0); // Helmsman
    
    println!("✓ Read {} crew members", crew.len());
}

#[test]
fn test_crew_member_positions() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let crew = game.read_crew_members().unwrap();
    
    // Lawrence is Helmsman
    let lawrence = &crew[0];
    assert_eq!(lawrence.position_name(), "Helmsman");
    
    // Brabham is Engineer
    let brabham = &crew[2];
    assert_eq!(brabham.name, "Brabham");
    assert_eq!(brabham.position_name(), "Engineer");
    
    // Belle is Gunner
    let belle = &crew[4];
    assert_eq!(belle.name, "Belle");
    assert_eq!(belle.position_name(), "Gunner");
    
    println!("✓ Crew member positions verified");
}

#[test]
fn test_crew_member_traits() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let crew = game.read_crew_members().unwrap();
    
    // Lawrence has Quick trait
    let lawrence = &crew[0];
    assert_eq!(lawrence.trait_id, 4); // Quick
    assert_eq!(lawrence.trait_value, 30);
    
    println!("✓ Crew member traits verified");
}

