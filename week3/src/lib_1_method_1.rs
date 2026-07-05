use ff::PrimeField;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, ErrorFront, Expression, Selector},
    poly::Rotation,
};

// implement fibonnaci circuit with one advice column
pub struct MyCircuit<F: PrimeField> {
    pub a: [Value<F>; 5], // 5 rows of fibonnaci sequence
}

#[derive(Clone)]
pub struct CircuitConfig {
    pub a: Column<Advice>,
    pub q: Selector,
    pub q_first: Selector, // for a[0] = a[1] = 1
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            a: [Value::unknown(); 5],
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();
        let q = meta.selector();
        let q_first = meta.selector();

        meta.create_gate("add gate", |meta| {
            let a_curr = meta.query_advice(a, Rotation::cur());
            let a_next = meta.query_advice(a, Rotation::next());
            let a_next_next = meta.query_advice(a, Rotation(2));
            let q = meta.query_selector(q);

            vec![q * (a_curr + a_next - a_next_next)]
        });

        meta.create_gate("limit first two values to 1", |meta| {
            let a_curr = meta.query_advice(a, Rotation::cur());
            let q_first = meta.query_selector(q_first);

            vec![q_first * (a_curr - Expression::Constant(F::ONE))]
        });

        CircuitConfig { a, q, q_first }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        layouter.assign_region(
            || "add region",
            |mut region| {
                for i in 0..2 {
                    config.q_first.enable(&mut region, i)?;
                }
                for i in 0..3 {
                    config.q.enable(&mut region, i)?;
                }

                for i in 0..5 {
                    region.assign_advice(|| "assign a", config.a, i, || self.a[i])?;
                }

                Ok(())
            },
        )?;
        Ok(())
    }
}
