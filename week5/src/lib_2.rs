//! Problem 2
//!
//! Create a circuit using a chip called `isZero`, which takes a field element as input and
//! outputs `1` if the element is zero and `0` otherwise.
//!
//! Note: An auxiliary input value is required for this circuit to operate.

use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, ErrorFront, Expression, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

pub struct MyCircuit<F: PrimeField> {
    x: Value<F>,
}

// x = 0  -> out = 1
// x != 0 -> out = 0

/// IS ZERO CHIP
#[derive(Clone)]
pub struct IsZeroConfig {
    x: Column<Advice>,
    x_inv: Column<Advice>,
    out: Column<Advice>,
    q: Selector,
}

pub struct IsZeroChip<F> {
    config: IsZeroConfig,
    _ph: PhantomData<F>,
}

impl<F: PrimeField> IsZeroChip<F> {
    pub fn construct(config: IsZeroConfig) -> Self {
        IsZeroChip {
            config,
            _ph: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> IsZeroConfig {
        let x = meta.advice_column();
        let x_inv = meta.advice_column();
        let out = meta.advice_column();
        let q = meta.selector();

        meta.enable_equality(x);
        meta.enable_equality(out);

        // x = 0  -> out = 1
        // x != 0 -> out = 0

        meta.create_gate("is zero gate", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let x_inv = meta.query_advice(x_inv, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());
            let q = meta.query_selector(q);
            let one = Expression::Constant(F::ONE);

            vec![
                q.clone() * x.clone() * out.clone(), // if x not 0, out has to be 0
                q * (one - out - x * x_inv),         // if x = 0, out has to be 1
            ]
        });

        IsZeroConfig { x, x_inv, out, q }
    }

    pub fn unconstrained(
        &self,
        layouter: &mut impl Layouter<F>,
        x: Value<F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "unconstrained",
            |mut region| {
                let x_cell = region.assign_advice(|| "unconstrained", config.x, 0, || x)?;
                Ok(x_cell)
            },
        )
    }

    pub fn is_zero(
        &self,
        layouter: &mut impl Layouter<F>,
        x: AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, ErrorFront> {
        let config = &self.config;

        layouter.assign_region(
            || "is zero region",
            |mut region| {
                config.q.enable(&mut region, 0)?;
                let x_val = x.value().copied();
                let x_inv_val = x_val.map(|x| x.invert().unwrap_or(F::ZERO));
                let out_val = x_val.map(|x| if x == F::ZERO { F::ONE } else { F::ZERO });

                let new_x = region.assign_advice(|| "x", config.x, 0, || x_val)?;
                region.constrain_equal(new_x.cell(), x.cell())?;
                region.assign_advice(|| "x_inv", config.x_inv, 0, || x_inv_val)?;
                let out = region.assign_advice(|| "out", config.out, 0, || out_val)?;

                Ok(out)
            },
        )
    }
}

/// MAIN CIRCUIT

#[derive(Clone)]
pub struct CircuitConfig {
    is_zero_config: IsZeroConfig,
    instance: Column<Instance>,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        MyCircuit {
            x: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let instance = meta.instance_column();
        meta.enable_equality(instance);

        let is_zero_config = IsZeroChip::configure(meta);
        CircuitConfig {
            is_zero_config,
            instance,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        let chip = IsZeroChip::construct(config.is_zero_config);
        let x_cell = chip.unconstrained(&mut layouter, self.x)?;

        let out_cell = chip.is_zero(&mut layouter, x_cell)?;
        layouter.constrain_instance(out_cell.cell(), config.instance, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

    #[test]
    fn returns_one_when_x_is_zero() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(0)),
        };

        let prover = MockProver::run(4, &circuit, vec![vec![Fr::from(1)]]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn returns_zero_when_x_is_nonzero() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(5)),
        };

        let prover = MockProver::run(4, &circuit, vec![vec![Fr::from(0)]]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn rejects_wrong_public_output() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(5)),
        };

        let prover = MockProver::run(4, &circuit, vec![vec![Fr::from(1)]]).unwrap();
        assert!(prover.verify().is_err());
    }
}
