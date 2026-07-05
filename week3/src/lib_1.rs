use ff::PrimeField;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, ErrorFront, Fixed, Selector},
    poly::Rotation,
};

// implement fibonnaci circuit with one advice column
// this struct typically stores private witnesses, i.e. values needed by synthesize function
pub struct MyCircuit<F: PrimeField> {
    pub a: [Value<F>; 5], // 5 rows of fibonnaci sequence
}

#[derive(Clone)]
pub struct CircuitConfig {
    pub a: Column<Advice>,
    pub b: Column<Fixed>,
    pub q: Selector,
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
        let b = meta.fixed_column();
        let q = meta.selector();

        meta.enable_equality(a);
        meta.enable_constant(b);

        meta.create_gate("add gate", |meta| {
            let a_curr = meta.query_advice(a, Rotation::cur());
            let a_next = meta.query_advice(a, Rotation::next());
            let a_next_next = meta.query_advice(a, Rotation(2));
            let q = meta.query_selector(q);

            vec![q * (a_curr + a_next - a_next_next)]
        });

        CircuitConfig { a, b, q }
    }

    fn synthesize(
        &self, // the circuit object itself, actual values passed in via test file
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        layouter.assign_region(
            || "add region",
            |mut region| {
                for i in 0..3 {
                    config.q.enable(&mut region, i)?;
                }

                let a0 = region.assign_advice(|| "assign a[0]", config.a, 0, || self.a[0])?;
                let a1 = region.assign_advice(|| "assign a[1]", config.a, 1, || self.a[1])?;

                for i in 0..5 {
                    region.assign_advice(|| "assign a", config.a, i, || self.a[i])?;
                }

                region.constrain_constant(a0.cell(), F::ONE)?;
                region.constrain_constant(a1.cell(), F::ONE)?;

                Ok(())
            },
        )?;
        Ok(())
    }
}
