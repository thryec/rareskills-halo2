//! Problem 1
//!
//! Design a circuit using an arithmetic chip that performs both addition and multiplication.
//! Combine both operations into a single chip. Define the columns `a`, `b`, and `c` at the
//! circuit level so they can be shared with other chips.
//!

use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, ErrorFront, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

pub struct MyCircuit<F: PrimeField> {
    x: Value<F>,
    y: Value<F>,
    p: Value<F>,
}

/// ADD / MUL CHIP
#[derive(Clone)]
pub struct AddMulConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    q_add: Selector,
    q_mul: Selector,
}

pub struct AddMulChip<F> {
    config: AddMulConfig,
    _ph: PhantomData<F>,
}

impl<F: PrimeField> AddMulChip<F> {
    pub fn construct(config: AddMulConfig) -> Self {
        AddMulChip {
            config,
            _ph: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        a: Column<Advice>,
        b: Column<Advice>,
        c: Column<Advice>,
    ) -> AddMulConfig {
        let q_add = meta.selector();
        let q_mul = meta.selector();

        meta.create_gate("add gate", |meta| {
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());

            let q_add = meta.query_selector(q_add);

            vec![q_add * (a + b - c)]
        });

        meta.create_gate("mul gate", |meta| {
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());

            let q_mul = meta.query_selector(q_mul);

            vec![q_mul * (a * b - c)]
        });

        AddMulConfig {
            a,
            b,
            c,
            q_add,
            q_mul,
        }
    }

    // turns raw value into assigned cell
    pub fn unconstrained(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Value<F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "unconstrained",
            |mut region| {
                let a_cell = region.assign_advice(|| "unconstrained", config.a, 0, || a)?;

                Ok(a_cell)
            },
        )
    }

    pub fn add(
        &self,
        layouter: &mut impl Layouter<F>,
        a: AssignedCell<F, F>,
        b: AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "add region",
            |mut region| {
                config.q_add.enable(&mut region, 0)?;

                let a_val = a.value().copied();
                let b_val = b.value().copied();

                let new_a = region.assign_advice(|| "a", config.a, 0, || a_val)?;
                let new_b = region.assign_advice(|| "b", config.b, 0, || b_val)?;

                region.constrain_equal(new_a.cell(), a.cell())?;
                region.constrain_equal(new_b.cell(), b.cell())?;

                let out_val = a_val + b_val;
                let out = region.assign_advice(|| "c", config.c, 0, || out_val)?;

                Ok(out)
            },
        )
    }

    pub fn mul(
        &self,
        layouter: &mut impl Layouter<F>,
        a: AssignedCell<F, F>,
        b: AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "mul region",
            |mut region| {
                config.q_mul.enable(&mut region, 0)?;

                let a_val = a.value().copied();
                let b_val = b.value().copied();

                let new_a = region.assign_advice(|| "a", config.a, 0, || a_val)?;
                let new_b = region.assign_advice(|| "b", config.b, 0, || b_val)?;

                region.constrain_equal(new_a.cell(), a.cell())?;
                region.constrain_equal(new_b.cell(), b.cell())?;

                let out_val = a_val * b_val;
                let out = region.assign_advice(|| "c", config.c, 0, || out_val)?;

                Ok(out)
            },
        )
    }
}

/// MAIN CIRCUIT

#[derive(Clone)]
pub struct CircuitConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    config: AddMulConfig,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        MyCircuit {
            x: Value::unknown(),
            y: Value::unknown(),
            p: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);

        let config = AddMulChip::configure(meta, a, b, c);
        CircuitConfig { a, b, c, config }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        let add_mul_chip: AddMulChip<F> = AddMulChip::construct(config.config);

        let x_cell = add_mul_chip.unconstrained(&mut layouter, self.x)?;
        let y_cell = add_mul_chip.unconstrained(&mut layouter, self.y)?;

        let z_cell = add_mul_chip.add(&mut layouter, x_cell, y_cell)?;

        let p_cell = add_mul_chip.unconstrained(&mut layouter, self.p)?;

        let _q_cell = add_mul_chip.mul(&mut layouter, z_cell, p_cell)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

    #[test]
    fn it_works() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(1)),
            y: Value::known(Fr::from(2)),
            p: Value::known(Fr::from(4)),
        };

        let prover = MockProver::run(4, &circuit, vec![]).unwrap();
        prover.assert_satisfied();

        let advices = prover.advice();
        let selectors = prover.selectors();

        for i in 0..advices[0].len() {
            println!(
                "a[{i}]={:?}, b[{i}]={:?}, c[{i}]={:?}",
                advices[0][i], advices[1][i], advices[2][i]
            );
        }

        for i in 0..selectors[0].len() {
            println!(
                "q_add[{i}]={:?}, q_mul[{i}]={:?}",
                selectors[0][i], selectors[1][i]
            );
        }
    }
}
