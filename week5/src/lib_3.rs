//! Problem 3
//!
//! Use the circuit from Problem 1 to prove that the prover knows a value `a` such that
//! `a^5 + a = b`, where `b` is a public input.
//!
//! Although you could implement this using a custom gate, such a solution would not be generic.
//! Instead, use the arithmetic chip to construct the circuit.

use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, ErrorFront, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

// prover's private witness values
pub struct MyCircuit<F: PrimeField> {
    a: Value<F>,
}

// ADD / MUL CHIPs
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
/// calculates a^5 + a = b

#[derive(Clone)]
pub struct CircuitConfig {
    instance: Column<Instance>,
    config: AddMulConfig,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        MyCircuit {
            a: Value::unknown(),
        }
    }

    // creates columns to pass into the chip
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let instance = meta.instance_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(instance);

        let config = AddMulChip::configure(meta, a, b, c);
        CircuitConfig { instance, config }
    }

    // fills the circuit table with actual witness values and connects cells
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        let chip: AddMulChip<F> = AddMulChip::construct(config.config);
        let a = chip.unconstrained(&mut layouter, self.a)?;
        let a2 = chip.mul(&mut layouter, a.clone(), a.clone())?;
        let a4 = chip.mul(&mut layouter, a2.clone(), a2.clone())?;
        let a5 = chip.mul(&mut layouter, a4.clone(), a.clone())?;

        let out = chip.add(&mut layouter, a5.clone(), a.clone())?;

        layouter.constrain_instance(out.cell(), config.instance, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

    fn expected_b(a: Fr) -> Fr {
        let a2 = a * a;
        let a4 = a2 * a2;
        let a5 = a4 * a;

        a5 + a
    }

    #[test]
    fn accepts_valid_public_output() {
        let a = Fr::from(2);
        let circuit = MyCircuit {
            a: Value::known(a),
        };

        let prover = MockProver::run(5, &circuit, vec![vec![expected_b(a)]]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn accepts_zero_witness() {
        let a = Fr::from(0);
        let circuit = MyCircuit {
            a: Value::known(a),
        };

        let prover = MockProver::run(5, &circuit, vec![vec![expected_b(a)]]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn rejects_wrong_public_output() {
        let a = Fr::from(2);
        let circuit = MyCircuit {
            a: Value::known(a),
        };

        let wrong_b = expected_b(a) + Fr::from(1);
        let prover = MockProver::run(5, &circuit, vec![vec![wrong_b]]).unwrap();

        assert!(prover.verify().is_err());
    }
}
