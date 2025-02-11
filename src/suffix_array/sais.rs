//! SA-IS implementation based on
//!    Ge Nong, Sen Zhang, & Wai Hong Chan. (2010). Two Efficient Algorithms for Linear Time Suffix Array Construction.
//!    IEEE Transactions on Computers, 60(10), 1471–1484. <https://doi.org/10.1109/tc.2010.188>
use vers_vecs::BitVec;

use crate::{
    converter::{Converter, IdConverter},
    Character,
};

pub fn count_chars<T, C, K>(text: K, converter: &C) -> Vec<u64>
where
    T: Character,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();
    let mut occs = vec![0; converter.len() as usize];
    for &c in text.iter() {
        let c = converter.convert(c).into() as usize;
        occs[c] += 1;
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

fn get_types<T, K>(text: K) -> (BitVec, Vec<usize>)
where
    T: Copy + Clone + Ord,
    K: AsRef<[T]>,
{
    let text = text.as_ref();
    let n = text.len();
    // 1 => S-Type, 0 => L-Type
    let mut types = BitVec::from_zeros(n);
    types.set(n - 1, 1).unwrap();

    if n == 1 {
        return (types, vec![]);
    }

    let mut lms = vec![n - 1];
    let mut prev_is_s_type = false;
    for i in (0..(n - 1)).rev() {
        // text[i] is S-type if either holds:
        //     - text[i] <  text[i + 1]
        //     - text[i] == text[i + 1] and text[i + 1] is S-type.
        // Otherwise, text[i] is L-type.
        // Notably, text[i] is S-type if text[i] is zero in a multi-text.
        let is_s_type = text[i] < text[i + 1] || (text[i] == text[i + 1] && prev_is_s_type);
        if is_s_type {
            types.set(i, 1).unwrap();
        } else if prev_is_s_type {
            // text[i + 1] is LMS-type (leftmost-S) if text[i] is L-type and text[i + 1] is S-type.
            lms.push(i + 1);
        }
        prev_is_s_type = is_s_type;
    }
    (types, lms)
}

fn is_lms(types: &BitVec, i: u64) -> bool {
    i > 0
        && i < u64::MAX
        && types.is_bit_set(i as usize).unwrap()
        && !types.is_bit_set(i as usize - 1).unwrap()
}

fn induced_sort<T, K, C>(text: K, converter: &C, types: &BitVec, occs: &[u64], sa: &mut [u64])
where
    T: Character,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();
    let n = text.len();
    let mut bucket_start_pos = get_bucket_start_pos(occs);
    for i in 0..n {
        let j = sa[i];
        if 0 < j && j < u64::MAX && !types.is_bit_set(j as usize - 1).unwrap() {
            let c = converter.convert(text[j as usize - 1]).into() as usize;
            let p = bucket_start_pos[c] as usize;
            sa[p] = j - 1;
            bucket_start_pos[c] += 1;
        }
    }

    let mut bucket_end_pos = get_bucket_end_pos(occs);
    for i in (0..n).rev() {
        let j = sa[i];
        if j != 0 && j != u64::MAX && types.is_bit_set(j as usize - 1).unwrap() {
            let c = converter.convert(text[j as usize - 1]).into() as usize;
            let p = bucket_end_pos[c] as usize - 1;
            sa[p] = j - 1;
            bucket_end_pos[c] -= 1;
        }
    }

    // After the induced sort, positions of end markers were placed at the first bucket of `sa`.
    // This rearranges them so that the end markers appear in the same order as the original text.
    let mut k = 0;
    for (i, &c) in text.iter().enumerate() {
        if c.into() == 0 {
            sa[k] = i as u64;
            k += 1;
        }
    }
}

/// Build a suffix array from the given [`text`] using SA-IS algorithm.
pub fn build_suffix_array<T, C, K>(text: K, converter: &C) -> Vec<u64>
where
    T: Character,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let n = text.as_ref().len();
    match n {
        0 => vec![],
        1 => vec![0],
        _ => {
            debug_assert_eq!(
                text.as_ref().iter().rposition(|&c| c.into() != 0u64),
                Some(text.as_ref().len() - 2),
                "the given text must end with a single 0.",
            );
            let mut sa = vec![u64::MAX; n];
            sais_sub(&text, &mut sa, converter);
            sa
        }
    }
}

#[allow(clippy::cognitive_complexity)]
fn sais_sub<T, C, K>(text: K, sa: &mut [u64], converter: &C)
where
    T: Character,
    K: AsRef<[T]>,
    C: Converter<T>,
{
    let text = text.as_ref();

    let n = text.len();
    let (types, lms) = get_types(text);
    let lms_len = lms.len();
    let occs = count_chars(text, converter);

    // Step 1.
    let mut bucket_end_pos = get_bucket_end_pos(&occs);
    for &i in lms.iter().rev() {
        // TODO: refactor
        let c = converter.convert(text[i]).into();
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
        // Put lexicographic names of LMS substrings into `names`
        // in the order of SA.
        //
        //      sa_lms         names
        //    +--------+--------------------+
        // sa |        |**n0**n1************|
        //    +--------+--------------------+
        //    <--------><------------------->
        //     lms_len      names.len <= sa.len / 2 (Lemma 4.10)

        let (sa_lms, names) = sa.split_at_mut(lms_len);
        for n in names.iter_mut() {
            *n = u64::MAX;
        }
        names[sa_lms[0] as usize / 2] = 0; // name of the sentinel
        if lms_len <= 1 {
            debug_assert!(lms_len != 0);
        } else {
            names[sa_lms[1] as usize / 2] = 1; // name of the second least LMS substring
            for i in 2..lms_len {
                let p = sa_lms[i - 1] as usize;
                let q = sa_lms[i] as usize;
                let mut d = 1;
                // Zeros must be considered different characters.
                let mut same = text[p].into() != 0
                    && text[p].into() != 0
                    && text[p] == text[q]
                    && types.is_bit_set(p) == types.is_bit_set(q);
                while same {
                    if text[p + d].into() == 0
                        || text[q + d].into() == 0
                        || text[p + d] != text[q + d]
                        || types.is_bit_set(p + d) != types.is_bit_set(q + d)
                    {
                        same = false;
                        break;
                    } else if is_lms(&types, (p + d) as u64) && is_lms(&types, (p + d) as u64) {
                        break;
                    }
                    d += 1;
                }
                if !same {
                    name += 1;
                }
                names[q / 2] = name;
            }
        }
        for s in sa_lms.iter_mut() {
            *s = u64::MAX;
        }
    }
    let mut i = sa.len() - 1;
    let mut j = 0;
    while j < lms_len {
        if sa[i] < u64::MAX {
            sa[sa.len() - 1 - j] = sa[i];
            j += 1;
        }
        i -= 1;
    }

    {
        let (sa1, s1) = sa.split_at_mut(sa.len() - lms_len);
        if name < lms_len as u64 {
            sais_sub(&s1, sa1, &IdConverter::with_size(name + 1));
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
        *i = u64::MAX;
    }

    let mut bucket_end_pos = get_bucket_end_pos(&occs);
    for i in (0..lms_len).rev() {
        let j = sa[i] as usize;
        sa[i] = u64::MAX;
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
    use num_traits::Zero;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    fn marks_to_lms(s: &str) -> Vec<usize> {
        s.as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, &c)| c == b'*')
            .map(|(i, _)| i)
            .rev()
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_get_types() {
        let text = "mmiissiissiippii\0";
        let n = text.len();
        let types_expected = "LLSSLLSSLLSSLLLLS";
        let lms_expected = marks_to_lms("  *   *   *     *");
        let (types, lms) = get_types(text);
        let types_actual = (0..n)
            .map(|i| {
                if types.is_bit_set(i).unwrap() {
                    'S'
                } else {
                    'L'
                }
            })
            .collect::<String>();

        assert_eq!(types_expected, types_actual);
        assert_eq!(lms_expected, lms);
    }

    #[test]
    fn test_get_types_zeros() {
        let text = "m\0\0a\0";
        let n = text.len();
        let types_expected = "LSSLS".to_string();
        let lms_expected = marks_to_lms(" *  *");
        let (types, lms) = get_types(text);
        let types_actual = (0..n)
            .map(|i| {
                if types.is_bit_set(i).unwrap() {
                    'S'
                } else {
                    'L'
                }
            })
            .collect::<String>();

        assert_eq!(types_expected, types_actual);
        assert_eq!(lms_expected, lms);
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
    #[should_panic]
    fn test_sais_no_trailing_zero() {
        let text = "nozero".to_string().into_bytes();
        let converter = RangeConverter::new(b'a', b'z');
        build_suffix_array(&text, &converter);
    }

    #[test]
    #[should_panic]
    fn test_sais_too_many_trailing_zero() {
        let text = "toomanyzeros\0\0".to_string().into_bytes();
        let converter = IdConverter::with_size(std::mem::size_of::<u8>() as u64);
        build_suffix_array(&text, &converter);
    }

    #[test]
    fn test_sais_1() {
        let text = &[0u8];
        let sa = build_suffix_array(text, &IdConverter::with_size(4));
        let expected = build_expected_suffix_array(text);
        assert_eq!(sa, expected);
    }

    #[test]
    fn test_sais_2() {
        let text = &[3u8, 0];
        let sa = build_suffix_array(text, &IdConverter::with_size(4));
        let expected = build_expected_suffix_array(text);
        assert_eq!(sa, expected);
    }

    #[test]
    fn test_sais_4() {
        let text = &[3u8, 2, 1, 0];
        let sa = build_suffix_array(text, &IdConverter::with_size(4));
        let expected = build_expected_suffix_array(text);
        assert_eq!(sa, expected);
    }

    #[test]
    fn test_sais_with_nulls() {
        let text = b"mm\0ii\0s\0sii\0ssii\0ppii\0".to_vec();
        let sa = build_suffix_array(&text, &RangeConverter::new(b'a', b'z'));
        let expected = build_expected_suffix_array(text);
        assert_eq!(sa, expected);
    }

    #[test]
    fn test_sais_with_consecutive_nulls() {
        let text = b"mm\0\0ii\0s\0\0\0sii\0ssii\0ppii\0".to_vec();
        let sa = build_suffix_array(&text, &RangeConverter::new(b'a', b'z'));
        let expected = build_expected_suffix_array(text);
        assert_eq!(sa, expected);
    }

    #[test]
    fn test_sais_starting_with_zero() {
        let text = b"\0\0mm\0\0ii\0s\0\0\0sii\0ssii\0ppii\0".to_vec();
        let sa = build_suffix_array(&text, &RangeConverter::new(b'a', b'z'));
        let expected = build_expected_suffix_array(text);
        assert_eq!(sa, expected);
    }

    #[test]
    fn test_sais_small() {
        let mut text = "mmiissiissiippii".to_string().into_bytes();
        text.push(0);
        let converter = RangeConverter::new(b'a', b'z');
        let sa = build_suffix_array(&text, &converter);
        let ans = build_expected_suffix_array(text);

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
    fn test_sais_rand_alphabets() {
        let len = 10000;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let text = build_text(|| rng.gen::<u8>() % (b'z' - b'a') + b'a', len);

        let converter = RangeConverter::new(b'a', b'z');
        let sa = build_suffix_array(&text, &converter);
        let ans = build_expected_suffix_array(&text);
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
    fn test_sais_rand_binary() {
        let len = 10000;
        let prob = 1.0 / 4.0;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let text = build_text(|| if rng.gen_bool(prob) { b'a' } else { b'b' }, len);

        let converter = RangeConverter::new(b'a', b'b');
        let sa = build_suffix_array(&text, &converter);
        let ans = build_expected_suffix_array(&text);
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
    fn test_sais_rand_nulls() {
        let len = 20;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let text = build_text(|| rng.gen::<u8>() % 2, len);

        let converter = IdConverter::with_size(256);
        let sa = build_suffix_array(&text, &converter);
        let ans = build_expected_suffix_array(&text);
        assert_eq!(sa, ans);
        assert_eq!(sa.len(), ans.len());
        for (i, (actual, expected)) in sa.into_iter().zip(ans.into_iter()).enumerate() {
            assert_eq!(
                actual, expected,
                "wrong at {}-th pos: expected {}, but actual {}",
                i, expected, actual
            );
        }
    }

    /// Build a text for tests using a generator function `gen`.
    fn build_text<T: Zero, F: FnMut() -> T>(mut gen: F, len: usize) -> Vec<T> {
        let mut text = (0..(len - 1)).map(|_| gen()).collect::<Vec<_>>();

        // Truncate the trailing zeros, since SA-IS does not support trailing zero suffix longer than 1.
        if let Some(pos) = text.iter().rposition(|x| !x.is_zero()) {
            text.truncate(pos + 1);
        } else {
            text.clear();
        }

        // Add non-zero elements until the text length reaches len - 1.
        while text.len() < len - 1 {
            let c = gen();
            if !c.is_zero() {
                text.push(c);
            }
        }

        // Add the last zero as a sentinel for SA-IS.
        text.push(T::zero());
        text
    }

    /// Compute the suffix array of the given text in naive way for testing purpose.
    /// This algorithm is aware of the order of end markers (zeros).
    fn build_expected_suffix_array<T, K>(text: K) -> Vec<u64>
    where
        T: Character,
        K: AsRef<[T]>,
    {
        let text = text.as_ref();
        let suffixes = (0..text.len())
            .map(|i| {
                text[i..]
                    .iter()
                    .enumerate()
                    .map(|(j, c)| if c.is_zero() { (c, i + j) } else { (c, 0) })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut sa = (0..text.len() as u64).collect::<Vec<_>>();
        sa.sort_by(|i, j| suffixes[*i as usize].cmp(&suffixes[*j as usize]));
        sa
    }
}
