use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::bn256::Fr};
use week3::lib_1::MyCircuit;

fn circuit(values: [u64; 5]) -> MyCircuit<Fr> {
    MyCircuit {
        a: values.map(|value| Value::known(Fr::from(value))),
    }
}

#[test]
fn accepts_fibonacci_sequence() {
    let circuit = circuit([1, 1, 2, 3, 5]);

    let prover = MockProver::run(4, &circuit, vec![]).unwrap();

    prover.assert_satisfied();
}

#[test]
fn rejects_non_fibonacci_sequence() {
    let circuit = circuit([1, 1, 2, 3, 6]);

    let prover = MockProver::run(4, &circuit, vec![]).unwrap();

    assert!(prover.verify().is_err());
}
