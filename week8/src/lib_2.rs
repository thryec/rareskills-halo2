//! Problem 2 (challenge)
//!
//! Design a circuit for a generic automaton with variable states, symbols, transitions, and
//! accepting states.
//!
//! Provide the transition function through instance columns so it stays public. Copy transition
//! tuples into three advice columns, then check the private trace against them with `lookup_any`.
//! Use flag columns to mark active lookup rows.
//!
//! Provide accepting states through the instance as well. Copy them into advice before checking
//! the final trace state.
//!
//! Test with this binary even/odd automaton:
//!
//! - `(even, 0, even)`
//! - `(even, 1, odd)`
//! - `(odd, 0, even)`
//! - `(odd, 1, odd)`
//!
//! The initial and accepting state is `even`.
