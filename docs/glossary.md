# Halo2 Glossary

Quick reference for halo2 PSE v0.4.0 and `ff`. Module names show where common imports come from.

## Field types

- **`Field`** (`ff`) - base field-arithmetic trait. Halo2's `Circuit<F>` is bounded by `F: Field`.
- **`PrimeField`** (`ff`) - `Field + From<u64>` plus prime-field encoding/metadata. Many examples use `F: PrimeField` for convenience.
- **`Fr`** (`halo2curves::bn256`) - concrete BN254 scalar field used as `MyCircuit::<Fr>` in tests.
- **`Value<F>`** (`circuit`) - witness wrapper: `Value::known(f)` or `Value::unknown()`. Similar to `Option<F>`, but designed to avoid branching on witness values.

## Columns

- **`Column<T>`** (`plonk`) - typed circuit column. `T` is a phantom tag such as `Advice`, `Instance`, or `Fixed`.
- **`Advice`** (`plonk`) - private prover-assigned witness column.
- **`Instance`** (`plonk`) - public-input column supplied by prover and verifier.
- **`Fixed`** (`plonk`) - keygen-time constants or tables.
- **`Selector`** (`plonk`) - fixed 0/1 gate switch, used as `q * constraint`.

## Circuit definition

- **`Circuit<F>`** (`plonk`) - trait with `Config`, `FloorPlanner`, `without_witnesses`, `configure`, and `synthesize`.
- **`ConstraintSystem<F>`** (`plonk`) - mutable registry used in `configure` to allocate columns and declare gates, lookups, and permutations.
- **`Expression<F>`** (`plonk`) - symbolic polynomial node. Gate closures return `Vec<Expression<F>>`; each expression must evaluate to zero.
- **`Rotation`** (`poly`) - relative row offset: `cur()` = 0, `next()` = +1, `prev()` = -1.
- **`FloorPlanner`** (`plonk`) - trait for placing regions into rows.
- **`SimpleFloorPlanner`** (`circuit`) - basic planner used by many examples; lays out regions with minimal optimization.

## Layout side

- **`Layouter<F>`** (`circuit`) - interface passed to `synthesize` for regions, namespaces, and public-input constraints.
- **`Region`** (`circuit`) - block of rows for assigning cells at relative offsets.
- **`Cell`** (`circuit`) - opaque coordinate for one table position.
- **`AssignedCell<V, F>`** (`circuit`) - handle returned by assignment methods; `.cell()` gives the coordinate.
- **`ErrorFront`** (`plonk`) - frontend synthesis error type exported by `halo2_proofs::plonk`.

## `ConstraintSystem` methods

- **`meta.advice_column()`** -> `Column<Advice>` - allocate private witness column.
- **`meta.instance_column()`** -> `Column<Instance>` - allocate public-input column.
- **`meta.fixed_column()`** -> `Column<Fixed>` - allocate fixed column.
- **`meta.selector()`** -> `Selector` - allocate simple gate selector.
- **`meta.complex_selector()`** -> `Selector` - allocate selector usable in arbitrary expressions, including lookup/shuffle inputs.
- **`meta.enable_equality(col)`** - allow cells in `col` to participate in copy constraints.
- **`meta.create_gate("name", |meta| { ... })`** - register gate constraints.
- **`meta.query_advice(col, rot)`** -> `Expression<F>` - symbolic advice-cell query.
- **`meta.query_fixed(col, rot)`** -> `Expression<F>` - symbolic fixed-cell query.
- **`meta.query_instance(col, rot)`** -> `Expression<F>` - symbolic public-input query.
- **`meta.query_selector(sel)`** -> `Expression<F>` - symbolic selector query at current row.

## Layout methods

- **`layouter.assign_region(|| "name", |mut region| { ... })`** - create a region. Under `SimpleFloorPlanner`, closure runs once for shape and once for assignment.
- **`region.assign_advice(|| "name", col, offset, || value)`** -> `Result<AssignedCell<_, _>, _>` - assign advice cell.
- **`region.assign_fixed(|| "name", col, offset, || value)`** -> `Result<AssignedCell<_, _>, _>` - assign fixed cell.
- **`region.constrain_equal(cell_a, cell_b)`** -> `Result<(), _>` - copy-constrain two equality-enabled cells.
- **`selector.enable(&mut region, offset)`** -> `Result<(), _>` - set selector to 1 at offset.
- **`layouter.constrain_instance(cell, instance_col, row)`** -> `Result<(), _>` - constrain a cell to a public input.

## Constructors

- **`Value::known(f)`** - known witness value.
- **`Value::unknown()`** - no witness value yet.
- **`Expression::Constant(F::from(n))`** - constant term in a gate.
- **`Rotation::cur()` / `next()` / `prev()`** - row offsets 0 / +1 / -1.

## Testing

- **`MockProver<F>`** (`dev`) - development checker; no real proof.
- **`MockProver::run(k, &circuit, instances)`** - run checker with `2^k` rows and public inputs.
- **`prover.assert_satisfied()`** - panic unless all constraints pass.
- **`prover.verify()`** - return `Result`, useful for negative tests.
- **`prover.usable_rows()`** - rows available to the circuit, excluding blinding rows.
- **`prover.advice()` / `prover.selectors()`** - inspect filled columns for debugging.

## Concepts

- **Witness** - private inputs and intermediate advice values.
- **Gate** - polynomial constraint that must evaluate to zero.
- **Copy constraint** - equality constraint between arbitrary cells, enforced by permutation.
- **Permutation / sigma columns** - internal machinery for copy constraints, created from equality-enabled columns.
- **Blinding rows** - trailing rows used to hide witness data in commitments.
- **Keygen vs proving** - keygen records shape/fixed/selector data with unknown witnesses; proving assigns real witnesses and creates the proof.
