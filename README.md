# spreadsheet-plr-bridge

[![crates.io](https://img.shields.io/crates/v/spreadsheet-plr-bridge.svg)](https://crates.io/crates/spreadsheet-plr-bridge)
[![docs.rs](https://docs.rs/spreadsheet-plr-bridge/badge.svg)](https://docs.rs/spreadsheet-plr-bridge)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## The Spreadsheet Moment for AI Music

This is the integration point where two crates meet and become something neither could be alone.

**[spreadsheet-engine]** gives you cells that hold values and formulas that compute them. **[groovemesh-plr]** gives you the PLR group (P, L, R operations on triads) that guarantees mathematically correct voice leading. This bridge connects them: spreadsheet cells hold chords, formulas use PLR operations, and a voice-leading conservation law prevents any formula from producing a jarring transition.

**You literally cannot write a formula that sounds bad.**

## The Problem

In a traditional music spreadsheet, a cell formula `A1 + 2` would transpose a chord by 2 semitones — but the result might be a chord that doesn't exist in any sensible harmonic context. There's no constraint that the output is musically meaningful.

In a traditional PLR system, you have mathematically guaranteed voice leading — but it's abstract. You're working with group operations on set classes, not with something as intuitive as a spreadsheet.

## The Insight

The PLR group (isomorphic to D₁₂, the dihedral group of order 24) acts transitively on the 24 major and minor triads. Every operation moves exactly 1 semitone in voice-leading space. If you use PLR operations as spreadsheet formulas:

- Every formula produces a valid triad (major or minor)
- Every transition is minimal voice leading (1 semitone per operation)
- A conservation budget limits total voice-leading distance across the sheet

This is the "spreadsheet moment" — a familiar interface (cells, formulas, references) backed by mathematical guarantees.

## How It Works

### Cells hold chords

```rust
use spreadsheet_plr_bridge::*;

// C major in cell A1
let c_major = ChordCell::major(0, 4, 7);

// C minor in cell A2
let c_minor = ChordCell::minor(0);
```

### Formulas use PLR operations

```rust
// P(C) = Cm (parallel: flips third)
let result = c_major.apply(Operation::P);
assert_eq!(result.quality(), ChordQuality::Minor);
assert_eq!(result.root(), 0); // same root

// L(C) = Em (leading-tone: shifts root)
let e_minor = c_major.apply(Operation::L);
assert_eq!(e_minor.root(), 4);

// R(C) = Am (relative: shifts root)
let a_minor = c_major.apply(Operation::R);
assert_eq!(a_minor.root(), 9);
```

### Compose operations for progressions

```rust
// Romanesca: P∘L∘P
let progression = c_major.compose(&[Operation::P, Operation::L, Operation::P]);

// Or from a formula string
let formula = Formula::parse("R∘L∘P(C)").unwrap();
let result = formula.evaluate(|_| None).unwrap();
```

### Conservation law enforces smooth transitions

```rust
use spreadsheet_plr_bridge::{SpreadsheetBridge, VoiceLeadingBudget};

// Budget: max 20 semitones of voice-leading across all evaluations
let mut bridge = SpreadsheetBridge::new(VoiceLeadingBudget::new(20));

bridge.set_cell("A1", ChordCell::major(0, 4, 7));  // C major
bridge.set_formula("B1", Formula::parse("P(A1)").unwrap());  // P(C) = Cm
bridge.set_formula("C1", Formula::parse("L(B1)").unwrap());  // L(Cm) = E♭
bridge.set_formula("D1", Formula::parse("R(C1)").unwrap());  // R(E♭) = ...

let results = bridge.evaluate_all();
// Each evaluation costs exactly 1 semitone (PLR property)
// After 3 evaluations: 3 semitones spent, 17 remaining
assert_eq!(bridge.remaining_budget(), 17);
```

## The PLR Group

| Operation | Major → | Minor → | Voice leading |
|---|---|---|---|
| **P** (Parallel) | Minor, same root | Major, same root | 1 semitone (third flips) |
| **L** (Leading-tone) | Minor, root+4 | Major, root+3 | 1 semitone |
| **R** (Relative) | Minor, root+9 | Major, root+8 | 1 semitone |

The group is isomorphic to D₁₂: P, L, R are reflections, and compositions generate all 24 transformations between major and minor triads. Only P is an involution (P∘P = id). L and R compose into transpositions.

## Module Map

| Module | What it does |
|---|---|
| `chord_cell` | `ChordCell` — triad stored as (root, quality), derives pitch classes |
| `operation` | `Operation` — P, L, R group elements |
| `formula` | `Formula` — parse and evaluate PLR formulas on cell references |
| `bridge` | `SpreadsheetBridge` — cells, formulas, evaluation engine |
| `conservation` | `VoiceLeadingBudget` — track and enforce total voice-leading distance |
| `error` | `BridgeError` — budget exceeded, missing cells, parse failures |

## Links

- [Documentation](https://docs.rs/spreadsheet-plr-bridge)
- [Repository](https://github.com/SuperInstance/spreadsheet-plr-bridge)
- [crates.io](https://crates.io/crates/spreadsheet-plr-bridge)
- See also: [spreadsheet-engine](https://crates.io/crates/spreadsheet-engine), [groovemesh-plr](https://crates.io/crates/groovemesh-plr)

## License

MIT
