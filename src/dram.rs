use std::{collections::VecDeque, fmt::Debug};

use crate::{memory::MappingType, request::ReqType};

/// the latency betreen two commands
/// - `cmd`: the target command
/// - `dist`: the distance in the history that this entry should refer to
/// - `val`: the latency
/// - `sibling`: is this latency should be applied to the sibling component instead of child component
#[derive(Clone, Debug)]
pub struct TimeEntry<Command: CommandTrait> {
    pub cmd: Command,
    pub dist: usize,
    pub val: u64,
    pub sibling: bool,
}
/// the level type should be implemented for a specific dram.
pub trait LevelTrait: Sized + Debug + PartialEq + Eq + Clone + Copy {
    const MAX_LEVEL: usize;
    fn next_level(&self) -> Option<Self>;
    fn channel() -> Self;

    fn is_row(&self) -> bool;
    fn need_init_dram(&self) -> bool;
    fn is_bank(&self) -> bool;
    fn have_bank_group() -> bool;
    fn is_channel(&self) -> bool;
    fn to_usize(&self) -> usize;
}
pub trait CommandTrait: Sized + Debug + Clone + Copy + PartialEq + Eq {
    const MAX: usize;
    fn try_from_u8(val: u8) -> Result<Self, ()>;
    fn to_u8(self) -> u8;
    fn try_from_usize(val: usize) -> Result<Self, ()>;
    fn to_usize(self) -> usize;
    fn is_act(&self) -> bool;
}
pub trait LevelSlice {}
pub trait DramSpec {
    /// the type to represent a level
    type Level: LevelTrait;
    /// the type to represent a command
    type Command: CommandTrait;

    /// the bits with of a decoded addr
    /// get the first command that a req should issue
    fn get_first_cmd(req_type: &ReqType) -> Self::Command;
    /// get the pre command of a command according to current dram state
    fn get_pre_cmd(dram: &Dram<Self>, cmd: &Self::Command, child_id: u64) -> Option<Self::Command>;
    /// for some level, get the init state
    fn get_start_state(level: &Self::Level) -> State;
    /// convert a addr to a decoded addr  
    fn decode_addr(&self, addr: u64, mapping_type: &MappingType) -> Vec<u64>;
    /// convert a decoded addr to a addr
    fn encode_addr(&self, addr: &[u64], mapping_type: &MappingType) -> u64;
    /// the the bits of a level
    fn get_addr_bits(&self, level: &Self::Level) -> usize;
    /// full addr bits from high(channel) to low
    fn get_full_addr_bits(&self) -> &[usize];

    /// the the size of a level
    fn get_addr_size(&self, level: &Self::Level) -> usize;
    /// full addr size from high(channel) to low
    fn get_full_addr_size(&self) -> &[usize];

    /// get the lowest level that related to  a command
    fn get_scope(&self, cmd: &Self::Command) -> Self::Level;
    /// update the state of dram according the command
    /// - child_id the next level id for dram, like dram is `bank`, child_id is `row id`
    fn update_state(&self, dram: &mut Dram<Self>, cmd: &Self::Command, child_id: u64);
    fn get_timming(&self, level: &Self::Level, cmd: &Self::Command) -> &[TimeEntry<Self::Command>];
    fn get_read_latency(&self) -> u64;
    /// the number of reads per read req
    fn get_prefetch_size(&self) -> usize;
    /// the channel output bits
    fn get_channel_width(&self) -> usize;
}

#[derive(PartialEq, Eq)]
pub enum State {
    Opened(u64),
    Closed,
    PowerUp,
    ActPowerDown,
    PrePowerDown,
    SelfRefresh,
    NoUse,
}
pub struct Dram<T: DramSpec + ?Sized> {
    pub level: T::Level,
    pub children: Vec<Dram<T>>,
    pub state: State,
    pub next_clk: Vec<u64>,
    pub prev: Vec<VecDeque<u64>>,
    pub id: usize,
}
impl<T> Dram<T>
where
    T: DramSpec,
{
    pub fn new(spec: &T, level: T::Level, id: usize) -> Self {
        let state = T::get_start_state(&level);
        let child_level = level.next_level().unwrap();
        let mut children = vec![];
        if child_level.need_init_dram() {
            for i in 0..spec.get_addr_size(&child_level) {
                children.push(Dram::new(spec, child_level, i));
            }
        }
        let mut prev = vec![];
        for i in 0..T::Command::MAX {
            let mut tmp = VecDeque::new();

            let mut dist = 0;
            let cmd = T::Command::try_from_usize(i).unwrap();
            for time_entry in spec.get_timming(&level, &cmd) {
                dist = dist.max(time_entry.dist);
            }
            tmp.resize(dist, u64::MAX);
            prev.push(tmp);
        }
        let next_clk = vec![0; T::Command::MAX];
        Self {
            level,
            state,
            children,
            next_clk,
            prev,
            id,
        }
    }
    pub fn decode(&self, spec: &T, cmd: &T::Command, addr_vec: &[u64]) -> T::Command {
        let child_index = addr_vec[self.level.to_usize() + 1];
        if let Some(command) = T::get_pre_cmd(self, cmd, child_index) {
            tracing::debug!(level = ?self.level, ?command, "find pre command");
            return command;
        } else {
            if self.level.is_bank() {
                return *cmd;
            } else {
                return self.children[child_index as usize].decode(spec, cmd, addr_vec);
            }
        }
    }

    pub fn get_first_cmd(&self, req_type: &ReqType) -> T::Command {
        T::get_first_cmd(req_type)
    }
    pub fn update(&mut self, spec: &T, cmd: &T::Command, addr_vec: &[u64], clk: u64) {
        self.update_state(spec, cmd, addr_vec);
        self.update_timming(spec, cmd, addr_vec, clk);
    }
    fn update_state(&mut self, spec: &T, cmd: &T::Command, addr_vec: &[u64]) {
        let child_level = self.level.next_level().unwrap();
        let child_index = addr_vec[child_level.to_usize()];
        tracing::debug!(level=?self.level,?child_level,child_index,"trying to update state");
        spec.update_state(self, cmd, child_index);
        if self.level == spec.get_scope(cmd) || self.children.is_empty() {
            return;
        }
        self.children[child_index as usize].update_state(spec, cmd, addr_vec);
    }
    fn update_timming(&mut self, spec: &T, cmd: &T::Command, addr_vec: &[u64], clk: u64) {
        let target_id = addr_vec[self.level.to_usize()] as usize;
        if target_id != self.id {
            // i'm the sibling not the child, so only tigger the sibling timming
            for timing in spec.get_timming(&self.level, cmd) {
                if !timing.sibling {
                    // only tigger the sibling
                    continue;
                }
                assert!(timing.dist == 1);
                let future = clk + timing.val;
                self.next_clk[timing.cmd.to_usize()] =
                    self.next_clk[timing.cmd.to_usize()].max(future);
            }
            return;
        }

        if !self.prev[cmd.to_usize()].is_empty() {
            self.prev[cmd.to_usize()].pop_back();
            self.prev[cmd.to_usize()].push_front(clk);
        }
        for timing in spec.get_timming(&self.level, cmd) {
            if timing.sibling {
                continue;
            }
            let past = self.prev[cmd.to_usize()][timing.dist - 1 as usize];
            if past == u64::MAX {
                continue;
            }
            let future = past + timing.val;
            tracing::debug!(
                ?self.level,
                ?cmd,
                ?timing.cmd,
                current = self.next_clk[timing.cmd.to_usize()],
                future,
                "update timing"
            );
            self.next_clk[timing.cmd.to_usize()] = future.max(self.next_clk[timing.cmd.to_usize()]);
        }
        self.children.iter_mut().for_each(|child| {
            child.update_timming(spec, cmd, addr_vec, clk);
        });
    }
    /// return if the command is ok to issue
    pub fn check(&self, spec: &T, cmd: &T::Command, addr_vec: &[u64], clk: u64) -> bool {
        if clk < self.get_next_avaliable_clk(cmd) {
            return false;
        }
        if let Some(child_level) = self.level.next_level() {
            if spec.get_scope(cmd) != self.level && !self.children.is_empty() {
                let child_index = addr_vec[child_level.to_usize()];
                return self.children[child_index as usize].check(spec, cmd, addr_vec, clk);
            }
        }
        true
    }
    pub fn get_next_avaliable_clk(&self, cmd: &T::Command) -> u64 {
        self.next_clk[cmd.to_usize()]
    }
}
