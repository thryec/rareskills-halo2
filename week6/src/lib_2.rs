//! Problem 2
//!
//! Create a 16-bit range table to perform the addition of two 16-bit numbers.
//! However, the gate cannot simply be `x + y - z`, because there is a possibility
//! of overflow that we need to handle.
//!
//! One way to solve this is by using a carry bit, which should be part of the Advice.
//! Try this approach.
//!
//! **Challenge**: Another possibility is to decompose the number into low and
//! high parts—for example, 8 bits each (or even more parts for numbers with more bits)
//! Can you figure out how to perform addition this way?
//! Try to design a circuit to add 32-bit numbers using smaller chunks—such as 16-bit or
//! 8-bit chunks—while accounting for potential overflow.
//!
//! *Note: Both problems require only static lookup tables.*

use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, ErrorFront, Expression, Selector, TableColumn,
};
use halo2_proofs::poly::Rotation;

pub struct Add16Circuit<F: PrimeField> {
    x: Value<F>,
    y: Value<F>,
    z: Value<F>,
    carry: Value<F>,
}

#[derive(Clone)]
pub struct Add16Config {
    x: Column<Advice>,
    y: Column<Advice>,
    z: Column<Advice>,
    carry: Column<Advice>,
    range: TableColumn,
    q: Selector,
}

impl<F: PrimeField> Circuit<F> for Add16Circuit<F> {
    type Config = Add16Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Add16Circuit {
            x: Value::unknown(),
            y: Value::unknown(),
            z: Value::unknown(),
            carry: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let z = meta.advice_column();
        let carry = meta.advice_column();
        let range = meta.lookup_table_column();

        let q = meta.complex_selector();

        meta.create_gate("addition with carry", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let y = meta.query_advice(y, Rotation::cur());
            let z = meta.query_advice(z, Rotation::cur());
            let carry = meta.query_advice(carry, Rotation::cur());

            let q = meta.query_selector(q);

            let base = Expression::Constant(F::from(1_u64 << 16));
            let one = Expression::Constant(F::ONE);

            let addition = x + y - z - base * carry.clone();

            vec![
                q.clone() * addition,
                q.clone() * (carry.clone() * (carry - one)),
            ]
        });

        meta.lookup("x range check", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let q = meta.query_selector(q);

            vec![(q * x, range)]
        });

        meta.lookup("y range check", |meta| {
            let y = meta.query_advice(y, Rotation::cur());
            let q = meta.query_selector(q);

            vec![(q * y, range)]
        });

        meta.lookup("z range check", |meta| {
            let z = meta.query_advice(z, Rotation::cur());
            let q = meta.query_selector(q);

            vec![(q * z, range)]
        });

        Add16Config {
            x,
            y,
            z,
            range,
            carry,
            q,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        layouter.assign_table(
            || "range table",
            |mut table| {
                for value in 0_u64..(1 << 16) {
                    table.assign_cell(
                        || "range value",
                        config.range,
                        value as usize,
                        || Value::known(F::from(value)),
                    )?;
                }
                Ok(())
            },
        )?;

        layouter.assign_region(
            || "values",
            |mut region| {
                region.assign_advice(|| "x value", config.x, 0, || self.x)?;
                region.assign_advice(|| "y value", config.y, 0, || self.y)?;
                region.assign_advice(|| "z value", config.z, 0, || self.z)?;
                region.assign_advice(|| "carry value", config.carry, 0, || self.carry)?;

                config.q.enable(&mut region, 0)?;
                Ok(())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Add16Circuit;
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr;

    #[test]
    fn it_works() {
        let circuit = Add16Circuit {
            x: Value::known(Fr::from(1000)),
            y: Value::known(Fr::from(2000)),
            z: Value::known(Fr::from(3000)),
            carry: Value::known(Fr::from(0)),
        };

        let prover = MockProver::run(18, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn fails_with_wrong_carry() {
        let circuit = Add16Circuit {
            x: Value::known(Fr::from(1000)),
            y: Value::known(Fr::from(2000)),
            z: Value::known(Fr::from(3000)),
            carry: Value::known(Fr::from(1)),
        };

        let prover = MockProver::run(18, &circuit, vec![]).unwrap();
        assert!(prover.verify().is_err());
    }

    #[test]
    fn works_with_overflow() {
        let base = 1_u64 << 16; // 2^16 
        let z = 70000 - base;

        let circuit = Add16Circuit {
            x: Value::known(Fr::from(50000)),
            y: Value::known(Fr::from(20000)),
            z: Value::known(Fr::from(z)),
            carry: Value::known(Fr::from(1)),
        };

        let prover = MockProver::run(18, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
