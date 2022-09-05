# ramu_rs
a re-implementation of [Ramulator](https://github.com/CMU-SAFARI/ramulator) in pure rust
- same accuracy as [Ramulator](https://github.com/CMU-SAFARI/ramulator)(the same result compare to Ramulator, not even one cycle error - tested by 4000k real application trace)
- a little faster: 12x faster than Ramulator,running a 4000k real-application read instructions, this crate costs 13s, and the Ramulator costs 156s (same config:DDR4, FCFS scheduler, No refreshing)
- trait system for implementing different dram types, it makes it easy to add more dram specifications.
- it's still in a very early stage, but ready to be used in some simple simulations. currently it only support DDR4, FCFS scheduler,( no refreshing, no FRFCFS, no HBM...)
