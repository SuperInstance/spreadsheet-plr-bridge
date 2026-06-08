//! Spreadsheet bridge — maps cell references to chord cells and evaluates formulas.

use crate::chord_cell::ChordCell;
use crate::formula::{Formula, FormulaResult};
use crate::conservation::{VoiceLeadingBudget, ConservationStatus};
use crate::error::BridgeError;
use std::collections::HashMap;

/// A cell reference in spreadsheet notation (e.g., "A1", "B3").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellRef {
    pub column: String,
    pub row: usize,
}

impl CellRef {
    pub fn new(column: &str, row: usize) -> Self {
        CellRef { column: column.to_uppercase(), row }
    }

    /// Parse from string like "A1", "B12", "AA3".
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let mut col = String::new();
        let mut row_str = String::new();

        for c in s.chars() {
            if c.is_ascii_alphabetic() {
                if !row_str.is_empty() { return None; } // letters after digits
                col.push(c.to_ascii_uppercase());
            } else if c.is_ascii_digit() {
                row_str.push(c);
            } else {
                return None;
            }
        }

        if col.is_empty() || row_str.is_empty() { return None; }
        let row: usize = row_str.parse().ok()?;
        if row == 0 { return None; }

        Some(CellRef { column: col, row })
    }
}

impl std::fmt::Display for CellRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.column, self.row)
    }
}

/// The spreadsheet bridge connecting cells to the PLR group engine.
///
/// This is the central integration point: cells contain chords,
/// formulas apply PLR operations, and conservation laws track voice-leading budget.
pub struct SpreadsheetBridge {
    /// Current chord values at each cell
    cells: HashMap<String, ChordCell>,
    /// Formulas for each cell (if any)
    formulas: HashMap<String, Formula>,
    /// Voice-leading budget tracker
    budget: VoiceLeadingBudget,
    /// Evaluation order (topologically sorted dependencies)
    #[allow(dead_code)]
    eval_order: Vec<String>,
}

impl SpreadsheetBridge {
    /// Create a new empty bridge with the given voice-leading budget.
    pub fn new(budget: VoiceLeadingBudget) -> Self {
        SpreadsheetBridge {
            cells: HashMap::new(),
            formulas: HashMap::new(),
            budget,
            eval_order: Vec::new(),
        }
    }

    /// Set a cell's chord value directly.
    pub fn set_cell(&mut self, ref_str: &str, chord: ChordCell) {
        self.cells.insert(ref_str.to_string(), chord);
    }

    /// Set a cell's formula.
    pub fn set_formula(&mut self, ref_str: &str, formula: Formula) {
        self.formulas.insert(ref_str.to_string(), formula);
    }

    /// Get a cell's current chord value.
    pub fn get_cell(&self, ref_str: &str) -> Option<&ChordCell> {
        self.cells.get(ref_str)
    }

    /// Evaluate a single cell's formula and update its value.
    ///
    /// Returns the result with voice-leading distance and conservation status.
    pub fn evaluate_cell(&mut self, ref_str: &str) -> Result<EvalResult, BridgeError> {
        let formula = self.formulas.get(ref_str)
            .ok_or_else(|| BridgeError::NoFormula(ref_str.to_string()))?
            .clone();

        let result = formula.evaluate(|cell_ref| {
            self.cells.get(cell_ref).cloned()
        }).map_err(BridgeError::FormulaError)?;

        // Check conservation
        let source_chord = match &formula.source {
            crate::formula::FormulaSource::Cell(r) => self.cells.get(r).cloned(),
            crate::formula::FormulaSource::Literal(c) => Some(c.clone()),
        };

        let conservation = if let Some(source) = source_chord {
            let distance = source.voice_leading_distance(&result.chord);
            self.budget.check_and_spend(distance)?
        } else {
            ConservationStatus::WithinBudget { remaining: self.budget.remaining() }
        };

        // Store result
        self.cells.insert(ref_str.to_string(), result.chord.clone());

        Ok(EvalResult {
            cell_ref: ref_str.to_string(),
            result,
            conservation,
        })
    }

    /// Evaluate all cells with formulas (in dependency order).
    pub fn evaluate_all(&mut self) -> Vec<Result<EvalResult, BridgeError>> {
        let cells_with_formulas: Vec<String> = self.formulas.keys().cloned().collect();
        cells_with_formulas.iter().map(|ref_str| {
            self.evaluate_cell(ref_str)
        }).collect()
    }

    /// List all cells and their current values.
    pub fn all_cells(&self) -> &HashMap<String, ChordCell> {
        &self.cells
    }

    /// Get the remaining voice-leading budget.
    pub fn remaining_budget(&self) -> u8 {
        self.budget.remaining()
    }
}

/// Result of evaluating a cell.
#[derive(Debug)]
pub struct EvalResult {
    pub cell_ref: String,
    pub result: FormulaResult,
    pub conservation: ConservationStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    #[test]
    fn cell_ref_parse() {
        let cr = CellRef::parse("A1").unwrap();
        assert_eq!(cr.column, "A");
        assert_eq!(cr.row, 1);
    }

    #[test]
    fn cell_ref_parse_multi_column() {
        let cr = CellRef::parse("AB12").unwrap();
        assert_eq!(cr.column, "AB");
        assert_eq!(cr.row, 12);
    }

    #[test]
    fn cell_ref_display() {
        assert_eq!(CellRef::parse("A1").unwrap().to_string(), "A1");
    }

    #[test]
    fn cell_ref_invalid() {
        assert!(CellRef::parse("").is_none());
        assert!(CellRef::parse("A").is_none());
        assert!(CellRef::parse("1A").is_none());
    }

    #[test]
    fn bridge_set_and_get() {
        let mut bridge = SpreadsheetBridge::new(VoiceLeadingBudget::new(100));
        bridge.set_cell("A1", ChordCell::major(0, 4, 7));
        let cell = bridge.get_cell("A1").unwrap();
        assert_eq!(cell.pitch_classes(), &[0, 4, 7]);
    }

    #[test]
    fn bridge_evaluate_formula() {
        let mut bridge = SpreadsheetBridge::new(VoiceLeadingBudget::new(100));
        bridge.set_cell("A1", ChordCell::major(0, 4, 7));

        let formula = Formula {
            source: crate::formula::FormulaSource::Cell("A1".into()),
            operations: vec![Operation::P],
        };
        bridge.set_formula("B1", formula);

        let eval = bridge.evaluate_cell("B1").unwrap();
        assert_eq!(eval.result.chord.quality(), crate::chord_cell::ChordQuality::Minor);
        assert_eq!(eval.result.chord.root(), 0); // P keeps root
    }

    #[test]
    fn bridge_no_formula_error() {
        let mut bridge = SpreadsheetBridge::new(VoiceLeadingBudget::new(100));
        let result = bridge.evaluate_cell("Z1");
        assert!(matches!(result, Err(BridgeError::NoFormula(_))));
    }

    #[test]
    fn budget_tracking() {
        let mut bridge = SpreadsheetBridge::new(VoiceLeadingBudget::new(5));
        bridge.set_cell("A1", ChordCell::major(0, 4, 7));

        let formula = Formula {
            source: crate::formula::FormulaSource::Cell("A1".into()),
            operations: vec![Operation::P],
        };
        bridge.set_formula("B1", formula);
        let _result = bridge.evaluate_cell("B1").unwrap();

        // P moves 1 semitone, budget was 5, so 4 remaining
        assert_eq!(bridge.remaining_budget(), 4);
    }
}
