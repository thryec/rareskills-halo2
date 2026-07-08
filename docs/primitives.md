# Halo2 Primitives

Every circuit in this bootcamp is a composition of six primitives. New problems are new compositions, not new patterns. When a problem feels unfamiliar, decompose it with the checklist below and name which primitives it needs before writing any Rust.

Companion docs: `glossary.md` (API lookup), `halo2_explainer.md` (mental model and lifecycle).

## The six primitives

### 1. Gate
Force an equation between cells in a row-window.

- API: `meta.create_gate` + selector + `Rotation` queries.
- The equation must be a polynomial that equals zero. Degree is bounded by the proving system config, so high-degree relations get split across rows or moved to primitive 5 or 6.
- Reach for it whenever the relation is directly expressible: `q * (a + b - c)`, `flag * (flag - 1)`, `limb * (limb - 1) * (limb - 2) * (limb - 3)`.
- A gate only sees a fixed window of rows around the selector (via rotations). Anything farther away needs primitive 2.

### 2. Copy constraint
Make a value travel to another cell anywhere in the table.

- API: `meta.enable_equality(col)` in configure, `region.constrain_equal(a.cell(), b.cell())` in synthesize. Backed by the permutation argument.
- Reach for it when a gate's input lives somewhere else: chaining Fibonacci rows, feeding a chip's output into the next chip's input row.
- Rule of thumb: gates relate neighbors, copies relate strangers.

### 3. Constant pin
Fix a cell to a value known at circuit-definition time.

- API: fixed column + `constrain_constant`, or a dedicated boundary gate like `q_first * (a - 1)`.
- Reach for it for boundary conditions. A recurrence without pinned initial values is underconstrained (any starting pair satisfies the Fibonacci gate).

### 4. Instance bind
Expose a cell to the verifier as a public input.

- API: instance column + `layouter.constrain_instance(cell, instance_col, row)`.
- Reach for it for every value the verifier must see or supply: the claimed result, a public index, a commitment.
- Anything not bound to instance or pinned to fixed is prover-chosen. If the "answer" cell is not instance-bound, the proof claims nothing.

### 5. Lookup
Prove a cell (or tuple of cells) is a member of a table.

- API: `meta.lookup` / `meta.lookup_any`, `TableColumn`, `layouter.assign_table`. Inputs gated by a `complex_selector`.
- Reach for it when the property is "x is in set S" and S is enumerable: byte ranges, ASCII validity, XOR truth tables, allowlists.
- Crossover point vs a product gate: range 0..4 is a cheap degree-4 gate; range 0..256 wants a lookup. Multi-column tables encode whole functions, e.g. rows (x, y, x xor y).

### 6. Auxiliary witness
When the property has no direct low-degree polynomial, invent helper columns plus gates that force them honest.

This is the creative primitive and the one that makes problems feel "new." The known sub-patterns:

- **Inverse trick** (IsZero): witness `x_inv`, gates `x * out = 0` and `1 - out - x * x_inv = 0`. Non-determinism: the prover computes the inverse off-circuit; the gates only verify.
- **Limb / bit decomposition** (range check): witness the limbs, constrain each limb to its small range (gate or lookup), constrain the weighted sum to reconstruct the original.
- **Flag column** (runtime-dependent behavior in a fixed-shape circuit): boolean gate `flag * (flag - 1)`, monotonic gate so once off it stays off, and gates multiplied by `flag` to switch logic per row.
- **Running counter / accumulator**: a column whose row-to-row delta is constrained by a gate, with the final cell instance-bound. Turns "something about all N rows" into one public value.

The honesty rule: every helper column needs constraints that make lying impossible, not just an assignment that happens to be correct. An assigned-but-unconstrained helper is the classic underconstraint bug.

## Decomposition checklist

Answer these five questions on paper before touching Rust. The answers are the circuit.

1. **What does the prover know?** Private values and helpers → advice columns.
2. **What does the verifier learn or supply?** → instance column + binds (primitive 4).
3. **What equations force honesty?** → gates (primitive 1). For each advice column ask: what stops the prover from lying in this cell?
4. **What is not directly polynomial-expressible?** → auxiliary witness sub-pattern or lookup (primitives 6, 5).
5. **Where must values travel?** Boundary pins and cross-row/cross-chip wiring → primitives 3, 2.

## Recipe log

One line per solved problem, in primitive vocabulary. Add an entry after every new problem set; retrieve recipes, not code.

- N-th value of a recurrence at a public index = flag aux + counter aux + monotonic gate + pad to fixed max + instance bind on final counter and final value.
- Nonzero / equality check = inverse-trick aux + two gates.
- k-bit operation (XOR, AND) = limb decomposition aux + per-limb function-table lookup + weighted reconstruction gate.
