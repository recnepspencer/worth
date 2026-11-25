use std::collections::BTreeSet;

use super::{OccurrencePath, OccurrencePathDenial};

fn path(key: &str) -> OccurrencePath {
    OccurrencePath::parse(key).expect("a canonical key")
}

fn anchored(name: &str, anchor: &str) -> OccurrencePath {
    path("gen").anchored(name, &path(anchor)).unwrap()
}

#[test]
fn an_anchored_segment_renders_its_anchor_and_parses_back() {
    let placed = anchored("placement", "base/instance[2]/hole");
    assert_eq!(
        placed.to_occurrence_key(),
        "gen/placement(base/instance[2]/hole)"
    );
    assert_eq!(path(&placed.to_occurrence_key()), placed);
    let segment = &placed.segments()[1];
    assert_eq!((segment.name(), segment.index()), ("placement", None));
    assert_eq!(segment.anchor(), Some(&path("base/instance[2]/hole")));
    assert_eq!(path("a/b[1]").segments()[1].anchor(), None);

    // Anchors nest, and a path continues after an anchored segment.
    let nested = path("g")
        .anchored("p", &placed)
        .unwrap()
        .child("hole")
        .unwrap();
    let key = nested.to_occurrence_key();
    assert_eq!(key, "g/p(gen/placement(base/instance[2]/hole))/hole");
    assert_eq!(path(&key), nested);
}

#[test]
fn plain_sorts_before_indexed_before_anchored_and_anchors_order_structurally() {
    let root = path("gen");
    let plain = root.clone().child("p").unwrap();
    let indexed = root.clone().index("p", u32::MAX).unwrap();
    let low = anchored("p", "a/i[9]");
    let high = anchored("p", "a/i[10]");
    assert!(plain < indexed && indexed < low && low < high);
    // The rendered keys would have ordered the anchors the other way.
    assert!(high.to_occurrence_key() < low.to_occurrence_key());
}

/// Every path over plain, indexed and anchored segments, two levels deep.
fn small_paths() -> Vec<OccurrencePath> {
    let anchors = [path("a"), path("a/b[1]"), path("a/b(c)")];
    let mut paths = Vec::new();
    for root in ["a", "b"] {
        let root = path(root);
        let mut ones = vec![
            root.clone().child("p").unwrap(),
            root.clone().index("p", 1).unwrap(),
        ];
        ones.extend(
            anchors
                .iter()
                .map(|a| root.clone().anchored("p", a).unwrap()),
        );
        for one in &ones {
            paths.push(one.clone());
            paths.push(one.clone().child("q").unwrap());
            for anchor in &anchors {
                paths.push(one.clone().anchored("q", anchor).unwrap());
            }
        }
    }
    paths
}

#[test]
fn distinct_anchored_paths_never_share_a_key() {
    let paths = small_paths();
    let keys: BTreeSet<_> = paths
        .iter()
        .map(OccurrencePath::to_occurrence_key)
        .collect();
    assert_eq!(paths.iter().collect::<BTreeSet<_>>().len(), paths.len());
    assert_eq!(keys.len(), paths.len(), "two paths rendered one key");
    for path in &paths {
        assert_eq!(
            OccurrencePath::parse(&path.to_occurrence_key()).as_ref(),
            Ok(path)
        );
    }
}

#[test]
fn parentheses_never_enter_a_segment_name() {
    for segment in ["a(", "a)", "(a)", "a(b)"] {
        assert_eq!(
            OccurrencePath::root(segment),
            Err(OccurrencePathDenial::InvalidSegment {
                segment: segment.to_owned()
            })
        );
        let anchor = path("x");
        assert_eq!(
            path("f").anchored(segment, &anchor),
            Err(OccurrencePathDenial::InvalidSegment {
                segment: segment.to_owned()
            })
        );
    }
    assert_eq!(
        path("f").anchored("", &path("x")),
        Err(OccurrencePathDenial::EmptySegment)
    );
}

#[test]
fn only_one_balanced_non_empty_anchor_closing_its_segment_parses() {
    let invalid_anchor = |segment: &str| {
        Err(OccurrencePathDenial::InvalidAnchor {
            segment: segment.to_owned(),
        })
    };
    for segment in [
        "a()", "a(b", "a(b)x", "a(b)(c)", "a(b))", "a((b)", "a(b)[1]", "a(b(c)",
    ] {
        assert_eq!(
            OccurrencePath::parse(&format!("f/{segment}")),
            invalid_anchor(segment),
            "{segment}"
        );
    }
    // The anchor itself must be canonical.
    assert_eq!(
        OccurrencePath::parse("f/a(b/i[01])"),
        Err(OccurrencePathDenial::InvalidIndex {
            segment: "i[01]".to_owned()
        })
    );
    assert_eq!(
        OccurrencePath::parse("f/a(/b)"),
        Err(OccurrencePathDenial::EmptySegment)
    );
    assert_eq!(
        OccurrencePath::parse("f/a)"),
        Err(OccurrencePathDenial::InvalidSegment {
            segment: "a)".to_owned()
        })
    );
    assert_eq!(
        OccurrencePath::parse("f/(b)"),
        Err(OccurrencePathDenial::EmptySegment)
    );
    assert_eq!(
        OccurrencePath::parse("f/a[1](b)"),
        Err(OccurrencePathDenial::InvalidIndex {
            segment: "a[1](b)".to_owned()
        })
    );
}
