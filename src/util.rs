pub fn log2(x: u64) -> u64 {
    ((std::mem::size_of::<u64>() * 8) as u64) - (x.leading_zeros() as u64) - 1
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
}
