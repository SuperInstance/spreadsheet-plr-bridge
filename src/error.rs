//! Error types for the spreadsheet-PLR bridge.

/// Errors that can occur during bridge operations.
#[derive(Debug)]
pub enum BridgeError {
    /// No formula set for the given cell
    NoFormula(String),
    /// Formula evaluation failed
    FormulaError(String),
    /// Voice-leading budget exceeded
    BudgetExceeded {
        requested: u8,
        remaining: u8,
    },
    /// Invalid cell reference
    InvalidCellRef(String),
    /// Invalid chord specification
    InvalidChord(String),
    /// Dependency cycle detected
    CycleDetected(Vec<String>),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::NoFormula(cell) => write!(f, "No formula set for cell '{}'", cell),
            BridgeError::FormulaError(msg) => write!(f, "Formula error: {}", msg),
            BridgeError::BudgetExceeded { requested, remaining } => {
                write!(f, "Voice-leading budget exceeded: requested {} semitones, {} remaining",
                    requested, remaining)
            }
            BridgeError::InvalidCellRef(s) => write!(f, "Invalid cell reference: '{}'", s),
            BridgeError::InvalidChord(s) => write!(f, "Invalid chord: '{}'", s),
            BridgeError::CycleDetected(path) => {
                write!(f, "Dependency cycle: {}", path.join(" → "))
            }
        }
    }
}

impl std::error::Error for BridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_no_formula() {
        let err = BridgeError::NoFormula("A1".into());
        assert!(err.to_string().contains("A1"));
    }

    #[test]
    fn display_budget_exceeded() {
        let err = BridgeError::BudgetExceeded { requested: 10, remaining: 3 };
        let msg = err.to_string();
        assert!(msg.contains("10"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn display_cycle() {
        let err = BridgeError::CycleDetected(vec!["A1".into(), "B1".into(), "A1".into()]);
        assert!(err.to_string().contains("A1 → B1 → A1"));
    }
}
