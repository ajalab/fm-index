pub fn log2(x: u64) -> u64 {
    ((std::mem::size_of::<u64>() * 8) as u64) - u64::from(x.leading_zeros()) - 1
}

pub fn log2_usize(x: usize) -> usize {
    (std::mem::size_of::<usize>() * 8) - (x.leading_zeros() as usize) - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_log2() {
        assert_eq!(log2(2u64), 1);
        assert_eq!(log2(3u64), 1);
        assert_eq!(log2(4u64), 2);
        assert_eq!(log2(5u64), 2);
        assert_eq!(log2(6u64), 2);
        assert_eq!(log2(7u64), 2);
        assert_eq!(log2(8u64), 3);
    }

    #[test]
    fn test_log2_usize() {
        assert_eq!(log2_usize(2usize), 1);
        assert_eq!(log2_usize(3usize), 1);
        assert_eq!(log2_usize(4usize), 2);
        assert_eq!(log2_usize(5usize), 2);
        assert_eq!(log2_usize(6usize), 2);
        assert_eq!(log2_usize(7usize), 2);
        assert_eq!(log2_usize(8usize), 3);
    }
}
