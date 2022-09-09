#[derive(Debug, Clone, Copy, enum_as_inner::EnumAsInner)]
pub enum ReqType {
    Read,
    Write,
}
#[derive(Debug)]
pub struct Request {
    pub addr: u64,
    pub addr_vec: Vec<u64>,
    pub done_setup: bool,
    pub req_type: ReqType,
    pub arrival_time: u64,
    pub finish_time: u64,
}
impl Request {
    pub fn new(addr: u64, req_type: ReqType) -> Self {
        Self {
            addr,
            addr_vec: Vec::new(),
            done_setup: false,
            req_type,
            arrival_time: 0,
            finish_time: 0,
        }
    }
    pub fn new_read(addr: u64) -> Self {
        Self::new(addr, ReqType::Read)
    }
    pub fn new_write(addr: u64) -> Self {
        Self::new(addr, ReqType::Write)
    }
}
