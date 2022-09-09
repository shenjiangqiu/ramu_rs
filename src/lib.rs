//! the pure rust implementation of the ramulator.
//!
//!
//!

pub mod config;
pub mod controller;
pub mod ddr4;
pub mod dram;
pub mod memory;
pub(crate) mod refresh;
pub mod request;
pub(crate) mod rowpolicy;
pub(crate) mod rowtable;
pub(crate) mod scheduler;
pub(crate) mod utils;

use config::Config;
use ddr4::DDR4;
use memory::{
    MemoryTrait,
    SimpleMemory::{self},
};
use request::{ReqType, Request};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub(crate) fn init_logger() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .try_init()
        .unwrap_or_else(|e| {
            eprintln!("Failed to init tracing: {}", e);
        });
}

type SimpleDDR4 = SimpleMemory<DDR4>;

#[cxx::bridge]
mod ffi {

    extern "Rust" {
        type SimpleDDR4;
        fn init_logger();
        fn new_ddr4(config: &str) -> Box<SimpleDDR4>;
        fn tick_ddr4(&mut self);
        fn try_send_addr(&mut self, addr: u64, is_write: bool) -> bool;
        fn try_recv_addr(&mut self, addr: &mut u64, is_write: &mut bool) -> bool;
        fn get_cycle(&self) -> u64;
    }
}

pub fn new_ddr4<'a>(config: &str) -> Box<SimpleDDR4> {
    let config = Config::from_toml_path(config);
    let ddr4 = DDR4::new(&config);
    Box::new(SimpleMemory::new(config, ddr4))
}
impl SimpleDDR4 {
    fn tick_ddr4(&mut self) {
        self.tick();
    }
    fn try_send_addr(&mut self, addr: u64, is_write: bool) -> bool {
        self.try_send(Request::new(
            addr,
            if is_write {
                ReqType::Write
            } else {
                ReqType::Read
            },
        ))
        .is_ok()
    }
    fn try_recv_addr(&mut self, addr: &mut u64, is_write: &mut bool) -> bool {
        if let Some(req) = self.try_recv() {
            *addr = req.addr;
            *is_write = req.req_type.is_write();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::Config,
        ddr4::DDR4,
        memory::{MemoryTrait, SimpleMemory},
        request::{ReqType, Request},
    };

    #[test]
    fn test_memory_diffrent_row() {
        init_logger();
        let config = Config::default();
        let ddr4 = DDR4::new(&config);

        let mut mem = SimpleMemory::new(config, ddr4);
        let addr_vec_1 = [0, 0, 0, 0, 0, 0];
        let addr_vec_2 = [0, 0, 0, 0, 1, 0];
        let addr_1 = mem.encode_addr(&addr_vec_1);
        let addr_2 = mem.encode_addr(&addr_vec_2);
        tracing::debug!(addr_1, addr_2);
        mem.try_send(Request::new(addr_1, ReqType::Read)).unwrap();
        mem.try_send(Request::new(addr_2, ReqType::Read)).unwrap();
        for _i in 0..1000 {
            mem.tick();
            while let Some(req) = mem.try_recv() {
                tracing::debug!(?req, "recv");
            }
        }
    }

    #[test]
    fn test_memory_write() {
        init_logger();
        let config = Config::default();
        let ddr4 = DDR4::new(&config);

        let mut mem = SimpleMemory::new(config, ddr4);
        let addr_vec_1 = [0, 0, 0, 0, 0, 0];
        let addr_vec_2 = [0, 0, 0, 0, 1, 0];
        let addr_1 = mem.encode_addr(&addr_vec_1);
        let addr_2 = mem.encode_addr(&addr_vec_2);
        tracing::debug!(addr_1, addr_2);
        mem.try_send(Request::new(addr_1, ReqType::Write)).unwrap();
        mem.try_send(Request::new(addr_2, ReqType::Write)).unwrap();
        for _i in 0..1000 {
            mem.tick();
            while let Some(req) = mem.try_recv() {
                tracing::debug!(?req, "recv");
            }
        }
    }

    #[test]
    fn test_memory_read_and_write() {
        init_logger();
        let config = Config::default();
        let ddr4 = DDR4::new(&config);

        let mut mem = SimpleMemory::new(config, ddr4);
        let addr_vec_1 = [0, 0, 0, 0, 0, 0];
        let addr_vec_2 = [0, 0, 0, 0, 1, 0];
        let addr_1 = mem.encode_addr(&addr_vec_1);
        let addr_2 = mem.encode_addr(&addr_vec_2);
        tracing::debug!(addr_1, addr_2);
        mem.try_send(Request::new(addr_1, ReqType::Read)).unwrap();
        mem.try_send(Request::new(addr_2, ReqType::Write)).unwrap();
        for _i in 0..1000 {
            mem.tick();
            while let Some(req) = mem.try_recv() {
                tracing::debug!(?req, "recv");
            }
        }
    }
}
