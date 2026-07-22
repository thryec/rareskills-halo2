## Problem 1

Use lookup tables to calculate the XOR of two 8-bit numbers. You may use chips, but it is not mandatory.

Can you scale this up to 16 or 32 bits? Try to determine at what point it becomes infeasible.

## Problem 2

Create a 16-bit range table to perform the addition of two 16-bit numbers. However, the gate cannot simply be x + y - z, because there is a possibility of overflow that we need to handle.

One way to solve this is by using a carry bit, which should be part of the Advice. Try this approach.

**Challenge**: Another possibility is to decompose the number into low and high parts—for example, 8 bits each (or even more parts for numbers with more bits). We will cover this in a future lesson. Can you figure out how to perform addition this way? Try to design a circuit to add 32-bit numbers using smaller chunks—such as 16-bit or 8-bit chunks—while accounting for potential overflow.

*Note: Both problems require only static lookup tables.*
