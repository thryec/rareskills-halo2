# Halo2 Reference

Concise map of a halo2 circuit: table layout, lifecycle, gates, copy constraints, and testing.

## Mental model

A halo2 circuit is a 2D table of field elements. With parameter `k`, the table has `2^k` physical rows; only a prefix is usable because trailing rows are reserved for blinding. Columns are typed. Gates are polynomial equations that must equal zero on selected rows. A selector is a fixed 0/1 switch that turns a gate on per row.

Proving means showing knowledge of private advice values that fill this table so all gates and copy constraints hold. The verifier sees the verifying key, proof, and public inputs, not private advice values.

## Lifecycle

1. **Configure**: declare columns, equality-enabled columns, gates, lookups, and fixed circuit shape. No witness values.
2. **Synthesize**: assign cells, enable selectors, and add copy constraints. Runs at keygen with unknown witnesses, then at proving with real witnesses.

`without_witnesses` returns the same shape with witness values set to `Value::unknown()`. During keygen, advice values are ignored; fixed cells and selector layout are recorded in the keys.

## Column types

| Type | Filled by | When | Public? |
|------|-----------|------|---------|
| `Advice` | prover | proving | no, only committed |
| `Instance` | prover + verifier | proving + verify | yes, public inputs |
| `Fixed` | circuit author | keygen | yes, in the key |
| `Selector` | circuit author | keygen | yes, fixed 0/1 gate switch |

Private data can live in advice. Public claims or trusted constants must be instance/fixed, or advice cells constrained to instance/fixed cells.

## Field types

Halo2's `Circuit<F>` trait requires `F: Field`. This repo uses the stronger `F: PrimeField`, common in examples that use `F::from(u64)`. Tests instantiate `F` with `Fr`, the BN254 scalar field.

`PrimeField` comes from the `ff` crate. Common circuit items (`Value`, `Layouter`, `SimpleFloorPlanner`, `Column`, `Advice`, `Selector`, `ConstraintSystem`, `Rotation`, `Circuit`, `AssignedCell`, `ErrorFront`) come from `halo2_proofs`.

## Witness vs config

- The circuit struct holds witness inputs, usually as `Value<F>` fields. `Value::known(f)` is a real witness; `Value::unknown()` is used when witness values are unavailable.
- `Value` prevents ordinary branching on witnesses. Circuit shape must not depend on private values.
- The config struct holds column handles from `configure` so `synthesize` can assign into those columns.
- `Config` must be `Clone` because `Circuit` requires `type Config: Clone`.

`self.a` and `config.a` are different:

- `self.a` is witness data stored in `MyCircuit`, such as `[1, 1, 2, 3, 5]`.
- `config.a` is the advice-column handle created in `configure`.

The witness reaches `synthesize` when the whole circuit is passed to Halo2:

```rust
let circuit = MyCircuit { a: witness_values };
MockProver::run(k, &circuit, instances)?;
```

Halo2 later calls `circuit.synthesize(config, layouter)`. Inside that method, `&self` is the `circuit` value, so `self.a[0]` means "first witness value from this circuit." A line like `region.assign_advice(..., config.a, 0, || self.a[0])` means "write witness value `self.a[0]` into advice column `config.a` at row 0."

## `Circuit<F>` trait

Required pieces:

- `type Config`: column-handle struct.
- `type FloorPlanner`: region placement strategy; `SimpleFloorPlanner` is fine for small examples.
- `without_witnesses`: same shape, unknown witnesses.
- `configure`: declare circuit shape.
- `synthesize`: fill cells and wire constraints.

## `configure`

`ConstraintSystem<F>` is the registry for circuit shape:

- `meta.advice_column()`, `meta.instance_column()`, `meta.fixed_column()` allocate columns.
- `meta.selector()` creates a simple gate switch. Use `meta.complex_selector()` when the selector appears in arbitrary expressions, such as lookup/shuffle inputs.
- `meta.enable_equality(col)` lets cells in that column participate in copy constraints.
- `meta.create_gate("name", |meta| {...})` registers constraints. Each returned `Expression<F>` must equal zero.

Queries inside a gate are symbolic, not values:

```rust
let a = meta.query_advice(a, Rotation::cur());
let b = meta.query_advice(b, Rotation::cur());
let c = meta.query_advice(c, Rotation::cur());
let q = meta.query_selector(q);
vec![q * (a + b - c)]
```

`Rotation::cur()` is same row, `next()` is row + 1, `prev()` is row - 1.

Gates live in `configure` because they are the statement being proven. Assignments in `synthesize` are only witness claims.

## `synthesize`

`Layouter` allocates regions. Region-relative offsets are used while the floor planner chooses absolute rows.

- `layouter.assign_region(...)` creates one region. With `SimpleFloorPlanner`, the closure runs once for shape and once for assignment.
- `region.assign_advice(...)` writes an advice cell and returns an `AssignedCell`.
- `selector.enable(...)` sets the selector to 1 at an offset.
- `region.constrain_equal(...)` copy-constrains two cells.
- `layouter.constrain_instance(...)` constrains a cell to a public input.
- `ErrorFront` is the frontend synthesis error type in `halo2_proofs::plonk` for PSE v0.4.0.

All these return `Result`; propagate with `?`.

## Copy constraints

Gates relate queried cells by row/rotation. To assert that two arbitrary cells are equal, use a copy constraint backed by the permutation argument.

1. In `configure`, call `meta.enable_equality(col)` for every copied column.
2. In `synthesize`, keep `AssignedCell` handles and call `region.constrain_equal(left.cell(), right.cell())`.

For public inputs, equality-enable both the advice column and instance column, then use `layouter.constrain_instance(...)`.

## Chips

A chip is a Rust abstraction for a reusable part of a circuit. Halo2 does not require a `Chip` trait in these examples; the pattern is just ordinary Rust structs and methods.

Typical chip pieces:

- `ChipConfig`: columns and selectors used by that chip.
- `Chip::configure(...)`: defines the chip's gates and returns its config.
- `Chip::construct(config)`: stores the config inside a chip object.
- `chip.assign(...)`, `chip.add(...)`, `chip.mul(...)`: assign rows, enable selectors, add copy constraints, and return output cells.

Main circuit still owns the full circuit lifecycle:

```text
main Circuit::configure
  -> allocate shared columns
  -> enable equality on copied columns
  -> call Chip::configure(...)

main Circuit::synthesize
  -> construct chip from config
  -> call chip methods in computation order
```

In lecture 5, `x`, `y`, and `p` are witness values stored in `MyCircuit`. The chip columns `a`, `b`, and `c` are reusable table slots.

For the computation:

```text
z = x + y
q = z * p
```

the conceptual table is:

```text
region              col a      col b      col c      selector
--------------------------------------------------------------
unconstrained x     x
unconstrained y     y

add row             x copy     y copy     z          q_add = 1

unconstrained p     p

mul row             z copy     p copy     q          q_mul = 1
```

With concrete values `x = 1`, `y = 2`, `p = 4`:

```text
region              col a      col b      col c
------------------------------------------------
unconstrained x     1
unconstrained y     2

add row             1          2          3

unconstrained p     4

mul row             3          4          12
```

The arrows are copy constraints:

```text
x cell  ------> add row col a
y cell  ------> add row col b
z cell  ------> mul row col a
p cell  ------> mul row col b
```

Gates only check the current row:

```text
add row: a + b = c
mul row: a * b = c
```

So chip assignment methods usually copy input `AssignedCell`s into a fresh operation row, enable that chip's selector, assign the output, and return the output `AssignedCell`.

`unconstrained` is a common helper name for "load a raw witness value into some advice cell without enabling an operation gate yet." It converts:

```text
Value<F> -> AssignedCell<F, F>
```

The loaded cell can then be copied into later add/mul rows.

## Lookup tables

A lookup proves that an advice value or tuple appears in a precomputed table. The prover supplies the values; the lookup checks membership in the allowed set or relation.

Lookups have their own constraint primitive because set membership is expensive to express with low-degree gates. For example, proving `x` is in `0..=15` directly would require the degree-16 expression `x(x - 1)...(x - 15) = 0`, or many smaller constraints. `meta.lookup` instead tells the backend to use a lookup argument that checks membership without revealing the advice value.

In short: a gate proves that values satisfy an equation; a lookup proves that values form an allowed tuple. This makes lookups useful for range checks, bitwise operations, addition with carry, and state transitions.

Declare static table columns and the lookup in `configure`:

```rust
let x_table = meta.lookup_table_column();
let y_table = meta.lookup_table_column();
let z_table = meta.lookup_table_column();
let q = meta.complex_selector();

meta.lookup("relation", |meta| {
    let x = meta.query_advice(x, Rotation::cur());
    let y = meta.query_advice(y, Rotation::cur());
    let z = meta.query_advice(z, Rotation::cur());
    let q = meta.query_selector(q);

    vec![
        (q.clone() * x, x_table),
        (q.clone() * y, y_table),
        (q * z, z_table),
    ]
});
```

The vector is one tuple lookup: `(x, y, z)` must match the same table row. It is not three independent checks. When `q = 0`, the input becomes the zero tuple, so static tables normally include a zero row.

Fill static tables in `synthesize` with `layouter.assign_table(...)` and `table.assign_cell(...)`. Examples:

- A one-column table containing `0..=15` proves a 4-bit range check.
- A table containing `(x, y, (x + y) mod 16)` proves modular addition.
- A table containing `(x, y, x XOR y)` proves XOR.

Use `meta.lookup` with `TableColumn` for static tables. `lookup_any` can use advice expressions on the table side, making table contents prover-supplied; those contents need their own constraints.

Table size matters. An `n`-bit range table needs `2^n` rows, while a full two-input operation table needs `2^(2n)` rows. An 8-bit XOR table needs 65,536 rows; a 16-bit table needs about 4.3 billion. For larger values, split inputs into smaller chunks and perform several lookups.

## Fibonacci chain

Gate: `q * (a + b - c) = 0`.

| row | a | b | c |
|-----|---|---|---|
| 0 | 1 | 1 | 2 |
| 1 | 1 | 2 | 3 |
| 2 | 2 | 3 | 5 |
| 3 | 3 | 5 | 8 |
| 4 | 5 | 8 | 13 |

The gate only checks each row. The sequence comes from copy constraints after the first row:

- `a[i] == b[i - 1]`
- `b[i] == c[i - 1]`

Keep previous-row `AssignedCell`s, pass `.cell()` to `constrain_equal`, and use `.as_ref().unwrap()` to borrow a saved `Option<AssignedCell>` without moving it.

## Testing

- `MockProver::run(k, &circuit, instances)` fills the table and checks constraints directly; it does not create a proof.
- `assert_satisfied()` panics with gate/region/row info on failure.
- `verify()` returns `Result`, useful for negative tests.
- `usable_rows()` is smaller than `2^k` because trailing rows are reserved for blinding.

## Appendix: key generation

In Halo2, key generation does not mean private keys or key-value pairs. It means generating a proving key and verifying key for a specific circuit shape.

The high-level algorithm is:

1. Create polynomial commitment parameters, such as KZG parameters, for table size `2^k`.
2. Compile the circuit by running `configure()` and a keygen-time `synthesize()` with unknown witnesses.
3. Record fixed circuit data: columns, gates, selector layouts, fixed columns, copy-constraint permutation data, and domain information.
4. Convert fixed columns, selectors, and permutation data into polynomials over the evaluation domain.
5. Commit to verifier-needed fixed/permutation polynomials. These commitments go into the verifying key.
6. Store prover-needed full polynomial data, cosets, helper polynomials, and evaluator data. These go into the proving key.

For KZG setup, the parameters conceptually contain elliptic-curve powers:

```text
G, tau * G, tau^2 * G, ..., tau^n * G
```

The secret `tau` comes from the trusted setup and must not be known by provers or verifiers.

In code, the flow looks like:

```rust
let params = ParamsKZG::<Bn256>::setup(k, OsRng);
let dummy_circuit = MyCircuit::default().without_witnesses();

let vk = keygen_vk(&params, &dummy_circuit)?;
let pk = keygen_pk(&params, vk.clone(), &dummy_circuit)?;
```

`keygen_vk` builds the verifying key from circuit shape and commitments to fixed circuit polynomials. `keygen_pk` builds the proving key from the verifying key plus full prover-side fixed/permutation polynomial data.

Keys depend on:

- columns
- gates
- selectors and where they are enabled
- fixed columns
- copy/equality configuration
- supported row count `k`

Keys do not depend on:

- private advice values
- public instance values for one proof
- Fibonacci witness rows for one proof
- flag/counter witness rows for one proof

So changing `configure()` usually requires new keys. Changing only witness or instance values uses the same keys.
