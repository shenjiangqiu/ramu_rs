use ramu_pim_rust::{
    config::Config,
    ddr4::DDR4,
    memory::{MemoryTrait, SimpleMemory},
    request::{ReqType, Request},
};

#[test]
fn sequence_read_test() {
    let config = Config::default();
    let ddr4 = DDR4::new(&config);
    let mut memory = SimpleMemory::new(&config, &ddr4);
    let mut on_going = 0;
    let mut addr_start = 0;
    loop {
        if let Ok(_) = memory.try_send(Request::new(addr_start, ReqType::Read)) {
            addr_start += 1;
            on_going += 1;
        }
        if let Some(_) = memory.try_recv() {
            on_going -= 1;
        }
        memory.tick();
        if addr_start == 10240 {
            break;
        }
    }
    while on_going != 0 {
        if let Some(_) = memory.try_recv() {
            on_going -= 1;
        }
        memory.tick();
    }
    println!("finish");
    let ramu_cycle = 63589;
    println!("cycle: {}", memory.get_cycle());
    println!("the ramu cycle is {}", ramu_cycle);
    println!(
        "the ratio is cycle/remu_cycle {}",
        memory.get_cycle() as f64 / ramu_cycle as f64
    );
}

#[test]
fn test_read_and_write() {
    let config = Config::default();
    let ddr4 = DDR4::new(&config);
    let mut memory = SimpleMemory::new(&config, &ddr4);
    let mut on_going = 0;
    let mut addr_start = 0;
    loop {
        if let Ok(_) = memory.try_send(Request::new(
            addr_start,
            if addr_start % 2 == 0 {
                ReqType::Read
            } else {
                ReqType::Write
            },
        )) {
            addr_start += 1;
            on_going += 1;
        }
        if let Some(_) = memory.try_recv() {
            on_going -= 1;
        }
        memory.tick();
        if addr_start == 10240 {
            break;
        }
    }
    while on_going != 0 {
        if let Some(_) = memory.try_recv() {
            on_going -= 1;
        }
        memory.tick();
    }
    println!("finish");
    let ramu_cycle = 64100;
    println!("cycle: {}", memory.get_cycle());
    println!("the ramu cycle is {}", ramu_cycle);
    println!(
        "the ratio is cycle/remu_cycle {}",
        memory.get_cycle() as f64 / ramu_cycle as f64
    );
}
#[test]
fn test_real_trace() {}
