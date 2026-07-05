## Problem 1

Design a circuit using an arithmetic chip that performs both addition and multiplication. Combine both operations into a single chip. Define the columns `a`, `b`, and `c` at the circuit level so they can be shared with other chips.

## Problem 2

Create a circuit using a chip called `isZero`, which takes a field element as input and outputs `1` if the element is zero and `0` otherwise.

Note: An auxiliary input value is required for this circuit to operate.

## Problem 3

Use the circuit from Problem 1 to prove that the prover knows a value $a$ such that $a^5 + a = b$, where $b$ is a public input.

Although you could implement this using a custom gate, such a solution would not be generic. Instead, use the arithmetic chip to construct the circuit.

## Problem 4

Implement a chip that computes the bitwise XOR of two unsigned integers of up to 32 bits. Be sure to constrain both inputs to the 32-bit range.