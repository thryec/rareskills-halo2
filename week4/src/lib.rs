use ff::PrimeField;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{
        Advice, Circuit, Column, ConstraintSystem, ErrorFront, Expression, Instance, Selector,
    },
    poly::Rotation,
};

pub const MAX_ROWS: usize = 16;

// defines the witness values for this circuit
pub struct MyCircuit<F: PrimeField> {
    pub fib_values: Vec<Value<F>>,
    pub flag_values: Vec<Value<F>>,
    pub counter_values: Vec<Value<F>>,
}

// defines the columns + gates for this circuit
#[derive(Clone)]
pub struct CircuitConfig {
    pub fib: Column<Advice>,
    pub flag: Column<Advice>,
    pub counter: Column<Advice>,
    pub instance: Column<Instance>,
    pub q_fib: Selector,
    pub q_flag: Selector,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            fib_values: vec![Value::unknown(); MAX_ROWS],
            flag_values: vec![Value::unknown(); MAX_ROWS],
            counter_values: vec![Value::unknown(); MAX_ROWS],
        }
    }

    // define the circuit shape: columns, selectors, equality, and gates
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let fib = meta.advice_column();
        let flag = meta.advice_column();
        let counter = meta.advice_column();
        let instance = meta.instance_column();
        let q_fib = meta.selector();
        let q_flag = meta.selector();
        let fixed = meta.fixed_column();

        meta.enable_equality(fib);
        meta.enable_equality(flag);
        meta.enable_equality(counter);
        meta.enable_equality(instance);
        meta.enable_constant(fixed);

        // gate: check fib sequence, controlled by flag
        meta.create_gate("fib gate", |meta| {
            let fib_cur = meta.query_advice(fib, Rotation::cur());
            let fib_next = meta.query_advice(fib, Rotation::next());
            let fib_next_next = meta.query_advice(fib, Rotation(2));
            let flag_next_next = meta.query_advice(flag, Rotation(2));
            let q = meta.query_selector(q_fib);

            // if the flag 2 rows down is 1, then fib sequence has to be enforced
            // if the flag 2 rows down is 0, then fib sequence will stay at the last fib value
            vec![
                q.clone()
                    * flag_next_next.clone()
                    * (fib_cur + fib_next.clone() - fib_next_next.clone()),
                q * (Expression::Constant(F::ONE) - flag_next_next) * (fib_next_next - fib_next),
            ]
        });

        // gate: check flag is either 0 or 1, counter should count number of flags
        // once flag is 0, it stays off at 0
        meta.create_gate("flag gate", |meta| {
            let flag_cur = meta.query_advice(flag, Rotation::cur());
            let flag_next = meta.query_advice(flag, Rotation::next());
            let counter_cur = meta.query_advice(counter, Rotation::cur());
            let counter_next = meta.query_advice(counter, Rotation::next());
            let q = meta.query_selector(q_flag);

            let one = Expression::Constant(F::ONE);

            // counter tracks the running sum of flags:
            // counter_next = counter_cur + flag_next

            // if flag_next = 1, counter increments by 1
            // if flag_next = 0, counter stays the same
            vec![
                // flag is either 0 or 1
                q.clone() * flag_cur.clone() * (flag_cur.clone() - one.clone()),
                q.clone() * flag_next.clone() * (flag_next.clone() - one.clone()),
                // counter tracks running sum of flags
                q.clone() * (counter_next - counter_cur - flag_next.clone()),
                // once flag is 0, next flag must stay 0
                q * (one - flag_cur) * flag_next,
            ]
        });

        CircuitConfig {
            fib,
            flag,
            counter,
            instance,
            q_fib,
            q_flag,
        }
    }

    // fill witness values into table columns and enable selectors for this proof
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        let (final_fib, final_counter) = layouter.assign_region(
            || "fib table",
            |mut region| {
                // fib selector, needs 2 rows after current row due to fib_next_next
                for offset in 0..MAX_ROWS - 2 {
                    config.q_fib.enable(&mut region, offset)?;
                }

                // flag selector, needs 1 row after current row due to flag_next
                for offset in 0..MAX_ROWS - 1 {
                    config.q_flag.enable(&mut region, offset)?;
                }

                let mut final_fib = None;
                let mut final_counter = None;

                for offset in 0..MAX_ROWS {
                    // retrieve individual values from witness
                    let fib_value = self
                        .fib_values
                        .get(offset)
                        .copied()
                        .unwrap_or_else(Value::unknown);
                    let flag_value = self
                        .flag_values
                        .get(offset)
                        .copied()
                        .unwrap_or_else(Value::unknown);
                    let counter_value = self
                        .counter_values
                        .get(offset)
                        .copied()
                        .unwrap_or_else(Value::unknown);

                    // assign values to actual circuit columns
                    let fib_cell =
                        region.assign_advice(|| "fib", config.fib, offset, || fib_value)?;
                    let flag_cell =
                        region.assign_advice(|| "flag", config.flag, offset, || flag_value)?;
                    let counter_cell = region.assign_advice(
                        || "counter",
                        config.counter,
                        offset,
                        || counter_value,
                    )?;

                    if offset == 0 {
                        region.constrain_constant(fib_cell.cell(), F::ONE)?;
                        region.constrain_constant(flag_cell.cell(), F::ONE)?;
                        region.constrain_constant(counter_cell.cell(), F::ONE)?;
                    }

                    if offset == 1 {
                        region.constrain_constant(fib_cell.cell(), F::ONE)?;
                    }

                    // save final row cells; padding should make final fib equal the requested value
                    if offset == MAX_ROWS - 1 {
                        final_fib = Some(fib_cell);
                        final_counter = Some(counter_cell);
                    }
                }

                Ok((final_fib.unwrap(), final_counter.unwrap()))
            },
        )?;

        // final counter equals public target index
        layouter.constrain_instance(final_counter.cell(), config.instance, 0)?;

        // final fib equals public expected Fibonacci value
        layouter.constrain_instance(final_fib.cell(), config.instance, 1)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

    fn known_values(values: &[u64]) -> Vec<Value<Fr>> {
        values
            .iter()
            .map(|value| Value::known(Fr::from(*value)))
            .collect()
    }

    #[test]
    fn accepts_fifth_fibonacci_value() {
        let mut fib_values = vec![1, 1, 2, 3, 5];
        fib_values.resize(MAX_ROWS, 5);

        let mut flag_values = vec![1, 1, 1, 1, 1];
        flag_values.resize(MAX_ROWS, 0);

        let mut counter_values = vec![1, 2, 3, 4, 5];
        counter_values.resize(MAX_ROWS, 5);

        let circuit = MyCircuit::<Fr> {
            fib_values: known_values(&fib_values),
            flag_values: known_values(&flag_values),
            counter_values: known_values(&counter_values),
        };

        let public_inputs = vec![vec![Fr::from(5), Fr::from(5)]];
        let prover = MockProver::run(5, &circuit, public_inputs).unwrap();

        prover.assert_satisfied();
    }
}
