//! Conservation tracking — voice-leading budget enforcement.

/// A voice-leading budget that constrains how far chords can move.
///
/// This implements the conservation law from the fleet physics:
/// total voice-leading distance across all evaluations must not exceed the budget.
/// This prevents formulas from producing jarring leaps — every transition stays smooth.
#[derive(Debug, Clone)]
pub struct VoiceLeadingBudget {
    /// Total budget in semitones
    total: u8,
    /// Budget remaining
    remaining: u8,
}

impl VoiceLeadingBudget {
    /// Create a new budget with the given total (in semitones).
    pub fn new(total: u8) -> Self {
        VoiceLeadingBudget { total, remaining: total }
    }

    /// Check if a distance is within budget and spend it.
    /// Returns the conservation status.
    pub fn check_and_spend(&mut self, distance: u8) -> Result<ConservationStatus, crate::error::BridgeError> {
        if distance > self.remaining {
            return Err(crate::error::BridgeError::BudgetExceeded {
                requested: distance,
                remaining: self.remaining,
            });
        }
        self.remaining -= distance;
        Ok(ConservationStatus::WithinBudget { remaining: self.remaining })
    }

    /// Get the remaining budget.
    pub fn remaining(&self) -> u8 {
        self.remaining
    }

    /// Get the total budget.
    pub fn total(&self) -> u8 {
        self.total
    }

    /// Reset the budget to full.
    pub fn reset(&mut self) {
        self.remaining = self.total;
    }

    /// Fraction of budget remaining [0.0, 1.0].
    pub fn fraction_remaining(&self) -> f64 {
        if self.total == 0 { return 1.0; }
        self.remaining as f64 / self.total as f64
    }
}

/// Status of conservation law check.
#[derive(Debug, Clone, PartialEq)]
pub enum ConservationStatus {
    /// Within budget, with this much remaining
    WithinBudget { remaining: u8 },
    /// Exactly on budget (no more moves allowed)
    ExactlyOnBudget,
    /// Budget was exceeded (should not happen if check_and_spend is used)
    BudgetExceeded { by: u8 },
}

impl std::fmt::Display for ConservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConservationStatus::WithinBudget { remaining } => {
                write!(f, "✓ Within budget ({} remaining)", remaining)
            }
            ConservationStatus::ExactlyOnBudget => write!(f, "⚡ Exactly on budget"),
            ConservationStatus::BudgetExceeded { by } => {
                write!(f, "✗ Budget exceeded by {} semitones", by)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_budget() {
        let budget = VoiceLeadingBudget::new(10);
        assert_eq!(budget.remaining(), 10);
        assert_eq!(budget.total(), 10);
    }

    #[test]
    fn spend_within_budget() {
        let mut budget = VoiceLeadingBudget::new(10);
        let status = budget.check_and_spend(3).unwrap();
        assert_eq!(budget.remaining(), 7);
        assert!(matches!(status, ConservationStatus::WithinBudget { remaining: 7 }));
    }

    #[test]
    fn spend_exactly_budget() {
        let mut budget = VoiceLeadingBudget::new(5);
        let status = budget.check_and_spend(5).unwrap();
        assert_eq!(budget.remaining(), 0);
        assert!(matches!(status, ConservationStatus::WithinBudget { remaining: 0 }));
    }

    #[test]
    fn spend_over_budget() {
        let mut budget = VoiceLeadingBudget::new(5);
        let result = budget.check_and_spend(6);
        assert!(result.is_err());
        assert_eq!(budget.remaining(), 5); // not spent
    }

    #[test]
    fn reset_budget() {
        let mut budget = VoiceLeadingBudget::new(10);
        budget.check_and_spend(7).unwrap();
        budget.reset();
        assert_eq!(budget.remaining(), 10);
    }

    #[test]
    fn fraction_remaining() {
        let mut budget = VoiceLeadingBudget::new(10);
        assert!((budget.fraction_remaining() - 1.0).abs() < 0.001);
        budget.check_and_spend(5).unwrap();
        assert!((budget.fraction_remaining() - 0.5).abs() < 0.001);
    }

    #[test]
    fn display_status() {
        let status = ConservationStatus::WithinBudget { remaining: 5 };
        assert!(status.to_string().contains("5 remaining"));
    }
}
