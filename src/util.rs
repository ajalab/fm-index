pub fn log2_usize(x: usize) -> usize {
    (std::mem::size_of::<usize>() * 8) - (x.leading_zeros() as usize) - 1
}

#[cfg(test)]
mod tests {
    use super::*;

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
