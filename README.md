# ramu_rs
- same accuracy as ramulator(exactly the same result compare to ramulator, not even one cycle bias - tested by 4000k real application trace)
- an re-implementation of ramulator in pure rust
- blazing fast: 12x faster than ramulator
  - running a 4000k real-application read instructions, this crate costs 13s, the ramulator costs 156s
- trait system for implemnting different dram type, it makes it easy to scale.
