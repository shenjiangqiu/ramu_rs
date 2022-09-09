//! the utils module

/// truncate lower bits of addr and return the truncated addr
pub fn slicing_lower_bits(addr: &mut u64, bits: usize) -> u64 {
    let mask = (1 << bits) - 1;
    let lower_bits = *addr & mask;
    *addr >>= bits;
    lower_bits
}

/// ignore the lower bits of addr
pub fn clear_lower_bits(addr: &mut u64, bits: usize) {
    *addr >>= bits;
}

/// map the addr into addrvec according to addr bits and sequence
pub fn setup_addr_vec(
    mut addr: u64,
    addr_bits: &[usize],
    addr_vec: &mut [u64],
    sequence: &[usize],
) {
    assert_eq!(addr_bits.len(), addr_vec.len());
    assert_eq!(addr_bits.len(), sequence.len());

    for &level in sequence.iter() {
        addr_vec[level] = slicing_lower_bits(&mut addr, addr_bits[level]);
    }
}

pub fn set_up_addr(addr_vec: &[u64], addr_bits: &[usize], sequence: &[usize]) -> u64 {
    let mut addr = 0;
    for &level in sequence.iter().rev() {
        addr <<= addr_bits[level];
        addr |= addr_vec[level];
    }
    addr
}

#[cfg(test)]
mod tests {
    use crate::utils::{clear_lower_bits, set_up_addr, setup_addr_vec};

    #[test]
    fn test_addr() {
        let mut addr = 114514;
        let mut addr_vec = vec![0, 0, 0, 0, 0];
        let addr_bits = [2, 3, 4, 5, 6];
        let sequence = [0, 1, 2, 3, 4];
        clear_lower_bits(&mut addr, 6);
        println!("addr: {}", addr);
        setup_addr_vec(addr, &addr_bits, &mut addr_vec, &sequence);
        println!("{:?}", addr_vec);
        let ret_addr = set_up_addr(&addr_vec, &addr_bits, &sequence);
        println!("{:?}", ret_addr);
    }
}
