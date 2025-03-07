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
        //     <------> <------------------>
        //     lms_len      names.len >= sa.len / 2 (Lemma 4.10)

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
                let mut same = text[p] == text[q] && types.is_bit_set(p) == types.is_bit_set(q);
                while same {
                    if text[p + d] != text[q + d]
                        || types.is_bit_set(p + d) != types.is_bit_set(q + d)
                    {
                        same = false;
                        break;
                    } else if is_lms(&types, (p + d) as u64) && is_lms(&types, (q + d) as u64) {
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
        //     sa1                 s1
        //    +-------------------+---------+
        // sa |                   |  names  |
        //    +-------------------+---------+
        //                         <------->
        //                          lms_len
        let (sa1, s1) = sa.split_at_mut(sa.len() - lms_len);
        if name < lms_len as u64 {
            // Names of LMS substrings are not unique.
            // Computes the suffix array of the names of LMS substrings into `sa1`.
            sais_sub(&s1, sa1, &IdConverter::with_size(name + 1));
        } else {
            // Names of LMS substrings are unique.
            // The suffix array of the names of LMS substrings is the same as the order of LMS substrings.
            for (i, &s) in s1.iter().enumerate() {
                sa1[s as usize] = i as u64
            }
        }

        //     sa1                 s1 (p1)
        //    +---------+---------+---------+
        // sa |names SA |         |  names  |
        //    +---------+---------+---------+
        //     <------->           <------->
        //      lms_len             lms_len
        //
        // Populate P1 (`p1`) with the positions of LMS substrings.
        let p1 = s1;
        for (j, i) in lms.into_iter().rev().enumerate() {
            p1[j] = i as u64;
        }

        //     sa1                 p1
        //    +---------+---------+---------+
        // sa |names SA |         |   P1    |
        //    +---------+---------+---------+
        //     <------->           <------->
        //      lms_len             lms_len
        //
        // Populate `sa1` with the positions of LMS substrings.
        for i in 0..lms_len {
            sa1[i] = p1[sa1[i] as usize];
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
        let sa_expected = vec![(b'\0', 0), (b'i', 1), (b'm', 9), (b'p', 11), (b's', 13)];
        for (c, expected) in sa_expected {
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
        let sa_expected = vec![(b'\0', 1), (b'i', 9), (b'm', 11), (b'p', 13), (b's', 17)];
        let converter = RangeConverter::new(b'a', b'z');
        let occs = count_chars(text, &converter);
        let bucket_end_pos = get_bucket_end_pos(&occs);
        for (c, expected) in sa_expected {
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
    fn test_panic_no_trailing_zero() {
        let text = "nozero".to_string().into_bytes();
        let converter = RangeConverter::new(b'a', b'z');
        build_suffix_array(&text, &converter);
    }

    #[test]
    #[should_panic]
    fn test_panic_too_many_trailing_zero() {
        let text = "toomanyzeros\0\0".to_string().into_bytes();
        let converter = IdConverter::with_size(std::mem::size_of::<u8>() as u64);
        build_suffix_array(&text, &converter);
    }

    #[test]
    #[should_panic]
    fn test_panic_consecutive_nulls() {
        let text = b"mm\0\0ii\0s\0\0\0sii\0ssii\0ppii\0".to_vec();
        let converter = RangeConverter::new(b'a', b'z');
        build_suffix_array(&text, &converter);
    }

    #[test]
    fn test_length_1() {
        let text = &[0u8];
        let sa_actual = build_suffix_array(text, &IdConverter::with_size(4));
        let sa_expected = build_expected_suffix_array(text);
        assert_eq!(sa_actual, sa_expected);
    }

    #[test]
    fn test_length_2() {
        let text = &[3u8, 0];
        let sa_actual = build_suffix_array(text, &IdConverter::with_size(4));
        let sa_expected = build_expected_suffix_array(text);
        assert_eq!(sa_actual, sa_expected);
    }

    #[test]
    fn test_length_4() {
        let text = &[3u8, 2, 1, 0];
        let sa_actual = build_suffix_array(text, &IdConverter::with_size(4));
        let sa_expected = build_expected_suffix_array(text);
        assert_eq!(sa_actual, sa_expected);
    }

    #[test]
    fn test_nulls() {
        let text = b"mm\0ii\0s\0sii\0ssii\0ppii\0".to_vec();
        let sa_actual = build_suffix_array(&text, &RangeConverter::new(b'a', b'z'));
        let sa_expected = build_expected_suffix_array(text);
        assert_eq!(sa_actual, sa_expected);
    }

    #[test]
    #[ignore]
    fn test_starting_with_zero() {
        let text = b"\0\0mm\0\0ii\0s\0\0\0sii\0ssii\0ppii\0".to_vec();
        let sa_actual = build_suffix_array(&text, &RangeConverter::new(b'a', b'z'));
        let sa_expected = build_expected_suffix_array(text);
        assert_eq!(sa_actual, sa_expected);
    }

    #[test]
    fn test_small() {
        let mut text = "mmiissiissiippii".to_string().into_bytes();
        text.push(0);
        let converter = RangeConverter::new(b'a', b'z');
        let sa_actual = build_suffix_array(&text, &converter);
        let sa_expected = build_expected_suffix_array(&text);
        assert_eq!(sa_actual, sa_expected, "text: {:?}", text);
    }

    #[test]
    fn test_rand_alphabets() {
        let len = 1000;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let converter = RangeConverter::new(b'a', b'z');

        for _ in 0..1000 {
            let text = build_text(|| rng.gen::<u8>() % (b'z' - b'a') + b'a', len);
            let sa_actual = build_suffix_array(&text, &converter);
            let sa_expected = build_expected_suffix_array(&text);
            assert_eq!(sa_actual, sa_expected, "text: {:?}", text);
        }
    }

    #[test]
    fn test_rand_binary_alphabets() {
        let len = 1000;
        let prob = 1.0 / 4.0;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let converter = RangeConverter::new(b'a', b'b');

        for _ in 0..1000 {
            let text = build_text(|| if rng.gen_bool(prob) { b'a' } else { b'b' }, len);
            let sa_actual = build_suffix_array(&text, &converter);
            let sa_expected = build_expected_suffix_array(&text);
            assert_eq!(sa_actual, sa_expected, "text: {:?}", text);
        }
    }

    #[test]
    fn test_rand_binary_zero_one() {
        let len = 1000;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let converter = IdConverter::with_size(256);

        for _ in 0..1000 {
            let text = build_text(|| rng.gen::<u8>() % 2, len);
            let sa_actual = build_suffix_array(&text, &converter);
            let sa_expected = build_expected_suffix_array(&text);
            assert_eq!(sa_actual, sa_expected, "text: {:?}", text);
        }
    }

    #[test]
    fn test_rand_bytes() {
        let len = 1000;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        let converter = IdConverter::new::<u8>();

        for _ in 0..1000 {
            let text = build_text(|| rng.gen::<u8>(), len);
            let sa_actual = build_suffix_array(&text, &converter);
            let sa_expected = build_expected_suffix_array(&text);
            assert_eq!(sa_actual, sa_expected, "text: {:?}", text);
        }
    }

    /// Build a text for tests using a generator function `gen`.
    fn build_text<T: Zero + Clone, F: FnMut() -> T>(mut gen: F, len: usize) -> Vec<T> {
        let mut text = vec![T::zero(); len];

        let mut prev_zero = true;
        for t in text.iter_mut().take(len - 1) {
            let mut c = gen();
            if prev_zero {
                while c.is_zero() {
                    c = gen();
                }
            }
            prev_zero = c.is_zero();
            *t = c;
        }

        while text[len - 2].is_zero() {
            text[len - 2] = gen();
        }

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
        let n = text.len();
        let suffixes = (0..n).map(|i| &text[i..n]).collect::<Vec<_>>();
        let mut sa = (0..(suffixes.len() as u64)).collect::<Vec<_>>();
        sa.sort_by_key(|i| suffixes[*i as usize]);
        sa
    }
}
