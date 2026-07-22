//! Problem 1
//!
//! Use lookup tables to calculate the XOR of two 8-bit numbers. You may use chips, but it is
//! not mandatory.
//!
//! Can you scale this up to 16 or 32 bits? Try to determine at what point it becomes infeasible.

use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, ErrorFront, Selector, TableColumn,
};
use halo2_proofs::poly::Rotation;

pub struct XorCircuit<F: PrimeField> {
    x: Value<F>,
    y: Value<F>,
    out: Value<F>,
}

#[derive(Clone)]
pub struct XorConfig {
    x: Column<Advice>,
    y: Column<Advice>,
    out: Column<Advice>,
    x_table: TableColumn,
    y_table: TableColumn,
    out_table: TableColumn,
    q: Selector,
}

impl<F: PrimeField> Circuit<F> for XorCircuit<F> {
    type Config = XorConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        XorCircuit {
            x: Value::unknown(),
            y: Value::unknown(),
            out: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let out = meta.advice_column();

        let x_table = meta.lookup_table_column();
        let y_table = meta.lookup_table_column();
        let out_table = meta.lookup_table_column();

        let q = meta.complex_selector();

        meta.lookup("xor binary check", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let y = meta.query_advice(y, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());

            let q = meta.query_selector(q);

            vec![
                (q.clone() * x, x_table),
                (q.clone() * y, y_table),
                (q * out, out_table),
            ]
        });

        XorConfig {
            x,
            y,
            out,
            x_table,
            y_table,
            out_table,
            q,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        layouter.assign_table(
            || "table",
            |mut table| {
                let mut row = 0;
                for x in 0..256 {
                    for y in 0..256 {
                        let out = x ^ y;

                        table.assign_cell(
                            || "x bit",
                            config.x_table,
                            row,
                            || Value::known(F::from(x as u64)),
                        )?;

                        table.assign_cell(
                            || "y bit",
                            config.y_table,
                            row,
                            || Value::known(F::from(y as u64)),
                        )?;

                        table.assign_cell(
                            || "out bit",
                            config.out_table,
                            row,
                            || Value::known(F::from(out as u64)),
                        )?;
                        row += 1;
                    }
                }

                Ok(())
            },
        )?;

        layouter.assign_region(
            || "region",
            |mut region| {
                region.assign_advice(|| "x", config.x, 0, || self.x)?;
                region.assign_advice(|| "y", config.y, 0, || self.y)?;
                region.assign_advice(|| "out", config.out, 0, || self.out)?;

                config.q.enable(&mut region, 0)?;
                Ok(())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::XorCircuit;
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr;

    #[test]
    fn should_work() {
        let circuit = XorCircuit {
            x: Value::known(Fr::from(1)),
            y: Value::known(Fr::from(1)),
            out: Value::known(Fr::from(0)),
        };

        let prover = MockProver::run(18, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn should_fail() {
        let circuit = XorCircuit {
            x: Value::known(Fr::from(1)),
            y: Value::known(Fr::from(1)),
            out: Value::known(Fr::from(1)),
        };

        let prover = MockProver::run(18, &circuit, vec![]).unwrap();
        assert!(prover.verify().is_err());
    }
}
