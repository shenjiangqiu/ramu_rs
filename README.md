# ramu_rs
a re-implementation of Ramulator in pure rust
- same accuracy as Ramulator(the same result compare to Ramulator, not even one cycle bias - tested by 4000k real application trace)
- blazing fast: 12x faster than Ramulator
  - running a 4000k real-application read instructions, this crate costs 13s, the Ramulator costs 156s
- trait system for implementing different dram types, it makes it easy to adding more dram specifications.
