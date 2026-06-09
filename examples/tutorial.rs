//! Tutorial: Spreadsheet cells that play music
//!
//! Shows how to create a spreadsheet where cells contain chords and formulas
//! use PLR (Parallel-Leittonwechsel-Relative) group algebra.

use spreadsheet_plr_bridge::*;
use spreadsheet_plr_bridge::operation::Operation;
use spreadsheet_plr_bridge::formula::Formula;

fn main() {
    println!("=== Spreadsheet-PLR Bridge Tutorial ===\n");

    // Part 1: Create a bridge with voice leading budget
    println!("Part 1: Setting up the bridge");
    let budget = VoiceLeadingBudget::new(30);
    let mut bridge = SpreadsheetBridge::new(budget);
    println!("  Voice leading budget: 30 semitones\n");

    // Part 2: Set cells with chords
    println!("Part 2: Creating chord cells");
    // C major in A1, G major in B1
    bridge.set_cell("A1", ChordCell::from_root_quality(0, ChordQuality::Major)); // C major
    bridge.set_cell("B1", ChordCell::from_root_quality(7, ChordQuality::Major)); // G major
    println!("  A1 = C major (root=0)");
    println!("  B1 = G major (root=7)\n");

    // Part 3: Apply PLR operations as formulas
    println!("Part 3: PLR formula operations");
    // P(C) = c minor, L(C) = e minor, R(C) = a minor
    bridge.set_formula("A2", Formula::parse("P(A1)").unwrap());
    bridge.set_formula("A3", Formula::parse("L(A1)").unwrap());
    bridge.set_formula("A4", Formula::parse("R(A1)").unwrap());
    println!("  A2 = P(A1) = Parallel of C");
    println!("  A3 = L(A1) = Leittonwechsel of C");
    println!("  A4 = R(A1) = Relative of C\n");

    // Part 4: Evaluate all cells
    println!("Part 4: Evaluation");
    let results = bridge.evaluate_all();
    for result in &results {
        match result {
            Ok(eval) => println!("  {} → root={:?} quality={:?}", 
                eval.cell_ref, eval.root, eval.quality),
            Err(e) => println!("  Error: {}", e),
        }
    }
    println!();

    // Part 5: Check budget conservation
    println!("Part 5: Conservation check");
    println!("  Remaining voice leading budget: {}", bridge.remaining_budget());
    println!("  The key insight: PLR operations always produce valid voice leading,");
    println!("  and the budget ensures total musical distance stays bounded.");
}
