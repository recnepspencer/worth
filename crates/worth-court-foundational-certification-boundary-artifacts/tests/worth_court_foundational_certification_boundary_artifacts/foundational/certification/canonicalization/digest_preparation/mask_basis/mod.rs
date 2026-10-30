use {
    worth_foundational::aspect_mask_digest_preparation_basis, worth_foundational::AspectMask,
    worth_foundational::MutationMask, worth_foundational::ProjectionMask,
};

use {crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::ready_mask};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key};

#[test]
fn mask_digest_preparation_basis_canonicalizes_paths_and_preserves_mode() {
    let left = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("title")),
        worth_foundational::CanonicalFieldPath::single(field("done")),
    ]);
    let right = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("done")),
        worth_foundational::CanonicalFieldPath::single(field("title")),
    ]);
    let projection = AspectMask::<ProjectionMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("done")),
        worth_foundational::CanonicalFieldPath::single(field("title")),
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
            worth_foundational::CanonicalDigestPreparationEntry::MaskFieldPath {
                key: key("task.summary"),
                mode: worth_foundational::CanonicalDigestMaskMode::Mutation,
                path: worth_foundational::CanonicalFieldPath::single(field("done")),
            },
            worth_foundational::CanonicalDigestPreparationEntry::MaskFieldPath {
                key: key("task.summary"),
                mode: worth_foundational::CanonicalDigestMaskMode::Mutation,
                path: worth_foundational::CanonicalFieldPath::single(field("title")),
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
        &[
            worth_foundational::CanonicalDigestPreparationEntry::MaskWholeAspect {
                key: key("task.summary"),
                mode: worth_foundational::CanonicalDigestMaskMode::Projection,
            }
        ]
    );
}
