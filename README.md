# RareSkills Halo2

Homework solutions for Cohort 0 of the RareSkills Halo2 course.

## Circuits and chips

The homework covers the following circuits and chips:

- [x] 01. Addition Gate
- [x] 02. Fibonacci, One Advice Column
- [x] 03. Base-4 Decomposition
- [x] 04. Fibonacci with Public Output
- [x] 05. Arithmetic Chip
- [x] 06. `IsZero` Chip
- [x] 07. Polynomial `a^5 + a = b`
- [x] 08. XOR-32 Chip
- [x] 09. XOR-8 Static Lookup
- [x] 10. 16-bit Addition with Carry
- [ ] 11. 32-bit Chunked Addition
- [x] 12. Fixed Finite Automaton
- [ ] 13. Generic Finite Automaton
- [ ] 14. Poseidon Hash Circuit
- [ ] 15. zkVM

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
