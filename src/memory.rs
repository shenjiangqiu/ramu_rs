use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::dram::LevelTrait;
use crate::{
    config::Config,
    controller::Controller,
    dram::{Dram, DramSpec},
    request::Request,
};
pub trait MemoryTrait {
    type T: DramSpec;
    fn clk_ns(&self) -> f64;
    fn tick(&mut self);
    fn try_send(&mut self, req: Request) -> Result<(), Request>;
    fn try_recv(&mut self) -> Option<Request>;
    fn pending_requests(&self) -> usize;
    fn finish(&mut self);
    fn get_spec(&self) -> &Self::T;
    fn decode_addr(&self, addr: u64) -> Vec<u64>;
    /// return the addr represented by the addr_vec, note: the lower 6 bits are ignored
    fn encode_addr(&self, addr: &[u64]) -> u64;
}

pub struct SimpleMemory<T: DramSpec> {
    config: Config,
    spec: T,
    clk: u64,
    controllers: Vec<Controller<T>>,
    ret_queue: VecDeque<Request>,
}
impl<T> SimpleMemory<T>
where
    T: DramSpec,
{
    pub fn new(config: Config, spec: T) -> Self {
        let mut controllers = Vec::new();
        for i in 0..config.channels {
            let dram = Dram::new(&spec, T::Level::channel(), i);
            let controller = Controller::new(&config, dram);
            controllers.push(controller);
        }

        SimpleMemory {
            config,
            spec,
            clk: 0,
            controllers,
            ret_queue: Default::default(),
        }
    }
    pub fn get_cycle(&self) -> u64 {
        self.clk
    }
}
#[derive(Debug, Serialize, Deserialize)]

pub enum MappingType {
    ChRaBaRoCo,
    RoBaRaCoCh,
    CoRoBaRaCh,
    RoCoBaRaCh,
}
impl MappingType {
    /// get the slicing sequence of the address,
    /// - `0` represent the channel, the `last` represent the column,
    /// - the `frist entry` represent the ***least*** significant bit in addr
    /// - the `last entry` represent the ***most significant*** bit in addr
    /// ## Example
    /// `[0,1,2,3,4]` means `__co__ro__ba__ra__ch__`, which is, first slice `ch`, then `ra`, `ba`, `ro`, `co`
    ///
    ///## Arguments
    /// - bg: contains bank group in level
    pub fn get_slice_sequence(&self, bg: bool) -> &'static [usize] {
        if !bg {
            match self {
                MappingType::ChRaBaRoCo => &[4, 3, 2, 1, 0],
                MappingType::RoBaRaCoCh => &[0, 4, 1, 2, 3],
                MappingType::CoRoBaRaCh => &[0, 1, 2, 3, 4],
                MappingType::RoCoBaRaCh => &[0, 1, 2, 4, 3],
            }
        } else {
            match self {
                MappingType::ChRaBaRoCo => &[5, 4, 3, 2, 1, 0],
                MappingType::RoBaRaCoCh => &[0, 5, 1, 3, 2, 4],
                MappingType::CoRoBaRaCh => &[0, 1, 3, 2, 4, 5],
                MappingType::RoCoBaRaCh => &[0, 1, 3, 2, 5, 4],
            }
        }
    }
}

impl<T> MemoryTrait for SimpleMemory<T>
where
    T: DramSpec,
{
    type T = T;
    fn clk_ns(&self) -> f64 {
        todo!()
    }

    fn tick(&mut self) {
        self.clk += 1;
        for controller in self.controllers.iter_mut() {
            controller.tick(&self.spec, self.clk);
            if let Some(req) = controller.finished_queue.pop_front() {
                self.ret_queue.push_back(req);
            }
        }
    }

    fn try_send(&mut self, mut req: Request) -> Result<(), Request> {
        // init the addr vec!
        if !req.done_setup {
            let decoded_addr = self.spec.decode_addr(req.addr, &self.config.mapping_type);
            req.addr_vec = decoded_addr;
            req.done_setup = true;
        }
        self.controllers[req.addr_vec[0] as usize].try_enqueue(req)
    }

    fn pending_requests(&self) -> usize {
        self.controllers
            .iter()
            .map(|c| {
                c.read_queue.size()
                    + c.write_queue.size()
                    + c.act_queue.size()
                    + c.other_queue.size()
            })
            .sum()
    }

    fn finish(&mut self) {
        todo!()
    }

    fn try_recv(&mut self) -> Option<Request> {
        self.ret_queue.pop_front()
    }

    fn get_spec(&self) -> &Self::T {
        &self.spec
    }

    fn decode_addr(&self, addr: u64) -> Vec<u64> {
        self.spec.decode_addr(addr, &self.config.mapping_type)
    }

    fn encode_addr(&self, addr: &[u64]) -> u64 {
        self.spec.encode_addr(addr, &self.config.mapping_type)
    }
}
