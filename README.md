# RareSkills Halo2

Notes, exercises, and reference code from the RareSkills Halo2 course. The Rust crates use the PSE `halo2` `v0.4.0` tag and Rust 2024 edition.

## Circuits and chips

Each circuit defines its columns, selectors, gates, and lookups in `configure`, then assigns witnesses and tables in `synthesize`. Chips group reusable constraints and assignment methods. `MockProver` tests check valid and invalid witnesses.

- [x] 01. Addition Gate — two advice columns hold `x` and `y`; a selector enforces `x + y = 10`.
- [x] 02. Fibonacci, One Advice Column — row rotations enforce the recurrence, while constants pin the first two values to `1`.
- [x] 03. Base-4 Decomposition — four limbs, each in `0..3`, reconstruct `x` and constrain it to `0..255`.
- [x] 04. Fibonacci with Public Output — advice columns hold the sequence, active flag, and counter; instance values bind the target index and result.
- [x] 05. Arithmetic Chip — shared `a`, `b`, and `c` advice columns use separate selectors for addition and multiplication.
- [x] 06. `IsZero` Chip — an inverse witness and two constraints return `1` only when the input is zero.
- [x] 07. Polynomial `a^5 + a = b` — the arithmetic chip composes multiplication and addition steps, then exposes `b` as a public value.
- [x] 08. XOR-32 Chip — boolean gates constrain each bit, XOR gates compute output bits, and reconstruction gates bind all three integers.
- [x] 09. XOR-8 Static Lookup — a 65,536-row table stores every `(x, y, x XOR y)` tuple.
- [x] 10. 16-bit Addition with Carry — range lookups constrain `x`, `y`, and `z`; a boolean carry enforces `x + y = z + 2^16 × carry`.
- [ ] 11. 32-bit Chunked Addition — smaller limbs with carry propagation.
- [ ] 12. Fixed Finite Automaton — static lookup for state transitions and padded traces.
- [ ] 13. Generic Finite Automaton — public transition table copied into advice for `lookup_any`.

## Setup

Install [Rust](https://www.rust-lang.org/tools/install), then clone this repository. Cargo downloads the Halo2 source on the first build.

Enter an exercise crate, then build or test it with Cargo:

```sh
cd week6
cargo build
cargo test
```

## Notes

- [Halo2 explainer](docs/halo2_explainer.md)
- [Glossary](docs/glossary.md)
- [Circuit primitives](docs/primitives.md)
