use crate::{
    controller::Queue,
    dram::{Dram, DramSpec},
    request::Request,
};

#[derive(Default)]
pub enum SchedulerType {
    #[default]
    FCFS,
    FRFCFS,
}
#[derive(Default)]
pub struct Scheduler {
    pub scheduler_type: SchedulerType,
}

impl Scheduler {
    pub fn get_best_req<'b, T: DramSpec + ?Sized>(
        &self,
        queue: &'b Queue,
        _dram: &Dram<T>,
    ) -> Option<(usize, &'b Request)> {
        if queue.size() == 0 {
            return None;
        } else {
            match self.scheduler_type {
                SchedulerType::FCFS => {
                    return Some((0, &queue.queue[0]));
                }
                SchedulerType::FRFCFS => None,
            }
        }
    }
}

#[cfg(test)]

mod tests {

    struct A {
        a: i32,
    }
    impl A {
        fn get_mut(&mut self) -> &mut i32 {
            &mut self.a
        }
    }
    #[test]
    fn test() {
        let mut a = A { a: 1 };
        let b = a.get_mut();
        *b = 2;
    }
}
