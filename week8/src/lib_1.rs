//! Problem 1
//!
//! It has three states $(r_0, r_1, r_2)$ and a two-letter alphabet
//! $\{0,1\}$. The accepting state is $r_0$, which is also the
//! initial state.
//!
//! The goal of this problem is to implement this finite automaton in
//! Halo2. Represent the transition function as a set of tuples of the
//! form *(current state, letter, next state)*, and use a lookup argument
//! to verify that every transition in the trace appears in this table
//! of tuples.
//!
//! The prover must prove knowledge of a word (a sequence of letters)
//! that is accepted by this automaton.
//!
//! Since the length of the word is unknown, introduce a **no-op** symbol
//! and use it to pad the trace to the maximum allowed word length.
//!
//! *note: If you use the value **0** to represent state $r_0$, you will
//! run into problems. Can you figure out why?*
//!
//! The source diagram represents these transition tuples:
//!
//! - `(r0, 0, r0)`
//! - `(r0, 1, r1)`
//! - `(r1, 0, r2)`
//! - `(r1, 1, r0)`
//! - `(r2, 0, r1)`
//! - `(r2, 1, r2)`
//!
//! no-ops:
//! (r0, 2, r0)
//! (r1, 2, r1)
//! (r2, 2, r2)
//!
//! r0 = 1, r1 = 2, r2 = 3

use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, ErrorFront, Selector, TableColumn,
};
use halo2_proofs::poly::Rotation;

const NUM_STEPS: usize = 12;

pub struct AutomataCircuit<F: PrimeField> {
    pub trace: [(Value<F>, Value<F>); NUM_STEPS],
    pub initial_state: F,
    pub accepting_states: Vec<Value<F>>,
    pub transitions: Vec<(Value<F>, Value<F>, Value<F>)>,
}

#[derive(Clone)]
pub struct AutomataConfig {
    state: Column<Advice>,
    symbol: Column<Advice>,
    q_transition: Selector,
    q_accept: Selector,
    t_state: TableColumn,
    t_symbol: TableColumn,
    t_next: TableColumn,
    t_acc: TableColumn,
}

impl<F: PrimeField> Circuit<F> for AutomataCircuit<F> {
    type Config = AutomataConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            trace: [(Value::unknown(), Value::unknown()); NUM_STEPS],
            initial_state: F::ZERO,
            accepting_states: vec![],
            transitions: vec![],
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let state = meta.advice_column();
        let symbol = meta.advice_column();
        let constants = meta.fixed_column();

        let q_transition = meta.complex_selector();
        let q_accept = meta.complex_selector();

        let t_state = meta.lookup_table_column();
        let t_symbol = meta.lookup_table_column();
        let t_next = meta.lookup_table_column();
        let t_acc = meta.lookup_table_column();

        meta.enable_constant(constants);
        meta.enable_equality(state);

        // check if transition is valid
        meta.lookup("check state transition", |meta| {
            let state_cur = meta.query_advice(state, Rotation::cur());
            let state_next = meta.query_advice(state, Rotation::next());
            let symbol = meta.query_advice(symbol, Rotation::cur());

            let q = meta.query_selector(q_transition);

            vec![
                (q.clone() * state_cur, t_state),
                (q.clone() * symbol, t_symbol),
                (q * state_next, t_next),
            ]
        });

        // check that final state is an accepting state i.e. r0
        meta.lookup("check final state", |meta| {
            let state = meta.query_advice(state, Rotation::cur());
            let q = meta.query_selector(q_accept);

            vec![(q * state, t_acc)]
        });

        AutomataConfig {
            state,
            symbol,
            q_transition,
            q_accept,
            t_state,
            t_symbol,
            t_next,
            t_acc,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        // populate transition table
        layouter.assign_table(
            || "transition table",
            |mut table| {
                for i in 0..self.transitions.len() {
                    table.assign_cell(|| "state", config.t_state, i, || self.transitions[i].0)?;
                    table.assign_cell(|| "symbol", config.t_symbol, i, || self.transitions[i].1)?;
                    table.assign_cell(|| "next", config.t_next, i, || self.transitions[i].2)?;
                }

                Ok(())
            },
        )?;

        // populate accepting states
        layouter.assign_table(
            || "accepting states",
            |mut table| {
                for (i, state) in self.accepting_states.iter().enumerate() {
                    table.assign_cell(|| "accepting", config.t_acc, i, || *state)?;
                }

                Ok(())
            },
        )?;

        // populate trace region
        layouter.assign_region(
            || "trace",
            |mut region| {
                for i in 0..NUM_STEPS {
                    if i == 0 {
                        region.assign_advice_from_constant(
                            || "initial state",
                            config.state,
                            0,
                            self.initial_state,
                        )?;
                    } else {
                        region.assign_advice(|| "state", config.state, i, || self.trace[i].0)?;
                    }
                    region.assign_advice(|| "symbol", config.symbol, i, || self.trace[i].1)?;

                    if i != NUM_STEPS - 1 {
                        config.q_transition.enable(&mut region, i)?;
                    } else {
                        config.q_accept.enable(&mut region, i)?;
                    }
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff::Field;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr;

    const K: u32 = 5;

    // State encoding: r0 = 1, r1 = 2, r2 = 3.
    // Symbol 2 is the no-op symbol used for padding.
    fn transition_table() -> Vec<(Value<Fr>, Value<Fr>, Value<Fr>)> {
        [
            (0, 0, 0), // sentinel for disabled lookups
            (1, 0, 1),
            (1, 1, 2),
            (2, 0, 3),
            (2, 1, 1),
            (3, 0, 2),
            (3, 1, 3),
            (1, 2, 1), // no-op self-loops
            (2, 2, 2),
            (3, 2, 3),
        ]
        .into_iter()
        .map(|(state, symbol, next)| {
            (
                Value::known(Fr::from(state)),
                Value::known(Fr::from(symbol)),
                Value::known(Fr::from(next)),
            )
        })
        .collect()
    }

    fn accepting_states() -> Vec<Value<Fr>> {
        vec![
            Value::known(Fr::ZERO), // sentinel for disabled lookups
            Value::known(Fr::ONE),  // r0 is accepting
        ]
    }

    fn make_trace(rows: [(u64, u64); NUM_STEPS]) -> [(Value<Fr>, Value<Fr>); NUM_STEPS] {
        rows.map(|(state, symbol)| {
            (
                Value::known(Fr::from(state)),
                Value::known(Fr::from(symbol)),
            )
        })
    }

    fn circuit_for(trace: [(Value<Fr>, Value<Fr>); NUM_STEPS]) -> AutomataCircuit<Fr> {
        AutomataCircuit {
            trace,
            initial_state: Fr::ONE,
            accepting_states: accepting_states(),
            transitions: transition_table(),
        }
    }

    #[test]
    fn accepts_number_divisible_by_three() {
        // The word 110 is binary 6, which is divisible by 3.
        let trace = make_trace([
            (1, 1), // r0 --1--> r1
            (2, 1), // r1 --1--> r0
            (1, 0), // r0 --0--> r0
            (1, 2),
            (1, 2),
            (1, 2),
            (1, 2),
            (1, 2),
            (1, 2),
            (1, 2),
            (1, 2),
            (1, 2), // final state is r0
        ]);

        let prover = MockProver::run(K, &circuit_for(trace), vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn rejects_number_not_divisible_by_three() {
        // The word 10 is binary 2, so the final state is r2, not r0.
        let trace = make_trace([
            (1, 1), // r0 --1--> r1
            (2, 0), // r1 --0--> r2
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2), // final state is r2
        ]);

        let prover = MockProver::run(K, &circuit_for(trace), vec![]).unwrap();
        assert!(prover.verify().is_err());
    }

    #[test]
    fn rejects_forged_transition() {
        // The no-op transition from r2 must stay at r2, not move to r0.
        let trace = make_trace([
            (1, 1), // r0 --1--> r1
            (2, 0), // r1 --0--> r2
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2),
            (3, 2), // forged transition below claims r2 -> r0
            (1, 2),
        ]);

        let prover = MockProver::run(K, &circuit_for(trace), vec![]).unwrap();
        assert!(prover.verify().is_err());
    }
}
