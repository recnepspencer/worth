use worth_foundational::{
    aspect_mask_digest_preparation_basis, AspectMask, CanonicalDigestMaskMode,
    CanonicalDigestPreparationEntry, CanonicalFieldPath, MutationMask, ProjectionMask,
};

use super::readiness_fixtures::ready_mask;
use crate::certification::foundational_vocabulary::{field, key};

#[test]
fn mask_digest_preparation_basis_canonicalizes_paths_and_preserves_mode() {
    let left = AspectMask::<MutationMask>::new([
        CanonicalFieldPath::single(field("title")),
        CanonicalFieldPath::single(field("done")),
    ]);
    let right = AspectMask::<MutationMask>::new([
        CanonicalFieldPath::single(field("done")),
        CanonicalFieldPath::single(field("title")),
    ]);
    let projection = AspectMask::<ProjectionMask>::new([
        CanonicalFieldPath::single(field("done")),
        CanonicalFieldPath::single(field("title")),
    ]);

    let left_ready = ready_mask(key("task.summary"), left);
    let right_ready = ready_mask(key("task.summary"), right);
    let projection_ready = ready_mask(key("task.summary"), projection);

    assert_eq!(
        aspect_mask_digest_preparation_basis(&left_ready),
        aspect_mask_digest_preparation_basis(&right_ready)
    );
    assert_ne!(
        aspect_mask_digest_preparation_basis(&left_ready),
        aspect_mask_digest_preparation_basis(&projection_ready)
    );
    assert_eq!(
        aspect_mask_digest_preparation_basis(&left_ready),
        &[
            CanonicalDigestPreparationEntry::MaskFieldPath {
                key: key("task.summary"),
                mode: CanonicalDigestMaskMode::Mutation,
                path: CanonicalFieldPath::single(field("done")),
            },
            CanonicalDigestPreparationEntry::MaskFieldPath {
                key: key("task.summary"),
                mode: CanonicalDigestMaskMode::Mutation,
                path: CanonicalFieldPath::single(field("title")),
            },
        ]
    );
}

#[test]
fn whole_aspect_mask_digest_preparation_basis_is_not_confused_with_empty_field_path() {
    let whole = AspectMask::<ProjectionMask>::whole_aspect();
    let ready = ready_mask(key("task.summary"), whole);

    assert_eq!(
        aspect_mask_digest_preparation_basis(&ready),
        &[CanonicalDigestPreparationEntry::MaskWholeAspect {
            key: key("task.summary"),
            mode: CanonicalDigestMaskMode::Projection,
        }]
    );
}
