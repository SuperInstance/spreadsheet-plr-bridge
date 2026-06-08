//! Formula language for spreadsheet cells — PLR operations on chord references.

use crate::chord_cell::ChordCell;
use crate::operation::Operation;

/// A token in the formula language.
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaToken {
    /// A PLR operation (P, L, R)
    Op(Operation),
    /// A cell reference (e.g., "A1")
    CellRef(String),
    /// An opening parenthesis
    LParen,
    /// A closing parenthesis
    RParen,
    /// The composition operator (∘ or ".")
    Compose,
    /// A chord literal: "C", "Dm", "F#dim"
    ChordLiteral(String, ChordCell),
}

/// A parsed formula — a composition of operations applied to a source.
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    /// The source: either a cell reference or a chord literal
    pub source: FormulaSource,
    /// Operations to apply left-to-right
    pub operations: Vec<Operation>,
}

/// Where a formula gets its initial chord.
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaSource {
    /// Reference to another cell
    Cell(String),
    /// A chord literal parsed from name
    Literal(ChordCell),
}

/// The result of evaluating a formula.
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaResult {
    /// The resulting chord
    pub chord: ChordCell,
    /// Total voice-leading distance from source to result
    pub total_distance: u8,
    /// Operations applied (for audit trail)
    pub operations_applied: Vec<Operation>,
}

impl Formula {
    /// Parse a simple formula: operations followed by a source.
    /// Supports: "P(C)", "L(A1)", "R∘L∘P(C)"
    pub fn parse(input: &str) -> Result<Self, String> {
        let tokens = tokenize(input)?;
        parse_tokens(&tokens)
    }

    /// Evaluate the formula against a cell resolver.
    pub fn evaluate<F>(&self, resolver: F) -> Result<FormulaResult, String>
    where
        F: Fn(&str) -> Option<ChordCell>,
    {
        let source_chord = match &self.source {
            FormulaSource::Cell(ref_str) => {
                resolver(ref_str).ok_or_else(|| format!("Cell '{}' not found", ref_str))?
            }
            FormulaSource::Literal(chord) => chord.clone(),
        };

        let mut current = source_chord.clone();
        let mut total_distance = 0u8;
        let mut applied = Vec::new();

        for &op in &self.operations {
            let prev = current.clone();
            current = prev.apply(op);
            total_distance += prev.voice_leading_distance(&current);
            applied.push(op);
        }

        Ok(FormulaResult {
            chord: current,
            total_distance,
            operations_applied: applied,
        })
    }
}

/// Try to parse a chord literal like "C", "Dm", "F#dim", "Bbm".
fn parse_chord_literal(s: &str) -> Option<ChordCell> {
    use crate::chord_cell::ChordQuality;

    let note_map: &[(&str, u8)] = &[
        ("C#", 1), ("Db", 1), ("D#", 3), ("Eb", 3), ("F#", 6), ("Gb", 6),
        ("G#", 8), ("Ab", 8), ("A#", 10), ("Bb", 10),
        ("C", 0), ("D", 2), ("E", 4), ("F", 5), ("G", 7), ("A", 9), ("B", 11),
    ];

    // Find longest matching note name
    let mut best_root: Option<u8> = None;
    let mut best_len = 0;

    for &(name, pc) in note_map {
        if s.starts_with(name) && name.len() > best_len {
            best_root = Some(pc);
            best_len = name.len();
        }
    }

    let root = best_root?;
    let suffix = &s[best_len..];

    match suffix {
        "" | "maj" | "M" => Some(ChordCell::from_root_quality(root, ChordQuality::Major)),
        "m" | "min" => Some(ChordCell::from_root_quality(root, ChordQuality::Minor)),
        "dim" => Some(ChordCell::from_root_quality(root, ChordQuality::Diminished)),
        "aug" => Some(ChordCell::from_root_quality(root, ChordQuality::Augmented)),
        _ => None,
    }
}

/// Tokenize a formula string.
fn tokenize(input: &str) -> Result<Vec<FormulaToken>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => { chars.next(); }
            '(' => { tokens.push(FormulaToken::LParen); chars.next(); }
            ')' => { tokens.push(FormulaToken::RParen); chars.next(); }
            '∘' | '.' => { tokens.push(FormulaToken::Compose); chars.next(); }
            'P' => { tokens.push(FormulaToken::Op(Operation::P)); chars.next(); }
            'L' => { tokens.push(FormulaToken::Op(Operation::L)); chars.next(); }
            'R' => { tokens.push(FormulaToken::Op(Operation::R)); chars.next(); }
            _ if c.is_ascii_alphabetic() || c == '#' => {
                let mut ident = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '#' || next == 'b' {
                        ident.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Some(chord) = parse_chord_literal(&ident) {
                    tokens.push(FormulaToken::ChordLiteral(ident, chord));
                } else {
                    tokens.push(FormulaToken::CellRef(ident));
                }
            }
            _ => return Err(format!("Unexpected character: '{}'", c)),
        }
    }

    Ok(tokens)
}

/// Parse tokens: collect leading ops, then source.
fn parse_tokens(tokens: &[FormulaToken]) -> Result<Formula, String> {
    if tokens.is_empty() {
        return Err("Empty formula".into());
    }

    let mut pos = 0;
    let mut operations = Vec::new();

    // Collect leading operations (skipping compose separators between them)
    while pos < tokens.len() {
        match &tokens[pos] {
            FormulaToken::Op(op) => { operations.push(*op); pos += 1; }
            FormulaToken::Compose => { pos += 1; } // skip compose between ops
            _ => break,
        }
    }

    // Source can be wrapped in parens: Op(source) or just source
    if pos < tokens.len() && matches!(tokens[pos], FormulaToken::LParen) {
        pos += 1; // skip (
    }

    let source = if pos >= tokens.len() {
        return Err("Expected source after operations".into());
    } else {
        match &tokens[pos] {
            FormulaToken::CellRef(name) => {
                let s = FormulaSource::Cell(name.clone());
                pos += 1;
                s
            }
            FormulaToken::ChordLiteral(_, chord) => {
                let s = FormulaSource::Literal(chord.clone());
                pos += 1;
                s
            }
            other => return Err(format!("Expected source, got {:?}", token_debug(other))),
        }
    };

    // Skip closing paren if present
    if pos < tokens.len() && matches!(tokens[pos], FormulaToken::RParen) {
        pos += 1;
    }

    // After source, collect more ops with optional compose separators
    while pos < tokens.len() {
        match &tokens[pos] {
            FormulaToken::Compose => { pos += 1; }
            FormulaToken::Op(op) => { operations.push(*op); pos += 1; }
            _ => { pos += 1; }
        }
    }

    Ok(Formula { source, operations })
}

fn token_debug(t: &FormulaToken) -> &'static str {
    match t {
        FormulaToken::Op(_) => "Op",
        FormulaToken::CellRef(_) => "CellRef",
        FormulaToken::LParen => "LParen",
        FormulaToken::RParen => "RParen",
        FormulaToken::Compose => "Compose",
        FormulaToken::ChordLiteral(_, _) => "ChordLiteral",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chord_cell::ChordQuality;

    #[test]
    fn parse_p_on_c() {
        let formula = Formula::parse("P(C)").unwrap();
        assert!(matches!(formula.source, FormulaSource::Literal(_)));
        assert_eq!(formula.operations, vec![Operation::P]);
    }

    #[test]
    fn parse_cell_ref() {
        let formula = Formula::parse("A1").unwrap();
        assert!(matches!(formula.source, FormulaSource::Cell(ref s) if s == "A1"));
        assert!(formula.operations.is_empty());
    }

    #[test]
    fn parse_rlp_on_chord() {
        let formula = Formula::parse("R∘L∘P(C)").unwrap();
        assert_eq!(formula.operations, vec![Operation::R, Operation::L, Operation::P]);
    }

    #[test]
    fn evaluate_p_on_c_major() {
        let formula = Formula {
            source: FormulaSource::Literal(ChordCell::major(0, 4, 7)),
            operations: vec![Operation::P],
        };
        let result = formula.evaluate(|_| None).unwrap();
        assert_eq!(result.chord.quality(), ChordQuality::Minor);
        assert_eq!(result.chord.root(), 0);
        assert_eq!(result.total_distance, 1);
    }

    #[test]
    fn evaluate_l_on_c_major() {
        let formula = Formula {
            source: FormulaSource::Literal(ChordCell::major(0, 4, 7)),
            operations: vec![Operation::L],
        };
        let result = formula.evaluate(|_| None).unwrap();
        // L(C+) = e- (E minor)
        assert_eq!(result.chord.quality(), ChordQuality::Minor);
        assert_eq!(result.chord.root(), 4);
    }

    #[test]
    fn evaluate_with_cell_ref() {
        let formula = Formula {
            source: FormulaSource::Cell("A1".into()),
            operations: vec![Operation::L],
        };
        let result = formula.evaluate(|ref_str| {
            if ref_str == "A1" { Some(ChordCell::major(0, 4, 7)) } else { None }
        }).unwrap();
        // L(C+) = e-
        assert_eq!(result.chord.quality(), ChordQuality::Minor);
        assert_eq!(result.chord.root(), 4);
    }

    #[test]
    fn evaluate_cell_not_found() {
        let formula = Formula {
            source: FormulaSource::Cell("Z99".into()),
            operations: vec![],
        };
        assert!(formula.evaluate(|_| None).is_err());
    }

    #[test]
    fn parse_chord_c_major() {
        let chord = parse_chord_literal("C").unwrap();
        assert_eq!(chord.pitch_classes(), &[0, 4, 7]);
        assert_eq!(chord.quality(), ChordQuality::Major);
    }

    #[test]
    fn parse_chord_d_minor() {
        let chord = parse_chord_literal("Dm").unwrap();
        assert_eq!(chord.pitch_classes(), &[2, 5, 9]);
        assert_eq!(chord.quality(), ChordQuality::Minor);
    }

    #[test]
    fn parse_chord_f_sharp_major() {
        let chord = parse_chord_literal("F#").unwrap();
        // F# = 6, major = [6, 10, 1] sorted = [1, 6, 10]
        assert_eq!(chord.pitch_classes(), &[1, 6, 10]);
        assert_eq!(chord.quality(), ChordQuality::Major);
        assert_eq!(chord.root(), 6);
    }
}
