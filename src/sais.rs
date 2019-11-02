use fid::BitArray;
use std::fmt::Debug;

use crate::converter::{Converter, IdConverter};

pub fn count_chars<T, C, K>(text: K, converter: &C) -> Vec<u64>
where
    T: Copy + Clone + Into<u64>,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();
    let mut occs = vec![0; converter.len() as usize];
    for &c in text.iter() {
        let c: u64 = converter.convert(c).into();
        occs[c as usize] += 1;
    }

    occs
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
    T: Copy + Clone + Ord,
    K: AsRef<[T]>,
{
    let text = text.as_ref();
    let n = text.len();
    // true => type S, false => type L
    let mut types = BitArray::new(n);
    types.set_bit(n - 2, false);
    types.set_bit(n - 1, true);

    let mut lms = vec![n - 1];
    let mut prev_type = false;
    for i in (0..(n - 1)).rev() {
        let ty = text[i] < text[i + 1] || (text[i] == text[i + 1] && prev_type);
        if ty {
            types.set_bit(i, true);
        } else if prev_type {
            lms.push(i + 1);
        }
        prev_type = ty;
    }
    (types, lms)
}

fn is_lms(types: &BitArray, i: u64) -> bool {
    i > 0 && i < u64::max_value() && types.get_bit(i as usize) && !types.get_bit(i as usize - 1)
}

fn induced_sort<T, K, C>(text: K, converter: &C, types: &BitArray, occs: &[u64], sa: &mut [u64])
where
    T: Into<u64> + Copy + Clone + Ord,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();
    let n = text.len();
    let mut bucket_start_pos = get_bucket_start_pos(occs);
    for i in 0..n {
        let j = sa[i];
        if 0 < j && j < u64::max_value() && !types.get_bit(j as usize - 1) {
            let c = converter.convert(text[j as usize - 1]).into() as usize;
            let p = bucket_start_pos[c] as usize;
            sa[p] = j - 1;
            bucket_start_pos[c] += 1;
        }
    }

    let mut bucket_end_pos = get_bucket_end_pos(&occs);
    for i in (0..n).rev() {
        let j = sa[i];
        if j != 0 && j != u64::max_value() && types.get_bit(j as usize - 1) {
            let c = converter.convert(text[j as usize - 1]).into() as usize;
            let p = bucket_end_pos[c] as usize - 1;
            sa[p] = j - 1;
            bucket_end_pos[c] -= 1;
        }
    }
}

pub fn sais<T, C, K>(text: K, converter: &C) -> Vec<u64>
where
    T: Into<u64> + Copy + Clone + Ord + Debug,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let mut sa = vec![u64::max_value(); text.as_ref().len()];
    sais_sub(&text, &mut sa, converter);
    sa
}

fn sais_sub<T, C, K>(text: K, sa: &mut [u64], converter: &C)
where
    T: Into<u64> + Copy + Clone + Ord + Debug,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();

    debug_assert!(text.last().map(|&c| c.into()) == Some(0u64));

    let n = text.len();
    let (types, lms) = get_types(text);
    let lms_len = lms.len();
    let occs = count_chars(text, converter);
    let mut bucket_end_pos = get_bucket_end_pos(&occs);

    // Step 1.
    for &i in lms.iter().rev() {
        // TODO: refactor
        let c = if i == n {
            0
        } else {
            converter.convert(text[i]).into()
        };
        let k = bucket_end_pos[c as usize] as usize - 1;
        sa[k] = i as u64;
        bucket_end_pos[c as usize] = k as u64;
    }

    // Step 2. Type-L
    // Step 3. Type-S
    induced_sort(text, converter, &types, &occs, sa);

    // Move all sorted LMS substrings into the first items of `sa`.
    let mut k = 0;
    for i in 0..n {
        let p = sa[i];
        if is_lms(&types, p) {
            sa[k] = p;
            k += 1;
            if k == lms_len {
                break;
            }
        }
    }

    let mut name = 1;
    {
        let (sa0, sa1) = sa.split_at_mut(lms_len);
        for s in sa1.iter_mut() {
            *s = u64::max_value();
        }

        sa1[sa0[0] as usize / 2] = 0; // name of the sentinel
        if lms_len <= 1 {
            debug_assert!(lms_len != 0);
        } else {
            sa1[sa0[1] as usize / 2] = 1; // name of the second least LMS substring
            for i in 2..lms_len {
                let p = sa0[i - 1] as usize;
                let q = sa0[i] as usize;
                let mut d = 1;
                let mut same = text[p] == text[q] && types.get_bit(p) == types.get_bit(q);
                while same {
                    if text[p + d] != text[q + d] || types.get_bit(p + d) != types.get_bit(q + d) {
                        same = false;
                    } else if is_lms(&types, (p + d) as u64) && is_lms(&types, (p + d) as u64) {
                        break;
                    }
                    d += 1;
                }
                if !same {
                    name += 1;
                }
                sa1[q / 2] = name;
            }
        }
        for s in sa0.iter_mut() {
            *s = u64::max_value();
        }
    }
    let mut i = sa.len() - 1;
    let mut j = 0;
    while j < lms_len {
        if sa[i] < u64::max_value() {
            sa[sa.len() - 1 - j] = sa[i];
            sa[i] = u64::max_value();
            j += 1;
        }
        i -= 1;
    }
    {
        let (sa1, s1) = sa.split_at_mut(sa.len() - lms_len);
        if name < lms_len as u64 {
            sais_sub(&s1[..s1.len()], sa1, &IdConverter::new(name + 1 as u64));
        } else {
            for (i, &s) in s1.iter().enumerate() {
                sa1[s as usize] = i as u64
            }
        }
        for (j, i) in lms.into_iter().rev().enumerate() {
            s1[j] = i as u64;
        }
        for i in 0..lms_len {
            sa1[i] = s1[sa1[i] as usize];
        }
    }
    for i in &mut sa[lms_len..] {
        *i = u64::max_value();
    }

    let mut bucket_end_pos = get_bucket_end_pos(&occs);
    for i in (0..lms_len).rev() {
        let j = sa[i] as usize;
        sa[i] = u64::max_value();
        let c = if j == n {
            0
        } else {
            converter.convert(text[j]).into()
        };
        let k = bucket_end_pos[c as usize] as usize - 1;
        sa[k] = j as u64;
        bucket_end_pos[c as usize] = k as u64;
    }
    induced_sort(text, converter, &types, &occs, sa);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::RangeConverter;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn test_get_types() {
        let text = "mmiissiissiippii\0";
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
        let text = "mmiissiissiippii\0";
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
        let text = "mmiissiissiippii\0";
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
    fn test_sais1() {
        let mut text = "mmiissiissiippii".to_string().into_bytes();
        text.push(0);
        let converter = RangeConverter::new(b'a', b'z');
        let sa = sais(&text, &converter);
        let ans = get_suffix_array(text);

        assert_eq!(sa.len(), ans.len());
        for (i, (actual, expected)) in sa.into_iter().zip(ans.into_iter()).enumerate() {
            assert_eq!(
                actual, expected,
                "wrong at {}-th pos: expected {}, but actual {}",
                i, expected, actual
            );
        }
    }

    #[test]
    fn test_sais2() {
        let text = &[2u64, 2, 1, 0];
        let converter = IdConverter::new(3);
        let sa = sais(&text, &converter);
        let ans = get_suffix_array(text);

        assert_eq!(sa.len(), ans.len());
        for (i, (actual, expected)) in sa.into_iter().zip(ans.into_iter()).enumerate() {
            assert_eq!(
                actual, expected,
                "wrong at {}-th pos: expected {}, but actual {}",
                i, expected, actual
            );
        }
    }

    #[test]
    fn test_sais_rand() {
        let len = 100_000;
        let prob = 1.0 / 4.0;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let mut text = (0..len)
            .map(|_| if rng.gen_bool(prob) { b'a' } else { b'b' })
            .collect::<Vec<_>>();
        text.push(0);

        let converter = RangeConverter::new(b'a', b'b');
        let sa = sais(&text, &converter);
        let ans = get_suffix_array(&text);
        assert_eq!(sa.len(), ans.len());
        for (i, (actual, expected)) in sa.into_iter().zip(ans.into_iter()).enumerate() {
            assert_eq!(
                actual, expected,
                "wrong at {}-th pos: expected {}, but actual {}",
                i, expected, actual
            );
        }
    }

    fn get_suffix_array<K: AsRef<[T]>, T: Copy + Clone + Ord>(text: K) -> Vec<u64> {
        let text = text.as_ref();
        let n = text.len();
        let suffixes = (0..n).map(|i| &text[i..n]).collect::<Vec<_>>();
        let mut sa = (0..(suffixes.len() as u64)).collect::<Vec<_>>();
        sa.sort_by_key(|i| suffixes[*i as usize]);
        sa
    }
}
