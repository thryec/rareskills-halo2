use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Advice, Circuit, Column, Expression, Selector};
use halo2_proofs::plonk::{ConstraintSystem, ErrorFront};
use halo2_proofs::poly::Rotation;
// we need to implement the circuit trait in some struct

pub struct MyCircuit<F: PrimeField> {
    pub x: Value<F>, // Value type is kinda like option in Rust but not exactly
    pub y: Value<F>,
}

#[derive(Clone)]
pub struct AddConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub q: Selector,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = AddConfig; // struct with the configuration of my table
    type FloorPlanner = SimpleFloorPlanner; // related with the regions we will use the SimpleFloorPlanner

    fn without_witnesses(&self) -> Self {
        MyCircuit {
            x: Value::unknown(),
            y: Value::unknown(),
        }
    }

    // where we define the custom gates and lookup arguments
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // a + b = 10
        let a = meta.advice_column();
        let b = meta.advice_column();
        let q = meta.selector(); // complex_selector() can also be used, but in lookup tables or other unique situations we have to use complex_selector()

        // copy constraints: define which columns have copy constraints, synthesize will specify which cells
        meta.enable_equality(a);
        meta.enable_equality(b);

        meta.create_gate("add gate", |meta| {
            let a = meta.query_advice(a, Rotation(0)); // 0 means same row, -1 means one row above, 1 means one row below
            let b = meta.query_advice(b, Rotation::cur()); // 0 means same row, -1 means one row above, 1 means one row below
            let q = meta.query_selector(q);

            vec![q * (a + b - Expression::Constant(F::from(10)))]
        });

        AddConfig { a, b, q }
    }

    // we need to fill in the values here (except instance values)
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        // fill in the selector here
        layouter.assign_region(
            || "add region",
            |mut region| {
                config.q.enable(&mut region, 0)?; // turning on the selector for the first row
                // or could also use: config.q.enable(&mut region, 0);

                // fill in the advice columns/witness values here
                region.assign_advice(|| "assign a", config.a, 0, || self.x)?;
                region.assign_advice(|| "assign b", config.b, 0, || self.y)?;

                Ok(())
            },
        )?;

        Ok(())
    }
}

// regions: a way that halo2 handles the rows of the table (not the same as chips)
//

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::bn256::Fr};

    #[test]
    fn accepts_values_that_sum_to_ten() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(4)),
            y: Value::known(Fr::from(6)),
        };

        let prover = MockProver::run(4, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn rejects_values_that_do_not_sum_to_ten() {
        let circuit = MyCircuit {
            x: Value::known(Fr::from(4)),
            y: Value::known(Fr::from(5)),
        };

        let prover = MockProver::run(4, &circuit, vec![]).unwrap();
        assert!(prover.verify().is_err());
    }
}
