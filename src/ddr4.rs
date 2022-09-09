use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    dram::{self, CommandTrait, Dram, DramSpec, LevelTrait, State, TimeEntry},
    memory::MappingType,
    request::ReqType,
    utils::{self, clear_lower_bits},
};
fn log2(mut x: usize) -> usize {
    let mut i = 0;
    while x > 1 {
        x = x >> 1;
        i += 1;
    }
    i
}
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Level {
    Channel = 0,
    Rank,
    BankGroup,
    Bank,
    Row,
    Column,
}

impl Level {
    pub fn next_level(&self) -> Option<Level> {
        match self {
            Level::Channel => Some(Level::Rank),
            Level::Rank => Some(Level::BankGroup),
            Level::BankGroup => Some(Level::Bank),
            Level::Bank => Some(Level::Row),
            Level::Row => Some(Level::Column),
            Level::Column => None,
        }
    }
}
impl LevelTrait for Level {
    const MAX_LEVEL: usize = 6;

    fn is_row(&self) -> bool {
        *self == Level::Row
    }

    fn is_bank(&self) -> bool {
        *self == Level::Bank
    }

    fn have_bank_group() -> bool {
        true
    }

    fn is_channel(&self) -> bool {
        *self == Level::Channel
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }

    fn next_level(&self) -> Option<Self> {
        self.next_level()
    }

    fn channel() -> Self {
        Level::Channel
    }

    fn need_init_dram(&self) -> bool {
        // all level below row do not need init, so we can just test if it is row, so lower level will be never triggered
        !self.is_row()
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    IntoPrimitive,
    TryFromPrimitive,
    enum_as_inner::EnumAsInner,
)]
#[repr(u8)]
pub enum Command {
    ACT = 0,
    PRE,
    PREA,
    RD,
    WR,
    RDA,
    WRA,
    REF,
    PDE,
    PDX,
    SRE,
    SRX,
}
impl CommandTrait for Command {
    const MAX: usize = 12;

    fn try_from_u8(val: u8) -> Result<Self, ()> {
        Self::try_from(val).map_err(|_| ())
    }

    fn to_u8(self) -> u8 {
        let val: u8 = self.into();
        val
    }

    fn to_usize(self) -> usize {
        let val: u8 = self.into();
        val as usize
    }

    fn try_from_usize(val: usize) -> Result<Self, ()> {
        Self::try_from(val as u8).map_err(|_| ())
    }

    fn is_act(&self) -> bool {
        return *self == Command::ACT;
    }
}
#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]

pub enum DDR4Org {
    DDR4_2Gb_x4,
    DDR4_2Gb_x8,
    DDR4_2Gb_x16,
    DDR4_4Gb_x4,
    DDR4_4Gb_x8,
    DDR4_4Gb_x16,
    DDR4_8Gb_x4,
    DDR4_8Gb_x8,
    DDR4_8Gb_x16,
    MAX,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Serialize, Deserialize)]

pub struct SpeedEntry {
    rate: u64,
    freq: f64,
    tCK: f64,
    nBL: u64,
    nCCDS: u64,
    nCCDL: u64,
    nRTRS: u64,
    nCL: u64,
    nRCD: u64,
    nRP: u64,
    nCWL: u64,
    nRAS: u64,
    nRC: u64,
    nRTP: u64,
    nWTRS: u64,
    nWTRL: u64,
    nWR: u64,
    nRRDS: u64,
    nRRDL: u64,
    nFAW: u64,
    nRFC: u64,
    nREFI: u64,
    nPD: u64,
    nXP: u64,
    nXPDLL: u64,
    nCKESR: u64,
    nXS: u64,
    nXSDLL: u64,
}
pub struct DDR4 {
    addr_size: Vec<usize>,
    addr_bits: Vec<usize>,
    timing: Vec<Vec<Vec<TimeEntry<Command>>>>,
    read_latency: u64,
}
#[derive(Debug, Serialize, Deserialize)]

pub enum Speed {
    DDR4_1600K,
    DDR4_1600L,
    DDR4_1866M,
    DDR4_1866N,
    DDR4_2133P,
    DDR4_2133R,
    DDR4_2400R,
    DDR4_2400U,
    DDR4_3200,
}
impl DDR4 {
    pub fn new(config: &Config) -> Self {
        tracing::info!("building ddr4");
        let channels = config.channels;
        let ranks = config.ranks;

        // not this is different than the original code, the col = origin_col -3 because we substracted the 3 bit for burst length of 8
        let addr_size = match config.ddr4_org {
            DDR4Org::DDR4_2Gb_x4 => vec![channels, ranks, 4, 4, 1 << 15, 1 << 7],
            DDR4Org::DDR4_2Gb_x8 => vec![channels, ranks, 4, 4, 1 << 14, 1 << 7],
            DDR4Org::DDR4_2Gb_x16 => vec![channels, ranks, 2, 4, 1 << 14, 1 << 7],
            DDR4Org::DDR4_4Gb_x4 => vec![channels, ranks, 4, 4, 1 << 16, 1 << 7],
            DDR4Org::DDR4_4Gb_x8 => vec![channels, ranks, 4, 4, 1 << 15, 1 << 7],
            DDR4Org::DDR4_4Gb_x16 => vec![channels, ranks, 2, 4, 1 << 15, 1 << 7],
            DDR4Org::DDR4_8Gb_x4 => vec![channels, ranks, 4, 4, 1 << 17, 1 << 7],
            DDR4Org::DDR4_8Gb_x8 => vec![channels, ranks, 4, 4, 1 << 16, 1 << 7],
            DDR4Org::DDR4_8Gb_x16 => vec![channels, ranks, 2, 4, 1 << 16, 1 << 7],
            DDR4Org::MAX => unreachable!(),
        };
        tracing::info!(?addr_size, "addr_size");
        let addr_bits = addr_size.iter().map(|x| log2(*x)).collect::<Vec<usize>>();
        tracing::info!(?addr_bits, "addr_bits");
        let speed_entry = Self::get_speed(&config.ddr4_speed);
        tracing::info!(?speed_entry, "speed_entry");
        let mut timing = vec![vec![vec![]; Command::MAX]; Level::MAX_LEVEL];
        Self::init_timing(&mut timing, &speed_entry);
        for (level, timing_level) in timing.iter().enumerate() {
            for (cmd, timing_command) in timing_level.iter().enumerate() {
                let level = Level::try_from_primitive(level as u8).unwrap();
                let cmd = Command::try_from_primitive(cmd as u8).unwrap();
                tracing::info!(?level, ?cmd, ?timing_command, "timing_command");
            }
        }
        let read_latency = speed_entry.nCL + speed_entry.nBL;
        tracing::info!(?read_latency, "read_latency");
        Self {
            addr_size,
            addr_bits,
            timing,
            read_latency,
        }
    }

    pub fn addr_from_addr_vec(&self, addr_vec: &[u64]) -> u64 {
        assert!(addr_vec.len() == self.addr_bits.len());
        let mut addr = 0;
        for i in 0..addr_vec.len() {
            addr <<= self.addr_bits[i];
            addr += addr_vec[i];
        }
        addr <<= 6;
        addr
    }

    fn init_timing(timing: &mut Vec<Vec<Vec<TimeEntry<Command>>>>, s: &SpeedEntry) {
        /*** Channel ***/
        let t = &mut timing[Level::Channel as usize];

        // CAS <-> CAS
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nBL,
            sibling: false,
        });

        /*** Rank ***/
        let t = &mut timing[Level::Rank as usize];

        // CAS <-> CAS
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCCDS,
            sibling: false,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCL + s.nBL + 2 - s.nCWL,
            sibling: false,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCL + s.nBL + 2 - s.nCWL,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCL + s.nBL + 2 - s.nCWL,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCL + s.nBL + 2 - s.nCWL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRS,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRS,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRS,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRS,
            sibling: false,
        });

        // CAS <-> CAS (between sibling ranks)
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nBL + s.nRTRS,
            sibling: true,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCL + s.nBL + s.nRTRS - s.nCWL,
            sibling: true,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCL + s.nBL + s.nRTRS - s.nCWL,
            sibling: true,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCL + s.nBL + s.nRTRS - s.nCWL,
            sibling: true,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCL + s.nBL + s.nRTRS - s.nCWL,
            sibling: true,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCWL + s.nBL + s.nRTRS - s.nCL,
            sibling: true,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nRTRS - s.nCL,
            sibling: true,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCWL + s.nBL + s.nRTRS - s.nCL,
            sibling: true,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nRTRS - s.nCL,
            sibling: true,
        });

        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::PREA,
            dist: 1,
            val: s.nRTP,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::PREA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWR,
            sibling: false,
        });

        // CAS <-> PD
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: s.nCL + s.nBL + 1,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: s.nCL + s.nBL + 1,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWR,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWR + 1,
            sibling: false,
        }); // +1 for pre
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });

        // CAS <-> SR: none (all banks have to be precharged)

        // RAS <-> RAS
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRRDS,
            sibling: false,
        });
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 4,
            val: s.nFAW,
            sibling: false,
        });
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::PREA,
            dist: 1,
            val: s.nRAS,
            sibling: false,
        });
        t[Command::PREA as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRP,
            sibling: false,
        });

        // RAS <-> REF
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nRC,
            sibling: false,
        });
        t[Command::PRE as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nRP,
            sibling: false,
        });
        t[Command::PREA as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nRP,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nRTP + s.nRP,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWR + s.nRP,
            sibling: false,
        });
        t[Command::REF as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRFC,
            sibling: false,
        });

        // RAS <-> PD
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: 1,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::PRE,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::PREA,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });

        // RAS <-> SR
        t[Command::PRE as usize].push(TimeEntry {
            cmd: Command::SRE,
            dist: 1,
            val: s.nRP,
            sibling: false,
        });
        t[Command::PREA as usize].push(TimeEntry {
            cmd: Command::SRE,
            dist: 1,
            val: s.nRP,
            sibling: false,
        });
        t[Command::SRX as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nXS,
            sibling: false,
        });

        // REF <-> REF
        t[Command::REF as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nRFC,
            sibling: false,
        });

        // REF <-> PD
        t[Command::REF as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: 1,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });

        // REF <-> SR
        t[Command::SRX as usize].push(TimeEntry {
            cmd: Command::REF,
            dist: 1,
            val: s.nXS,
            sibling: false,
        });

        // PD <-> PD
        t[Command::PDE as usize].push(TimeEntry {
            cmd: Command::PDX,
            dist: 1,
            val: s.nPD,
            sibling: false,
        });
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });

        // PD <-> SR
        t[Command::PDX as usize].push(TimeEntry {
            cmd: Command::SRE,
            dist: 1,
            val: s.nXP,
            sibling: false,
        });
        t[Command::SRX as usize].push(TimeEntry {
            cmd: Command::PDE,
            dist: 1,
            val: s.nXS,
            sibling: false,
        });

        // SR <-> SR
        t[Command::SRE as usize].push(TimeEntry {
            cmd: Command::SRX,
            dist: 1,
            val: s.nCKESR,
            sibling: false,
        });
        t[Command::SRX as usize].push(TimeEntry {
            cmd: Command::SRE,
            dist: 1,
            val: s.nXS,
            sibling: false,
        });

        /*** Bank Group ***/
        let t = &mut timing[Level::BankGroup as usize];
        // CAS <-> CAS
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nCCDL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRL,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRL,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRL,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWTRL,
            sibling: false,
        });

        // RAS <-> RAS
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRRDL,
            sibling: false,
        });

        /*** Bank ***/
        let t = &mut timing[Level::Bank as usize];

        // CAS <-> RAS
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::RD,
            dist: 1,
            val: s.nRCD,
            sibling: false,
        });
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::RDA,
            dist: 1,
            val: s.nRCD,
            sibling: false,
        });
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::WR,
            dist: 1,
            val: s.nRCD,
            sibling: false,
        });
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::WRA,
            dist: 1,
            val: s.nRCD,
            sibling: false,
        });

        t[Command::RD as usize].push(TimeEntry {
            cmd: Command::PRE,
            dist: 1,
            val: s.nRTP,
            sibling: false,
        });
        t[Command::WR as usize].push(TimeEntry {
            cmd: Command::PRE,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWR,
            sibling: false,
        });

        t[Command::RDA as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRTP + s.nRP,
            sibling: false,
        });
        t[Command::WRA as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nCWL + s.nBL + s.nWR + s.nRP,
            sibling: false,
        });

        // RAS <-> RAS
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRC,
            sibling: false,
        });
        t[Command::ACT as usize].push(TimeEntry {
            cmd: Command::PRE,
            dist: 1,
            val: s.nRAS,
            sibling: false,
        });
        t[Command::PRE as usize].push(TimeEntry {
            cmd: Command::ACT,
            dist: 1,
            val: s.nRP,
            sibling: false,
        });
    }

    pub fn get_speed(speed: &Speed) -> SpeedEntry {
        match speed {
            Speed::DDR4_1600K => SpeedEntry {
                rate: 1600,
                freq: (400.0 / 3.0) * 6.0,
                tCK: (3.0 / 0.4) / 6.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 5,
                nRTRS: 2,
                nCL: 11,
                nRCD: 11,
                nRP: 11,
                nCWL: 9,
                nRAS: 28,
                nRC: 39,
                nRTP: 6,
                nWTRS: 2,
                nWTRL: 6,
                nWR: 12,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 4,
                nXP: 5,
                nXPDLL: 0,
                nCKESR: 5,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_1600L => SpeedEntry {
                rate: 1600,
                freq: (400.0 / 3.0) * 6.0,
                tCK: (3.0 / 0.4) / 6.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 5,
                nRTRS: 2,
                nCL: 12,
                nRCD: 12,
                nRP: 12,
                nCWL: 9,
                nRAS: 28,
                nRC: 40,
                nRTP: 6,
                nWTRS: 2,
                nWTRL: 6,
                nWR: 12,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 4,
                nXP: 5,
                nXPDLL: 0,
                nCKESR: 5,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_1866M => SpeedEntry {
                rate: 1866,
                freq: (400.0 / 3.0) * 7.0,
                tCK: (3.0 / 0.4) / 7.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 5,
                nRTRS: 2,
                nCL: 13,
                nRCD: 13,
                nRP: 13,
                nCWL: 10,
                nRAS: 32,
                nRC: 45,
                nRTP: 7,
                nWTRS: 3,
                nWTRL: 7,
                nWR: 14,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 5,
                nXP: 6,
                nXPDLL: 0,
                nCKESR: 6,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_1866N => SpeedEntry {
                rate: 1866,
                freq: (400.0 / 3.0) * 7.0,
                tCK: (3.0 / 0.4) / 7.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 5,
                nRTRS: 2,
                nCL: 14,
                nRCD: 14,
                nRP: 14,
                nCWL: 10,
                nRAS: 32,
                nRC: 46,
                nRTP: 7,
                nWTRS: 3,
                nWTRL: 7,
                nWR: 14,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 5,
                nXP: 6,
                nXPDLL: 0,
                nCKESR: 6,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_2133P => SpeedEntry {
                rate: 2133,
                freq: (400.0 / 3.0) * 8.0,
                tCK: (3.0 / 0.4) / 8.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 6,
                nRTRS: 2,
                nCL: 15,
                nRCD: 15,
                nRP: 15,
                nCWL: 11,
                nRAS: 36,
                nRC: 51,
                nRTP: 8,
                nWTRS: 3,
                nWTRL: 8,
                nWR: 16,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 6,
                nXP: 7,
                nXPDLL: 0,
                nCKESR: 7,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_2133R => SpeedEntry {
                rate: 2133,
                freq: (400.0 / 3.0) * 8.0,
                tCK: (3.0 / 0.4) / 8.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 6,
                nRTRS: 2,
                nCL: 16,
                nRCD: 16,
                nRP: 16,
                nCWL: 11,
                nRAS: 36,
                nRC: 52,
                nRTP: 8,
                nWTRS: 3,
                nWTRL: 8,
                nWR: 16,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 6,
                nXP: 7,
                nXPDLL: 0,
                nCKESR: 7,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_2400R => SpeedEntry {
                rate: 2400,
                freq: (400.0 / 3.0) * 9.0,
                tCK: (3.0 / 0.4) / 9.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 6,
                nRTRS: 2,
                nCL: 16,
                nRCD: 16,
                nRP: 16,
                nCWL: 12,
                nRAS: 39,
                nRC: 55,
                nRTP: 9,
                nWTRS: 3,
                nWTRL: 9,
                nWR: 18,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 6,
                nXP: 8,
                nXPDLL: 0,
                nCKESR: 7,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_2400U => SpeedEntry {
                rate: 2400,
                freq: (400.0 / 3.0) * 9.0,
                tCK: (3.0 / 0.4) / 9.0,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 6,
                nRTRS: 2,
                nCL: 18,
                nRCD: 18,
                nRP: 18,
                nCWL: 12,
                nRAS: 39,
                nRC: 57,
                nRTP: 9,
                nWTRS: 3,
                nWTRL: 9,
                nWR: 18,
                nRRDS: 0,
                nRRDL: 0,
                nFAW: 0,
                nRFC: 0,
                nREFI: 0,
                nPD: 6,
                nXP: 8,
                nXPDLL: 0,
                nCKESR: 7,
                nXS: 0,
                nXSDLL: 0,
            },
            Speed::DDR4_3200 => SpeedEntry {
                rate: 3200,
                freq: 1600.0,
                tCK: 0.625,
                nBL: 4,
                nCCDS: 4,
                nCCDL: 10,
                nRTRS: 2,
                nCL: 22,
                nRCD: 22,
                nRP: 22,
                nCWL: 16,
                nRAS: 56,
                nRC: 78,
                nRTP: 12,
                nWTRS: 4,
                nWTRL: 12,
                nWR: 24,
                nRRDS: 8,
                nRRDL: 10,
                nFAW: 40,
                nRFC: 0,
                nREFI: 0,
                nPD: 8,
                nXP: 10,
                nXPDLL: 0,
                nCKESR: 8,
                nXS: 0,
                nXSDLL: 0,
            },
        }
    }
}
impl DramSpec for DDR4 {
    type Level = Level;

    type Command = Command;
    fn get_first_cmd(req_type: &ReqType) -> Command {
        match req_type {
            ReqType::Read => Command::RD,
            ReqType::Write => Command::WR,
        }
    }

    fn get_pre_cmd(dram: &Dram<Self>, cmd: &Command, child_id: u64) -> Option<Command> {
        match (&dram.level, cmd) {
            (Level::Rank, Command::RD) | (Level::Rank, Command::WR) => match dram.state {
                dram::State::PowerUp => None,
                dram::State::ActPowerDown => Some(Command::PDX),
                dram::State::PrePowerDown => Some(Command::PDX),
                dram::State::SelfRefresh => Some(Command::SRX),
                _ => unreachable!("invalid dram state"),
            },
            (Level::Rank, Command::REF) => {
                if dram.children.iter().any(|bank_group| {
                    bank_group
                        .children
                        .iter()
                        .any(|bank| !matches!(bank.state, dram::State::Closed))
                }) {
                    Some(Command::PREA)
                } else {
                    Some(Command::REF)
                }
            }
            (Level::Rank, Command::PDE) => match dram.state {
                dram::State::PowerUp | dram::State::ActPowerDown | dram::State::PrePowerDown => {
                    Some(Command::PDE)
                }
                dram::State::SelfRefresh => Some(Command::SRX),
                _ => unreachable!("invalid dram state"),
            },
            (Level::Rank, Command::SRE) => match dram.state {
                dram::State::PowerUp => Some(Command::SRE),
                dram::State::ActPowerDown | dram::State::PrePowerDown => Some(Command::PDX),
                dram::State::SelfRefresh => Some(Command::SRX),
                _ => unreachable!("invalid dram state"),
            },

            (Level::Bank, Command::RD) | (Level::Bank, Command::WR) => match dram.state {
                dram::State::Closed => Some(Command::ACT),
                dram::State::Opened(row_id) => {
                    tracing::debug!(row_id, child_id, "the row is open");
                    if row_id == child_id {
                        Some(*cmd)
                    } else {
                        Some(Command::PRE)
                    }
                }
                _ => unreachable!("invalid dram state"),
            },
            _ => None,
        }
    }

    fn get_start_state(level: &Level) -> dram::State {
        match level {
            Level::Channel => dram::State::NoUse,
            Level::Rank => dram::State::PowerUp,
            Level::BankGroup => dram::State::NoUse,
            Level::Bank => dram::State::Closed,
            Level::Row => dram::State::Closed,
            Level::Column => dram::State::NoUse,
        }
    }

    fn get_scope(&self, cmd: &Command) -> Level {
        match cmd {
            Command::ACT => Level::Row,
            Command::PRE => Level::Bank,
            Command::PREA => Level::Rank,
            Command::RD => Level::Column,
            Command::WR => Level::Column,
            Command::RDA => Level::Column,
            Command::WRA => Level::Column,
            Command::REF => Level::Rank,
            Command::PDE => Level::Rank,
            Command::PDX => Level::Rank,
            Command::SRE => Level::Rank,
            Command::SRX => Level::Rank,
        }
    }

    fn update_state(&self, dram: &mut Dram<Self>, cmd: &Command, child_id: u64) {
        match (dram.level, cmd) {
            (Level::Bank, Command::ACT) => {
                tracing::debug!(child_id, "ACT in Bank");
                dram.state = dram::State::Opened(child_id);
            }
            (Level::Bank, Command::PRE | Command::RDA | Command::WRA) => {
                dram.state = dram::State::Closed;
            }
            (Level::Rank, Command::PREA) => {
                dram.children.iter_mut().for_each(|bank_group| {
                    bank_group.children.iter_mut().for_each(|bank| {
                        bank.state = dram::State::Closed;
                    })
                });
            }
            (Level::Rank, Command::PDE) => {
                for bank_group in dram.children.iter_mut() {
                    for bank in bank_group.children.iter_mut() {
                        if bank.state != dram::State::Closed {
                            dram.state = State::ActPowerDown;
                            return;
                        }
                    }
                }
                dram.state = State::PrePowerDown;
            }
            (Level::Rank, Command::PDX) => {
                dram.state = State::PowerUp;
            }
            (Level::Rank, Command::SRE) => {
                dram.state = State::SelfRefresh;
            }
            (Level::Rank, Command::SRX) => {
                dram.state = State::PowerUp;
            }

            _ => {}
        }
    }
    fn get_timming(&self, level: &Level, cmd: &Command) -> &[TimeEntry<Self::Command>] {
        return &self.timing[*level as usize][*cmd as usize];
    }

    fn get_read_latency(&self) -> u64 {
        return self.read_latency;
    }

    fn get_addr_bits(&self, level: &Self::Level) -> usize {
        return self.addr_bits[level.to_usize()];
    }

    fn get_addr_size(&self, level: &Self::Level) -> usize {
        return self.addr_size[level.to_usize()];
    }

    fn decode_addr(&self, mut addr: u64, mapping_type: &MappingType) -> Vec<u64> {
        clear_lower_bits(&mut addr, 6);
        let mut addr_vec = vec![0; Level::MAX_LEVEL];
        utils::setup_addr_vec(
            addr,
            self.get_full_addr_bits(),
            &mut addr_vec,
            mapping_type.get_slice_sequence(true),
        );
        addr_vec
    }
    /// return the addr represented by the addr_vec, note: the lower 6 bits are ignored
    fn encode_addr(&self, addr: &[u64], mapping_type: &MappingType) -> u64 {
        utils::set_up_addr(
            addr,
            self.get_full_addr_bits(),
            mapping_type.get_slice_sequence(true),
        ) << 6
    }

    fn get_full_addr_bits(&self) -> &[usize] {
        &self.addr_bits
    }

    fn get_full_addr_size(&self) -> &[usize] {
        &self.addr_size
    }

    fn get_prefetch_size(&self) -> usize {
        8
    }

    fn get_channel_width(&self) -> usize {
        64
    }
}

#[cfg(test)]
mod tests {
    use crate::init_logger;

    use super::*;
    #[test]
    fn test_address() {
        init_logger();
        let config = Config::default();
        let ddr4 = DDR4::new(&config);
        let span = tracing::span!(tracing::Level::DEBUG, "testing ddr4 address");
        let _enter = span.enter();
        let addr = 1;

        tracing::debug!(addr);
        let addr_vec = ddr4.decode_addr(addr, &MappingType::RoBaRaCoCh);
        tracing::debug!("{:?}", addr_vec);
        let addr_ = ddr4.encode_addr(&addr_vec, &MappingType::RoBaRaCoCh);
        tracing::debug!("{:?}", addr_);
        assert_eq!(addr, addr_ + 1);

        let addr = 65;

        tracing::debug!(addr);
        let addr_vec = ddr4.decode_addr(addr, &MappingType::RoBaRaCoCh);
        tracing::debug!("{:?}", addr_vec);
        let addr_ = ddr4.encode_addr(&addr_vec, &MappingType::RoBaRaCoCh);
        tracing::debug!("{:?}", addr_);
        assert_eq!(addr, addr_ + 1);

        let addr = 64 * 4 + 1;

        tracing::debug!(addr);
        let addr_vec = ddr4.decode_addr(addr, &MappingType::RoBaRaCoCh);
        tracing::debug!("{:?}", addr_vec);
        let addr_ = ddr4.encode_addr(&addr_vec, &MappingType::RoBaRaCoCh);
        tracing::debug!("{:?}", addr_);
        assert_eq!(addr, addr_ + 1);
    }
}
