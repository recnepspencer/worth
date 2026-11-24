use std::collections::BTreeSet;

use super::{OccurrencePath, OccurrencePathDenial};

fn instance(index: u32) -> OccurrencePath {
    OccurrencePath::root("pattern_1")
        .and_then(|path| path.index("instance", index))
        .and_then(|path| path.child("hole"))
        .expect("a well-formed path")
}

#[test]
fn indexes_order_by_number_where_their_rendered_keys_order_by_character() {
    assert!(instance(10) > instance(9) && instance(9) > instance(2));
    let mut sorted = vec![instance(10), instance(2), instance(9), instance(100)];
    sorted.sort();
    let indexes: Vec<_> = sorted
        .iter()
        .map(|path| path.segments()[1].index())
        .collect();
    assert_eq!(indexes, [Some(2), Some(9), Some(10), Some(100)]);
    // The rendered keys would have sorted the other way.
    assert!(instance(10).to_occurrence_key() < instance(9).to_occurrence_key());

    // An unindexed occurrence sorts before every repetition of the same name,
    // and a path before the paths below it.
    let root = OccurrencePath::root("pattern_1").unwrap();
    let unindexed = root.clone().child("instance").unwrap();
    let first = root.clone().index("instance", 0).unwrap();
    assert!(root < unindexed && unindexed < first && first < instance(0));
}

#[test]
fn a_path_renders_one_key_and_parses_back_from_it() {
    let path = instance(2);
    assert_eq!(path.to_occurrence_key(), "pattern_1/instance[2]/hole");
    assert_eq!(path.to_string(), path.to_occurrence_key());
    assert_eq!(String::from(&path), path.to_occurrence_key());
    assert_eq!(String::from(path.clone()), "pattern_1/instance[2]/hole");
    assert_eq!("pattern_1/instance[2]/hole".parse(), Ok(path.clone()));

    let names: Vec<_> = path.segments().iter().map(|s| s.name()).collect();
    assert_eq!(names, ["pattern_1", "instance", "hole"]);
    let greatest = OccurrencePath::root("f")
        .unwrap()
        .index("i", u32::MAX)
        .unwrap();
    assert_eq!(greatest.to_occurrence_key(), "f/i[4294967295]");
    assert_eq!(OccurrencePath::parse("f/i[4294967295]"), Ok(greatest));
}

/// Every path over a small alphabet of segments, to depth three.
fn small_paths() -> Vec<OccurrencePath> {
    let steps: [(&str, Option<u32>); 6] = [
        ("a", None),
        ("a", Some(0)),
        ("a", Some(1)),
        ("a", Some(10)),
        ("a1", None),
        ("a0", Some(1)),
    ];
    let extend = |path: OccurrencePath, (name, index): (&str, Option<u32>)| match index {
        Some(index) => path.index(name, index).unwrap(),
        None => path.child(name).unwrap(),
    };
    let mut paths = Vec::new();
    for root in ["a", "a1", "b"] {
        let root = OccurrencePath::root(root).unwrap();
        paths.push(root.clone());
        for first in steps {
            let one = extend(root.clone(), first);
            paths.push(one.clone());
            paths.extend(steps.map(|second| extend(one.clone(), second)));
        }
    }
    paths
}

#[test]
fn distinct_paths_never_share_a_key_and_every_key_reads_back() {
    let paths = small_paths();
    let distinct: BTreeSet<_> = paths.iter().cloned().collect();
    let keys: BTreeSet<_> = paths
        .iter()
        .map(OccurrencePath::to_occurrence_key)
        .collect();
    assert_eq!(distinct.len(), paths.len());
    assert_eq!(keys.len(), paths.len(), "two paths rendered one key");
    for path in &paths {
        let key = path.to_occurrence_key();
        assert_eq!(OccurrencePath::parse(&key).as_ref(), Ok(path), "{key}");
        // The key is what every occurrence check downstream accepts: not
        // blank, and unchanged by trimming.
        assert!(!key.trim().is_empty() && key.trim() == key);
    }
}

#[test]
fn a_segment_that_would_make_the_key_ambiguous_is_refused() {
    let invalid = |segment: &str| {
        Err(OccurrencePathDenial::InvalidSegment {
            segment: segment.to_owned(),
        })
    };
    assert_eq!(
        OccurrencePath::root(""),
        Err(OccurrencePathDenial::EmptySegment)
    );
    for segment in [
        "a/b", "a[1]", "a]", "[", " a", "a b", "a\t", "a\n", "\u{a0}a", "a\u{7}",
    ] {
        assert_eq!(
            OccurrencePath::root(segment),
            invalid(segment),
            "{segment:?}"
        );
        let root = OccurrencePath::root("f").unwrap();
        assert_eq!(root.clone().child(segment), invalid(segment), "{segment:?}");
        assert_eq!(root.index(segment, 1), invalid(segment), "{segment:?}");
    }
    let root = OccurrencePath::root("f").unwrap();
    assert_eq!(
        root.clone().child(""),
        Err(OccurrencePathDenial::EmptySegment)
    );
    assert_eq!(root.index("", 3), Err(OccurrencePathDenial::EmptySegment));
}

#[test]
fn only_the_canonical_spelling_of_a_key_parses() {
    let invalid_index = |segment: &str| {
        Err(OccurrencePathDenial::InvalidIndex {
            segment: segment.to_owned(),
        })
    };
    for key in ["", "/a", "a/", "a//b", "[1]"] {
        assert_eq!(
            OccurrencePath::parse(key),
            Err(OccurrencePathDenial::EmptySegment),
            "{key:?}"
        );
    }
    // A second spelling of an index would give one path two keys.
    for segment in [
        "i[01]",
        "i[+1]",
        "i[-1]",
        "i[]",
        "i[1",
        "i[1]x",
        "i[1][2]",
        "i[4294967296]",
        "i[ 1]",
        "i[x]",
    ] {
        assert_eq!(
            OccurrencePath::parse(&format!("f/{segment}")),
            invalid_index(segment),
            "{segment:?}"
        );
    }
    assert_eq!(
        OccurrencePath::parse("f/a b"),
        Err(OccurrencePathDenial::InvalidSegment {
            segment: "a b".to_owned()
        })
    );
    assert_eq!(
        OccurrencePath::parse("f/i]"),
        Err(OccurrencePathDenial::InvalidSegment {
            segment: "i]".to_owned()
        })
    );
    assert!(OccurrencePath::parse("f/i[0]").is_ok());
}
