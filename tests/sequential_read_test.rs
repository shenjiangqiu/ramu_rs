use std::{fs::File, io::BufRead};

use ramu_rs::{
    config::Config,
    ddr4::DDR4,
    memory::{MemoryTrait, SimpleMemory},
    request::{ReqType, Request},
};

#[test]
fn sequence_read_test() {
    let config = Config::default();
    let ddr4 = DDR4::new(&config);
    let mut memory = SimpleMemory::new(config, ddr4);
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
    let mut memory = SimpleMemory::new(config, ddr4);
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
fn test_real_trace() {
    let file_name = "trace.bin";
    let file = File::open(file_name).unwrap();
    let reader = std::io::BufReader::new(file);
    let num_lines = reader.lines().into_iter().map(|line| line.unwrap());
    let config = Config::default();
    let ddr4 = DDR4::new(&config);
    let mut memory = SimpleMemory::new(config, ddr4);
    let mut on_going = 0;
    let now = std::time::Instant::now();
    for line in num_lines {
        let line = line.split_whitespace().collect::<Vec<_>>();
        let addr = line[1].parse::<u64>().unwrap();
        while let Err(_) = memory.try_send(Request::new(addr, ReqType::Read)) {
            memory.tick();
            if let Some(_) = memory.try_recv() {
                on_going -= 1;
            }
        }
        on_going += 1;
    }
    while on_going != 0 {
        memory.tick();
        if let Some(_) = memory.try_recv() {
            on_going -= 1;
        }
    }
    println!("finish");
    let ramu_cycle = 63589;
    println!("cycle: {}", memory.get_cycle());
    let elapsed = now.elapsed();
    println!(
        "the time is {}",
        elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
    );
    println!(
        "time per 10000 cycle: {}",
        elapsed.as_secs() as f64 / memory.get_cycle() as f64 * 10000.0
    );
    println!("the ramu cycle is {}", ramu_cycle);
}
