use std::collections::VecDeque;

use crate::{
    config::Config,
    dram::{CommandTrait, Dram, DramSpec},
    refresh::Refresh,
    request::{ReqType, Request},
    rowpolicy::RowPolicy,
    rowtable::RowTable,
    scheduler::Scheduler,
};
pub struct Queue {
    pub queue: VecDeque<Request>,
    max: usize,
}
impl Default for Queue {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
            max: 512,
        }
    }
}
enum QueueType {
    Read,
    Write,
    Act,
    Other,
}
impl Queue {
    pub fn new(max: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max,
        }
    }
    pub fn size(&self) -> usize {
        self.queue.len()
    }
    pub fn full(&self) -> bool {
        self.queue.len() >= self.max
    }
}
pub enum RunningMode {
    Reading,
    Writing,
}
pub struct Controller<'a, T: DramSpec + ?Sized> {
    pub channel: Dram<'a, T>,
    pub scheduler: Scheduler,
    pub row_policy: RowPolicy,
    pub row_table: RowTable,
    pub refresh: Refresh,
    pub read_queue: Queue,
    pub write_queue: Queue,
    pub act_queue: Queue,
    pub other_queue: Queue,
    pub pending_queue: VecDeque<Request>,
    pub finished_queue: VecDeque<Request>,
    pub running_mode: RunningMode,
    pub wr_hight_watermark: f32,
    pub wr_low_watermark: f32,
}

impl<'a, T> Controller<'a, T>
where
    T: DramSpec,
{
    pub fn new(_config: &Config, dram: Dram<'a, T>) -> Self {
        Self {
            channel: dram,
            scheduler: Default::default(),
            row_policy: Default::default(),
            row_table: Default::default(),
            refresh: Default::default(),
            read_queue: Default::default(),
            write_queue: Default::default(),
            act_queue: Default::default(),
            other_queue: Default::default(),
            pending_queue: Default::default(),
            finished_queue: Default::default(),
            running_mode: RunningMode::Reading,
            wr_hight_watermark: 0.8,
            wr_low_watermark: 0.2,
        }
    }
    pub fn finish(_read_req: u64, _dram_cycles: u64) {
        todo!("implement me")
    }
    pub fn try_enqueue(&mut self, req: Request) -> Result<(), Request> {
        assert!(req.done_setup);
        let queue = match req.req_type {
            ReqType::Read => &mut self.read_queue,
            ReqType::Write => &mut self.write_queue,
        };
        if queue.full() {
            return Err(req);
        } else {
            queue.queue.push_back(req);
        }

        Ok(())
    }
    pub fn tick(&mut self, clk: u64) {
        // serve pending requests
        if let Some(req) = self.pending_queue.pop_front() {
            if req.finish_time <= clk {
                self.finished_queue.push_back(req);
            } else {
                self.pending_queue.push_front(req);
            }
        }
        // serve refresh
        self.refresh.tick(clk);

        // serve read/write queue
        match self.running_mode {
            RunningMode::Reading => {
                if self.read_queue.queue.is_empty()
                    || self.write_queue.size()
                        > (self.wr_hight_watermark * self.write_queue.max as f32) as usize
                {
                    self.running_mode = RunningMode::Writing;
                }
            }
            RunningMode::Writing => {
                if self.read_queue.size() > 0
                    && self.write_queue.size()
                        < (self.wr_low_watermark * self.write_queue.max as f32) as usize
                {
                    self.running_mode = RunningMode::Reading;
                }
            }
        }
        // find the best command to schedule
        if let Some((index, req)) = self.scheduler.get_best_req(&self.act_queue, &self.channel) {
            let cmd = self.get_first_cmd(req);
            let is_last = cmd == T::get_first_cmd(&req.req_type);
            if self.is_ready_cmd(&cmd, &req.addr_vec, clk) {
                self.issue_cmd(cmd, &req.addr_vec.clone(), clk);
                self.handle_after_issue(index, &cmd, is_last, QueueType::Act, clk);
                return;
            } else {
                // not find any valid command in act queue
                //tracing::debug!(?req, "not ready in act queue")
            }
        }
        // not find the act queue req
        let (queue, queue_type) = self.get_best_queue();
        if let Some((index, req)) = self.scheduler.get_best_req(queue, &self.channel) {
            let cmd = self.get_first_cmd(req);
            let is_last = cmd == T::get_first_cmd(&req.req_type);
            if self.is_ready_cmd(&cmd, &req.addr_vec, clk) {
                // pop the request from the queue
                self.issue_cmd(cmd, &req.addr_vec.clone(), clk);
                self.handle_after_issue(index, &cmd, is_last, queue_type, clk);
                return;
            } else {
                // not ready
                //tracing::debug!(?req, "not ready in rd/wr queue");
                return;
            }
        }

        // find read and write queue
    }
    pub fn is_ready_req(&self, _cmd: &Request) -> bool {
        todo!("implement me")
    }
    pub fn is_ready_cmd(&self, cmd: &T::Command, addr_vec: &[u64], clk: u64) -> bool {
        self.channel.check(cmd, addr_vec, clk)
    }
    pub fn is_row_hit_req(&self, _req: &Request) -> bool {
        todo!("implement me")
    }
    pub fn is_row_hit_cmd(&self, _cmd: &T::Command, _addr_vec: &[u64]) -> bool {
        todo!("implement me")
    }
    pub fn is_row_open_req(&self, _req: &Request) -> bool {
        todo!("implement me")
    }
    pub fn is_row_open_cmd(&self, _cmd: &T::Command, _addr_vec: &[u64]) -> bool {
        todo!("implement me")
    }
    pub fn is_active(&self) -> bool {
        todo!("implement me")
    }
    pub fn is_refreshing(&self) -> bool {
        todo!("implement me")
    }

    fn get_first_cmd(&self, req: &Request) -> T::Command {
        let frist_cmd = self.channel.get_first_cmd(&req.req_type);
        tracing::debug!(?frist_cmd, "the init cmd");
        return self.channel.decode(&frist_cmd, &req.addr_vec);
    }
    fn issue_cmd(&mut self, cmd: T::Command, addr_vec: &[u64], clk: u64) {
        tracing::debug!(?cmd, clk, "issue cmd");
        self.channel.update(&cmd, addr_vec, clk);
    }
    fn handle_after_issue(
        &mut self,
        cmd_index: usize,
        cmd: &T::Command,
        is_last: bool,
        queue_type: QueueType,
        clk: u64,
    ) {
        // check if the request is finished
        if is_last {
            let queue = match queue_type {
                QueueType::Read => &mut self.read_queue,
                QueueType::Write => &mut self.write_queue,
                QueueType::Act => &mut self.act_queue,
                QueueType::Other => &mut self.other_queue,
            };
            let mut req = queue.queue.remove(cmd_index).unwrap();
            match req.req_type {
                ReqType::Read => {
                    req.finish_time = clk + self.channel.spec.get_read_latency();
                    tracing::debug!(?req, clk, req.finish_time, "read request finished");

                    self.pending_queue.push_back(req);
                }
                ReqType::Write => {
                    req.finish_time = clk;
                    tracing::debug!(?req, "write request finished");

                    self.finished_queue.push_back(req);
                }
            }
        } else {
            if cmd.is_act() {
                let queue = match queue_type {
                    QueueType::Read => &mut self.read_queue,
                    QueueType::Write => &mut self.write_queue,
                    QueueType::Act => &mut self.act_queue,
                    QueueType::Other => &mut self.other_queue,
                };
                let req = queue.queue.remove(cmd_index).unwrap();
                self.act_queue.queue.push_back(req);
            }
        }
    }

    fn get_best_queue(&self) -> (&Queue, QueueType) {
        match self.other_queue.size() {
            0 => match self.running_mode {
                RunningMode::Reading => (&self.read_queue, QueueType::Read),
                RunningMode::Writing => (&self.write_queue, QueueType::Write),
            },
            _ => (&self.other_queue, QueueType::Other),
        }
    }
    #[allow(dead_code)]
    fn get_best_queue_mut(&mut self) -> &mut Queue {
        match self.other_queue.size() {
            0 => match self.running_mode {
                RunningMode::Reading => &mut self.read_queue,
                RunningMode::Writing => &mut self.write_queue,
            },
            _ => &mut self.other_queue,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ddr4::{Level, DDR4};
    use crate::dram::LevelTrait;
    use crate::test::init_logger;

    use super::*;

    #[test]
    fn test_controller_simple_read() {
        // act and rd
        init_logger();
        let config = Config::default();
        let ddr4 = DDR4::new(&config);
        let dram = Dram::new(&ddr4, Level::channel(), 0);
        let mut controller = Controller::new(&config, dram);
        let req = Request {
            req_type: ReqType::Read,
            addr_vec: vec![0x0, 0x0, 0x0, 0x0, 0, 0],
            finish_time: 0,
            addr: 0,
            done_setup: true,
            arrival_time: 0,
        };
        controller.try_enqueue(req).unwrap();
        // the first command should be act
        for i in 0..48 {
            // in cycle 22, it will be rd(act to rd)
            // in cycle
            controller.tick(i);
        }
        // in cycle 48, it will be finished(rd latency)
        assert!(controller.finished_queue.is_empty());
        controller.tick(48);
        assert_eq!(controller.finished_queue.len(), 1);
    }
}
