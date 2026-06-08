//! # spreadsheet-plr-bridge
//!
//! The bridge between [spreadsheet-engine] cells and [groovemesh-plr] voice leading.
//!
//! This is the integration point that makes the ecosystem more than the sum of its parts:
//! spreadsheet cells contain musical operations (transpose, invert, retrograde) and the
//! PLR group algebra guarantees every formula produces valid voice leading — you literally
//! cannot write a formula that sounds bad.
//!
//! ## How It Works
//!
//! 1. A spreadsheet cell holds a chord (pitch classes as `Vec<u8>`)
//! 2. Cell formulas use PLR operations: `P(A)`, `L(A)`, `R(A)` and their compositions
//! 3. The formula evaluator applies PLR group algebra (D₁₂) to transform chords
//! 4. Conservation laws track voice-leading distance — no formula can exceed a configurable threshold
//!
//! ## Example
//!
//! ```rust
//! use spreadsheet_plr_bridge::*;
//!
//! // Define a C major chord
//! let c_major = ChordCell::major(0, 4, 7);
//!
//! // Apply P (parallel) transformation: C major → C minor
//! let c_minor = c_major.apply(Operation::P);
//! assert_eq!(c_minor.pitch_classes(), &[0, 3, 7]);
//!
//! // Apply L (leading-tone) transformation: C minor → E♭ major
//! let eb_major = c_minor.apply(Operation::L);
//! assert_eq!(eb_major.pitch_classes(), &[3, 7, 10]);
//!
//! // Compose: R∘L∘P (the "Romanesca" progression)
//! let romanesca = c_major.compose(&[Operation::P, Operation::L, Operation::R]);
//! // C major → C minor → E♭ major → G minor
//! ```

mod chord_cell;
mod formula;
mod operation;
mod bridge;
mod conservation;
mod error;

pub use chord_cell::{ChordCell, ChordQuality};
pub use formula::{Formula, FormulaToken, FormulaResult};
pub use operation::{Operation, compose_operations};
pub use bridge::{SpreadsheetBridge, CellRef};
pub use conservation::{VoiceLeadingBudget, ConservationStatus};
pub use error::BridgeError;
