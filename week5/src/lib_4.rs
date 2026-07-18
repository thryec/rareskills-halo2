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

#[cfg(test)]
fn decompose_to_bits<F: PrimeField>(value: u64) -> [Value<F>; NUM_BITS] {
    let mut bits = [Value::known(F::ZERO); NUM_BITS];

    for i in 0..NUM_BITS {
        let bit = (value >> i) & 1;
        bits[i] = Value::known(F::from(bit));
    }
    bits
}

#[cfg(test)]
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

        // Reconstruct each integer from its little-endian bits. Together with
        // the boolean constraints, this also range-checks x, y, and out.
        meta.create_gate("32-bit reconstruction gate", |meta| {
            let q = meta.query_selector(q);
            let x = meta.query_advice(x, Rotation::cur());
            let y = meta.query_advice(y, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());
            let x_bits = x_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let y_bits = y_bits.map(|col| meta.query_advice(col, Rotation::cur()));
            let out_bits = out_bits.map(|col| meta.query_advice(col, Rotation::cur()));

            let mut x_reconstructed = Expression::Constant(F::ZERO);
            let mut y_reconstructed = Expression::Constant(F::ZERO);
            let mut out_reconstructed = Expression::Constant(F::ZERO);
            let mut coefficient = F::ONE;

            for i in 0..NUM_BITS {
                let coefficient_expr = Expression::Constant(coefficient);
                x_reconstructed = x_reconstructed + coefficient_expr.clone() * x_bits[i].clone();
                y_reconstructed = y_reconstructed + coefficient_expr.clone() * y_bits[i].clone();
                out_reconstructed = out_reconstructed + coefficient_expr * out_bits[i].clone();
                coefficient = coefficient.double();
            }

            vec![
                q.clone() * (x - x_reconstructed),
                q.clone() * (y - y_reconstructed),
                q * (out - out_reconstructed),
            ]
        });

        meta.enable_equality(x);
        meta.enable_equality(y);
        meta.enable_equality(out);

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
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "unconstrained inputs",
            |mut region| {
                let x_cell = region.assign_advice(|| "x", config.x, 0, || x)?;
                let y_cell = region.assign_advice(|| "y", config.y, 0, || y)?;

                Ok((x_cell, y_cell))
            },
        )
    }

    pub fn xor(
        &self,
        layouter: &mut impl Layouter<F>,
        x: AssignedCell<F, F>,
        y: AssignedCell<F, F>,
        x_bits: [Value<F>; NUM_BITS],
        y_bits: [Value<F>; NUM_BITS],
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "xor",
            |mut region| {
                config.q.enable(&mut region, 0)?;

                let x_copy = region.assign_advice(|| "x", config.x, 0, || x.value().copied())?;
                let y_copy = region.assign_advice(|| "y", config.y, 0, || y.value().copied())?;
                region.constrain_equal(x_copy.cell(), x.cell())?;
                region.constrain_equal(y_copy.cell(), y.cell())?;

                let two = F::from(2);
                let out_bits: [Value<F>; NUM_BITS] = std::array::from_fn(|i| {
                    x_bits[i]
                        .zip(y_bits[i])
                        .map(|(x_bit, y_bit)| x_bit + y_bit - two * x_bit * y_bit)
                });

                let mut out_value = Value::known(F::ZERO);
                let mut coefficient = F::ONE;

                for i in 0..NUM_BITS {
                    region.assign_advice(|| "x bit", config.x_bits[i], 0, || x_bits[i])?;
                    region.assign_advice(|| "y bit", config.y_bits[i], 0, || y_bits[i])?;
                    region.assign_advice(|| "out bit", config.out_bits[i], 0, || out_bits[i])?;

                    out_value = out_value + out_bits[i].map(|bit| coefficient * bit);
                    coefficient = coefficient.double();
                }

                region.assign_advice(|| "out", config.out, 0, || out_value)
            },
        )
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
            x_bits: [Value::unknown(); NUM_BITS],
            y_bits: [Value::unknown(); NUM_BITS],
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let instance = meta.instance_column();
        meta.enable_equality(instance);

        let config = XorChip::configure(meta);
        CircuitConfig { instance, config }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        let chip = XorChip::construct(config.config);
        let (x, y) = chip.unconstrained(&mut layouter, self.x, self.y)?;
        let out = chip.xor(&mut layouter, x, y, self.x_bits, self.y_bits)?;

        layouter.constrain_instance(out.cell(), config.instance, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

    fn circuit(x: u64, y: u64) -> MyCircuit<Fr> {
        MyCircuit {
            x: Value::known(Fr::from(x)),
            y: Value::known(Fr::from(y)),
            x_bits: decompose_to_bits(x),
            y_bits: decompose_to_bits(y),
        }
    }

    fn assert_valid_xor(x: u64, y: u64) {
        let prover =
            MockProver::run(4, &circuit(x, y), vec![vec![Fr::from(compute_xor(x, y))]]).unwrap();

        prover.assert_satisfied();
    }

    #[test]
    fn computes_bitwise_xor() {
        assert_valid_xor(0b1010, 0b1100);
        assert_valid_xor(0, 0);
        assert_valid_xor(u32::MAX as u64, 0);
        assert_valid_xor(u32::MAX as u64, u32::MAX as u64);
    }

    #[test]
    fn rejects_wrong_public_output() {
        let prover = MockProver::run(4, &circuit(10, 12), vec![vec![Fr::from(7)]]).unwrap();

        assert!(prover.verify().is_err());
    }

    #[test]
    fn rejects_inputs_larger_than_32_bits() {
        let too_large = 1_u64 << NUM_BITS;

        let x_prover = MockProver::run(
            4,
            &circuit(too_large, 1),
            vec![vec![Fr::from(compute_xor(too_large, 1))]],
        )
        .unwrap();
        assert!(x_prover.verify().is_err());

        let y_prover = MockProver::run(
            4,
            &circuit(1, too_large),
            vec![vec![Fr::from(compute_xor(1, too_large))]],
        )
        .unwrap();
        assert!(y_prover.verify().is_err());
    }
}
