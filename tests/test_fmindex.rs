use fm_index::{converter::RangeConverter, FMIndex, FMIndexWithLocate};

#[test]
fn test_small() {
    let text = "mississippi".to_string().into_bytes();
    let ans = vec![
        ("m", vec![0]),
        ("mi", vec![0]),
        ("i", vec![1, 4, 7, 10]),
        ("iss", vec![1, 4]),
        ("ss", vec![2, 5]),
        ("p", vec![8, 9]),
        ("ppi", vec![8]),
        ("z", vec![]),
        ("pps", vec![]),
    ];

    let fm_index = FMIndexWithLocate::new(text, RangeConverter::new(b'a', b'z'), 2);

    for (pattern, positions) in ans {
        let search = fm_index.search(pattern);
        let expected = positions.len() as u64;
        let actual = search.count();
        assert_eq!(
            expected,
            actual,
            "pattern \"{}\" must occur {} times, but {}: {:?}",
            pattern,
            expected,
            actual,
            search.locate()
        );
        let mut res = search.locate();
        res.sort();
        assert_eq!(res, positions);
    }
}

#[test]
fn test_small_contain_null() {
    let text = "miss\0issippi\0".to_string().into_bytes();
    let fm_index = FMIndex::new(text, RangeConverter::new(b'a', b'z'));

    assert_eq!(fm_index.search("m").count(), 1);
    assert_eq!(fm_index.search("ssi").count(), 1);
    assert_eq!(fm_index.search("iss").count(), 2);
    assert_eq!(fm_index.search("p").count(), 2);
    assert_eq!(fm_index.search("\0").count(), 2);
    assert_eq!(fm_index.search("\0i").count(), 1);
}

#[test]
fn test_utf8() {
    let text = "みんなみんなきれいだな"
        .chars()
        .map(|c| c as u32)
        .collect::<Vec<u32>>();
    let ans = vec![
        ("み", vec![0, 3]),
        ("みん", vec![0, 3]),
        ("な", vec![2, 5, 10]),
    ];
    let fm_index = FMIndexWithLocate::new(text, RangeConverter::new('あ' as u32, 'ん' as u32), 2);

    for (pattern, positions) in ans {
        let pattern: Vec<u32> = pattern.chars().map(|c| c as u32).collect();
        let search = fm_index.search(pattern);
        assert_eq!(search.count(), positions.len() as u64);
        let mut res = search.locate();
        res.sort();
        assert_eq!(res, positions);
    }
}

#[test]
fn test_search_backward() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
    let word_pairs = vec![("ipsum", " dolor"), ("sit", " amet"), ("sed", " do")];
    let fm_index = FMIndexWithLocate::new(text, RangeConverter::new(b' ', b'~'), 2);
    for (fst, snd) in word_pairs {
        let search1 = fm_index.search(snd).search(fst);
        let concat = fst.to_owned() + snd;
        let search2 = fm_index.search(&concat);
        assert!(search1.count() > 0);
        assert_eq!(search1.count(), search2.count());
        assert_eq!(search1.locate(), search2.locate());
    }
}

#[test]
fn test_iter_backward() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
    let index = FMIndex::new(text, RangeConverter::new(b' ', b'~'));
    let search = index.search("sit ");
    let mut prev_seq = search.iter_backward(0).take(6).collect::<Vec<_>>();
    prev_seq.reverse();
    assert_eq!(prev_seq, b"dolor ".to_owned());
}

#[test]
fn test_iter_forward() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
    let index = FMIndex::new(text, RangeConverter::new(b' ', b'~'));
    let search = index.search("sit ");
    let next_seq = search.iter_forward(0).take(10).collect::<Vec<_>>();
    assert_eq!(next_seq, b"sit amet, ".to_owned());
}
