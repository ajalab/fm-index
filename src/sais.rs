use fid::BitArray;
use std::fmt::Debug;

use crate::converter::Converter;

pub fn count_chars<T, C, K>(text: K, converter: &C) -> Vec<u64>
where
    T: Copy + Clone + Debug + Into<u64>,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();
    let mut occs = vec![0; converter.len() as usize];
    occs[0] = 1;
    for &c in text.iter() {
        let c: u64 = converter.convert(c).into();
        occs[c as usize] += 1;
    }

    return occs;
}

pub fn get_bucket_start_pos(occs: &[u64]) -> Vec<u64> {
    let mut sum = 0;
    let mut buckets = vec![0; occs.len()];
    for (&occ, b) in occs.iter().zip(buckets.iter_mut()) {
        *b = sum;
        sum += occ
    }
    buckets
}

pub fn get_bucket_end_pos(occs: &[u64]) -> Vec<u64> {
    let mut sum = 0;
    let mut buckets = vec![0; occs.len()];
    for (&occ, b) in occs.iter().zip(buckets.iter_mut()) {
        sum += occ;
        *b = sum;
    }
    buckets
}

fn get_types<T, K>(text: K) -> (BitArray, Vec<usize>)
where
    T: Into<u64> + Copy + Clone + Ord + Debug,
    K: AsRef<[T]>,
{
    let text = text.as_ref();
    let n = text.len();
    // true => type S, false => type L
    let mut types = BitArray::new(n + 1);
    types.set_bit(n, true);
    types.set_bit(n - 1, false);

    let mut lms = vec![n];
    let mut prev_type = false;
    for i in (0..=(n - 2)).rev() {
        let ty = text[i] < text[i + 1] || (text[i] == text[i + 1] && prev_type);
        if ty {
            types.set_bit(i, true);
        } else {
            if prev_type {
                lms.push(i + 1);
            }
        }
        prev_type = ty;
    }
    lms.reverse();
    (types, lms)
}

pub fn sais<T, C, K>(text: K, converter: &C) -> Vec<usize>
where
    T: Into<u64> + Copy + Clone + Ord + Debug,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();
    let n = text.len();
    let (types, lms) = get_types(text);
    let occs = count_chars(text, converter);
    let mut bucket_end_pos = get_bucket_end_pos(&occs);
    let mut sa = vec![usize::max_value(); n + 1];

    // Step 1.
    for &i in &lms {
        // TODO: refactor
        let c = if i == n {
            0
        } else {
            converter.convert(text[i]).into()
        };
        let k = bucket_end_pos[c as usize] as usize - 1;
        sa[k] = i;
        bucket_end_pos[c as usize] = k as u64;
    }

    // Step 2. Type-L
    let mut bucket_start_pos = get_bucket_start_pos(&occs);
    for i in 0..=n {
        let j = sa[i];
        if j != 0 && j != usize::max_value() {
            if !types.get_bit(j - 1) {
                let c = converter.convert(text[j - 1]).into() as usize;
                let p = bucket_start_pos[c] as usize;
                sa[p] = j - 1;
                bucket_start_pos[c] += 1;
            }
        }
    }

    // Step 3. Type-S
    let mut bucket_end_pos = get_bucket_end_pos(&occs);
    for i in (0..=n).rev() {
        let j = sa[i];
        if j != 0 && j != usize::max_value() {
            if types.get_bit(j - 1) {
                let c = converter.convert(text[j - 1]).into() as usize;
                let p = bucket_end_pos[c] as usize - 1;
                sa[p] = j - 1;
                bucket_end_pos[c] -= 1;
            }
        }
    }

    // Move all LMS substrings into the first items of `sa`.
    for (i, &j) in lms.iter().enumerate() {
        sa[i] = sa[j];
    }

    for i in lms.len()..=n {
        sa[i] = usize::max_value();
    }

    for i in 0..lms.len() {
        let j = sa[i];
        let diff = false;
        for d in 0..=n {}
    }
    println!("sa: {:?}", sa);

    let suffixes = (0..=n).map(|i| &text[i..n]).collect::<Vec<_>>();
    let mut sa = (0..suffixes.len()).collect::<Vec<_>>();
    sa.sort_by_key(|i| suffixes[*i]);
    return sa;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::RangeConverter;

    #[test]
    fn test_get_types() {
        let text = "mmiissiissiippii";
        let type_ans = "LLSSLLSSLLSSLLLLS";
        let _lms_ans = "  *   *   *     *";
        let (types, _lms) = get_types(text);
        for (i, &a) in type_ans.as_bytes().iter().enumerate() {
            assert_eq!(types.get_bit(i), a == b'S');
        }
        // TODO: write a test for lms
    }

    #[test]
    fn test_get_bucket_start_pos() {
        let text = "mmiissiissiippii";
        let converter = RangeConverter::new(b'a', b'z');
        let occs = count_chars(text, &converter);
        let bucket_start_pos = get_bucket_start_pos(&occs);
        let ans = vec![(b'\0', 0), (b'i', 1), (b'm', 9), (b'p', 11), (b's', 13)];
        for (c, expected) in ans {
            let actual = bucket_start_pos[converter.convert(c) as usize];
            assert_eq!(
                actual, expected,
                "bucket_start_pos['{}'] should be {} but {}",
                c as char, expected, actual
            );
        }
    }

    #[test]
    fn test_get_bucket_end_pos() {
        let text = "mmiissiissiippii";
        let ans = vec![(b'\0', 1), (b'i', 9), (b'm', 11), (b'p', 13), (b's', 17)];
        let converter = RangeConverter::new(b'a', b'z');
        let occs = count_chars(text, &converter);
        let bucket_end_pos = get_bucket_end_pos(&occs);
        for (c, expected) in ans {
            let actual = bucket_end_pos[converter.convert(c) as usize];
            assert_eq!(
                actual, expected,
                "bucket_end_pos['{}'] should be {} but {}",
                c as char, expected, actual
            );
        }
    }

    #[test]
    fn test_sais() {
        let text = "mmiissiissiippii";
        let converter = RangeConverter::new(b'a', b'z');
        let _ = sais(text, &converter);
    }
}
