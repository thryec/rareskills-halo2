## Problem 2 (challenge)

We want to verify that a specific Fibonacci number was computed correctly—for example, the sixth Fibonacci number, assuming the sequence starts with 1 and 1.

The index of the Fibonacci number that the verifier wants to check is provided as part of the instance. This complicates matters because the index is not known at compile time.

One possible solution is the following. Let us define a maximum number of rows, say `max = 16`. The prover computes Fibonacci numbers up to the 16th row, but only performs the actual Fibonacci recurrence while a flag remains enabled. Once the target index is reached, the prover stops computing new values and simply copies the last computed Fibonacci number into all remaining rows.

This requires at least two columns: one for the Fibonacci values and another for a flag indicating whether the prover should continue the computation.

For example, if `max = 8` and the verifier requests the fifth Fibonacci number, the prover fills the columns as follows:

```
Fib  Flag
1    1
1    1
2    1
3    1
5    1
5    0
5    0
5    0
```

Notice that once the flag is disabled, the prover simply pads the remaining rows with the last computed value.

At first glance, this seems sufficient. The verifier can provide both the target index and the expected Fibonacci number as public inputs. We then constrain the final row (given by `max`) of the Fibonacci column to equal the expected value. Since `max` is known at compile time, creating a copy constraint between the `max` row and an instance cell is straightforward.

The original difficulty is that we cannot create a copy constraint whose row index is itself provided as an instance value. Halo2 circuits are fixed at compile time, so constraints cannot depend on a row number supplied by the verifier.

This pattern—computing up to a fixed maximum and padding the remainder—is very common in zero-knowledge circuits. However, this construction it is still not enough to correctly verify Fibonacci numbers.

Can you see what is missing?