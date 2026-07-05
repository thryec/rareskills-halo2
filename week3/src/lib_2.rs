use ff::PrimeField;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, ErrorFront, Expression, Selector},
    poly::Rotation,
};

// range check for 0..15
// this needs 4 bit decomposition

pub struct MyCircuit<F: PrimeField> {
    x: Value<F>,
    b: [Value<F>; 4],
}

#[derive(Clone)]
pub struct CircuitConfig {
    x: Column<Advice>,
    b: [Column<Advice>; 4],
    q: Selector,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            x: Value::unknown(),
            b: [Value::unknown(); 4],
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let x = meta.advice_column();
        let b0 = meta.advice_column();
        let b1 = meta.advice_column();
        let b2 = meta.advice_column();
        let b3 = meta.advice_column();
        let q = meta.selector();

        // each b is a bit
        // X = b0 + 2b1 + 4b2 + 8b3
        meta.create_gate("range check", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let b0 = meta.query_advice(b0, Rotation::cur());
            let b1 = meta.query_advice(b1, Rotation::cur());
            let b2 = meta.query_advice(b2, Rotation::cur());
            let b3 = meta.query_advice(b3, Rotation::cur());
            let q = meta.query_selector(q);

            vec![
                q.clone()
                    * (b0.clone()
                        * (b0.clone() - Expression::Constant(F::ONE))
                        * (b0.clone() - Expression::Constant(F::from(2)))
                        * (b0.clone() - Expression::Constant(F::from(3)))),
                q.clone()
                    * (b1.clone()
                        * (b1.clone() - Expression::Constant(F::ONE))
                        * (b1.clone() - Expression::Constant(F::from(2)))
                        * (b1.clone() - Expression::Constant(F::from(3)))),
                q.clone()
                    * (b2.clone()
                        * (b2.clone() - Expression::Constant(F::ONE))
                        * (b2.clone() - Expression::Constant(F::from(2)))
                        * (b2.clone() - Expression::Constant(F::from(3)))),
                q.clone()
                    * (b3.clone()
                        * (b3.clone() - Expression::Constant(F::ONE))
                        * (b3.clone() - Expression::Constant(F::from(2)))
                        * (b3.clone() - Expression::Constant(F::from(3)))),
                q * (x
                    - (b0
                        + Expression::Constant(F::from(4)) * b1
                        + Expression::Constant(F::from(16)) * b2
                        + Expression::Constant(F::from(64)) * b3)),
            ]
            // if we change to base 4, we need to check that Bs can be 0,1,2,3
        });

        CircuitConfig {
            x,
            b: [b0, b1, b2, b3], // array of 4 columns
            q,
        }
    }

    // fills columns with whatever witness values i receive
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        layouter.assign_region(
            || "fill",
            |mut region| {
                region.assign_advice(|| "x", config.x, 0, || self.x)?;
                region.assign_advice(|| "b0", config.b[0], 0, || self.b[0])?;
                region.assign_advice(|| "b1", config.b[1], 0, || self.b[1])?;
                region.assign_advice(|| "b2", config.b[2], 0, || self.b[2])?;
                region.assign_advice(|| "b3", config.b[3], 0, || self.b[3])?;

                // enable selector
                config.q.enable(&mut region, 0)?;
                // alternative
                // region.enable_selector(|| "selector", &config.q, 0)?;

                Ok(())
            },
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::lib_2::MyCircuit;
    use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::bn256::Fr};

    #[test]
    fn test_range_check() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(64)), // base 4 - 64 , decomposition should be 0,0,0,1
            b: [
                Value::known(Fr::from(0)),
                Value::known(Fr::from(0)),
                Value::known(Fr::from(0)),
                Value::known(Fr::from(1)),
            ],
        };

        let prover = MockProver::run(4, &circuit, vec![]).unwrap(); // 2^4 = 16 rows, technically 16 rows
        prover.assert_satisfied();
    }
}
