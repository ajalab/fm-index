pub fn log2(x: u64) -> usize {
    (std::mem::size_of::<usize>() * 8) - x.leading_zeros() as usize - 1
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
