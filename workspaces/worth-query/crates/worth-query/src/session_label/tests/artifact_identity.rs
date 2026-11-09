use super::{render_collision_labels, typed_temporal_preview_label};
use crate::facade::runtime::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
    WorthQuerySessionLabel, WorthQuerySessionLabelSegment, WorthQuerySessionNamespace,
};

#[test]
fn artifact_is_exported_through_runtime_facade_and_recomposes_from_typed_parts() {
    let label = typed_temporal_preview_label();
    let recomposed =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SessionLabelIdentity)
            .field_value(
                WorthQueryEvidenceTag::new("session_label_namespace"),
                "worth-kernel",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("session_label_name_segment_count"),
                2,
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("session_label_name_segments"),
                ["temporal", "preview"],
            )
            .seal()
            .expect("fixture digest must fit");

    assert_eq!(
        label.identity_digest().scope(),
        WorthQueryEvidenceScope::SessionLabelIdentity
    );
    assert_eq!(label.display(), "worth-kernel.temporal.preview");
    assert_eq!(label.identity_digest(), &recomposed);
}

#[test]
fn scoped_and_string_construction_paths_share_identity() {
    let typed = typed_temporal_preview_label();
    let strings = WorthQuerySessionLabel::scoped_strs("worth-kernel", ["temporal", "preview"])
        .expect("string label should build");

    assert_eq!(typed, strings);
    assert_eq!(typed.identity_digest(), strings.identity_digest());
    assert_eq!(
        typed.identity_digest().scope(),
        WorthQuerySessionLabel::identity_scope()
    );
}

#[test]
fn display_is_projection_over_typed_parts() {
    let label = typed_temporal_preview_label();

    assert_eq!(label.namespace().as_str(), "worth-kernel");
    assert_eq!(
        label
            .name_segments()
            .iter()
            .map(WorthQuerySessionLabelSegment::as_str)
            .collect::<Vec<_>>(),
        vec!["temporal", "preview"]
    );
    assert_eq!(label.display(), "worth-kernel.temporal.preview");
    assert_eq!(label.to_string(), label.display());
}

#[test]
fn render_collisions_do_not_collapse_identity() {
    let (left, right) = render_collision_labels();

    assert_eq!(left.display(), right.display());
    assert_ne!(left, right);
    assert_ne!(left.identity_digest(), right.identity_digest());
}

#[test]
fn dotted_segment_formatting_accidents_do_not_collapse_into_multiple_segments() {
    let dotted_segment = WorthQuerySessionLabel::scoped(
        WorthQuerySessionNamespace::new("worth-kernel").expect("namespace should build"),
        [WorthQuerySessionLabelSegment::new("temporal.preview").expect("segment should build")],
    )
    .expect("label should build");
    let split_segments =
        WorthQuerySessionLabel::scoped_strs("worth-kernel", ["temporal", "preview"])
            .expect("label should build");

    assert_eq!(dotted_segment.display(), split_segments.display());
    assert_eq!(
        dotted_segment
            .name_segments()
            .iter()
            .map(WorthQuerySessionLabelSegment::as_str)
            .collect::<Vec<_>>(),
        vec!["temporal.preview"]
    );
    assert_eq!(
        split_segments
            .name_segments()
            .iter()
            .map(WorthQuerySessionLabelSegment::as_str)
            .collect::<Vec<_>>(),
        vec!["temporal", "preview"]
    );
    assert_ne!(dotted_segment, split_segments);
    assert_ne!(
        dotted_segment.identity_digest(),
        split_segments.identity_digest()
    );
}

#[test]
fn ordered_segments_change_identity_and_digest() {
    let left = WorthQuerySessionLabel::scoped_strs("worth-kernel", ["preview", "temporal"])
        .expect("label should build");
    let right = WorthQuerySessionLabel::scoped_strs("worth-kernel", ["temporal", "preview"])
        .expect("label should build");

    assert_ne!(left, right);
    assert_ne!(left.identity_digest(), right.identity_digest());
}

#[test]
fn namespace_changes_identity_and_digest_even_with_same_segments() {
    let left = WorthQuerySessionLabel::scoped_strs("worth-kernel", ["temporal", "preview"])
        .expect("label should build");
    let right = WorthQuerySessionLabel::scoped_strs("worth-runtime", ["temporal", "preview"])
        .expect("label should build");

    assert_ne!(left, right);
    assert_ne!(left.identity_digest(), right.identity_digest());
}

#[test]
fn typed_namespace_and_segment_parts_carry_identity_before_rendering() {
    let namespace =
        WorthQuerySessionNamespace::new("worth-kernel").expect("namespace should build");
    let first_segment =
        WorthQuerySessionLabelSegment::new("temporal").expect("segment should build");
    let second_segment =
        WorthQuerySessionLabelSegment::new("preview").expect("segment should build");
    let label = WorthQuerySessionLabel::scoped(
        namespace.clone(),
        [first_segment.clone(), second_segment.clone()],
    )
    .expect("label should build");

    assert_eq!(label.namespace(), &namespace);
    assert_eq!(label.name_segments(), &[first_segment, second_segment]);
}
