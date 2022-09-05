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
#[cfg(test)]
mod test {
    use crate::{
        config::Config,
        ddr4::DDR4,
        dram::DramSpec,
        memory::{MemoryTrait, SimpleMemory},
        request::{ReqType, Request},
    };
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
    #[test]
    fn test_memory_diffrent_row() {
        init_logger();
        let config = Config::default();
        let ddr4 = DDR4::new(&config);

        let mut mem = SimpleMemory::new(&config, &ddr4);
        let addr_vec_1 = [0, 0, 0, 0, 0, 0];
        let addr_vec_2 = [0, 0, 0, 0, 1, 0];
        let addr_1 = ddr4.encode_addr(&addr_vec_1, &config.mapping_type);
        let addr_2 = ddr4.encode_addr(&addr_vec_2, &config.mapping_type);
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

        let mut mem = SimpleMemory::new(&config, &ddr4);
        let addr_vec_1 = [0, 0, 0, 0, 0, 0];
        let addr_vec_2 = [0, 0, 0, 0, 1, 0];
        let addr_1 = ddr4.encode_addr(&addr_vec_1, &config.mapping_type);
        let addr_2 = ddr4.encode_addr(&addr_vec_2, &config.mapping_type);
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

        let mut mem = SimpleMemory::new(&config, &ddr4);
        let addr_vec_1 = [0, 0, 0, 0, 0, 0];
        let addr_vec_2 = [0, 0, 0, 0, 1, 0];
        let addr_1 = ddr4.encode_addr(&addr_vec_1, &config.mapping_type);
        let addr_2 = ddr4.encode_addr(&addr_vec_2, &config.mapping_type);
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
