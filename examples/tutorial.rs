//! Tutorial: Spreadsheet cells that play music
//!
//! Shows how to create a spreadsheet where cells contain chords and formulas
//! use PLR (Parallel-Leittonwechsel-Relative) group algebra.

use spreadsheet_plr_bridge::*;

fn main() {
    println!("=== Spreadsheet-PLR Bridge Tutorial ===\n");

    // Part 1: Create a bridge with voice leading budget
    println!("Part 1: Setting up the bridge");
    let budget = VoiceLeadingBudget::new(30);
    let mut bridge = SpreadsheetBridge::new(budget);
    println!("  Voice leading budget: 30 semitones\n");

    // Part 2: Set cells with chords
    println!("Part 2: Creating chord cells");
    bridge.set_cell("A1", ChordCell::from_root_quality(0, ChordQuality::Major)); // C major
    bridge.set_cell("B1", ChordCell::from_root_quality(7, ChordQuality::Major)); // G major
    println!("  A1 = C major, B1 = G major\n");

    // Part 3: Apply PLR operations as formulas
    println!("Part 3: PLR formula operations");
    bridge.set_formula("A2", Formula::parse("P(A1)").unwrap());
    bridge.set_formula("A3", Formula::parse("L(A1)").unwrap());
    bridge.set_formula("A4", Formula::parse("R(A1)").unwrap());
    bridge.set_formula("B2", Formula::parse("P(B1)").unwrap());
    println!("  A2=P(A1) A3=L(A1) A4=R(A1) B2=P(B1)\n");

    // Part 4: Evaluate all cells
    println!("Part 4: Evaluation results");
    let results = bridge.evaluate_all();
    for result in &results {
        match result {
            Ok(eval) => println!("  {} → {:?} (conservation: {:?})", 
                eval.cell_ref, eval.result, eval.conservation),
            Err(e) => println!("  Error: {}", e),
        }
    }
    println!();

    // Part 5: Conservation budget
    println!("Part 5: Budget remaining: {}", bridge.remaining_budget());
    println!("  PLR always produces valid voice leading!");
}
