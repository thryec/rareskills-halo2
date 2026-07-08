//! Problem 4
//!
//! Implement a chip that computes the bitwise XOR of two unsigned integers of up to 32 bits.
//! Be sure to constrain both inputs to the 32-bit range.

// 1. break down inputs into 32 bits
// 2. make sure each bit is either 0 or 1
// 3. compute output bits with XOR
// 4. rebuild x, y, output from bits
// 5. make sure x and y equal to their rebuilt 32-bit values

use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, ErrorFront, Expression, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

//  bit decomposition and calculation helper functions

const NUM_BITS: usize = 32;

fn decompose_to_bits<F: PrimeField>(value: u64) -> [Value<F>; NUM_BITS] {
    let mut bits = [Value::known(F::ZERO); NUM_BITS];

    for i in 0..NUM_BITS {
        let bit = (value >> i) & 1;
        bits[i] = Value::known(F::from(bit));
    }
    bits
}

fn compute_xor(x: u64, y: u64) -> u64 {
    (x as u32 ^ y as u32) as u64
}

pub struct MyCircuit<F: PrimeField> {
    x: Value<F>,
    y: Value<F>,
    x_bits: [Value<F>; NUM_BITS],
    y_bits: [Value<F>; NUM_BITS],
}

/// Bitwise XOR CHIP
#[derive(Clone)]
pub struct XorConfig {
    x: Column<Advice>,
    y: Column<Advice>,
    out: Column<Advice>,
    x_bits: [Column<Advice>; NUM_BITS],
    y_bits: [Column<Advice>; NUM_BITS],
    out_bits: [Column<Advice>; NUM_BITS],
    q: Selector,
}

pub struct XorChip<F> {
    config: XorConfig,
    _ph: PhantomData<F>,
}

impl<F: PrimeField> XorChip<F> {
    pub fn construct(config: XorConfig) -> Self {
        XorChip {
            config,
            _ph: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> XorConfig {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let out = meta.advice_column();
        let q = meta.selector();

        let x_bits: [Column<Advice>; NUM_BITS] = std::array::from_fn(|_| meta.advice_column());
        let y_bits: [Column<Advice>; NUM_BITS] = std::array::from_fn(|_| meta.advice_column());
        let out_bits: [Column<Advice>; NUM_BITS] = std::array::from_fn(|_| meta.advice_column());

        // ensure bits are either 0 or 1
        meta.create_gate("boolean gate", |meta| {
            let x_bits = x_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let y_bits = y_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let out_bits = out_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let q = meta.query_selector(q);

            let one = Expression::Constant(F::ONE);

            let mut constraints = Vec::new();

            // ensure bits are either 0 or 1
            for i in 0..NUM_BITS {
                constraints.push(q.clone() * x_bits[i].clone() * (x_bits[i].clone() - one.clone()));
                constraints.push(q.clone() * y_bits[i].clone() * (y_bits[i].clone() - one.clone()));
                constraints
                    .push(q.clone() * out_bits[i].clone() * (out_bits[i].clone() - one.clone()));
            }

            constraints
        });

        // if x bit and y bit different, then out bit is 1
        // if x bit and y bit same, then out bit is 0
        //  out = x + y - 2xy
        meta.create_gate("xor gate", |meta| {
            let q = meta.query_selector(q);

            let x_bits = x_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let y_bits = y_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let out_bits = out_bits.map(|col| meta.query_advice(col, Rotation::cur()));

            let two = Expression::Constant(F::from(2));

            let mut constraints = Vec::new();

            for i in 0..NUM_BITS {
                constraints.push(
                    q.clone()
                        * (x_bits[i].clone() + y_bits[i].clone()
                            - two.clone() * x_bits[i].clone() * y_bits[i].clone()
                            - out_bits[i].clone()),
                );
            }

            constraints
        });

        XorConfig {
            x,
            y,
            out,
            x_bits,
            y_bits,
            out_bits,
            q,
        }
    }

    pub fn unconstrained(
        &self,
        layouter: &mut impl Layouter<F>,
        x: Value<F>,
        y: Value<F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let cell = layouter.assign_region(
            || "x",
            |mut region| region.assign_advice(|| "x", self.config.x, 0, x),
        )?;
    }

    pub fn xor(
        &self,
        layouter: &mut impl Layouter<F>,
        x: AssignedCell<F, F>,
        y: AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let cell = layouter.assign_region(
            || "xor",
            |mut region| region.assign_advice(|| "xor", self.config.x, 0, x),
        )?;
    }
}

/// MAIN CIRCUIT

#[derive(Clone)]
pub struct CircuitConfig {
    instance: Column<Instance>,
    config: XorConfig,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        MyCircuit {
            x: Value::unknown(),
            y: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();
    }

    fn synthesize() {}
}
