use super::{
    UiMountedSemanticTextCompletionDenial, UiMountedSemanticTextCompletionInput,
    UiMountedSemanticTextMechanic, UiMountedTextSchemaVersion, UiSemanticTextBaselinePosture,
    UiSemanticTextSlot, UiSemanticTextWrapPosture,
};
use crate::UiMountedInstanceIdentity;

pub(super) fn validate_completion(
    input: &UiMountedSemanticTextCompletionInput,
) -> Result<(), UiMountedSemanticTextCompletionDenial> {
    if input.bounds.posture() != super::super::UiMountedGeometryPosture::Area {
        return Err(UiMountedSemanticTextCompletionDenial::NonAreaGeometry);
    }
    if input.clip_bounds.coordinate_space() != input.bounds.coordinate_space() {
        return Err(UiMountedSemanticTextCompletionDenial::ClipMismatch);
    }
    let max_x = input.bounds.x() + input.bounds.width();
    let max_y = input.bounds.y() + input.bounds.height();
    if !input.origin_x.is_finite()
        || !input.origin_y.is_finite()
        || input.origin_x < input.bounds.x()
        || input.origin_x > max_x
        || input.origin_y < input.bounds.y()
        || input.origin_y > max_y
    {
        return Err(UiMountedSemanticTextCompletionDenial::InvalidTextOrigin);
    }
    if input.node_receipt.frame() != input.frame {
        return Err(UiMountedSemanticTextCompletionDenial::NodeReceiptFrameMismatch);
    }
    if input.node_receipt.mounted_instance() != input.mounted_instance {
        return Err(UiMountedSemanticTextCompletionDenial::NodeReceiptInstanceMismatch);
    }
    if input.text.len() > UiMountedSemanticTextMechanic::MAX_CONTENT_BYTES {
        return Err(UiMountedSemanticTextCompletionDenial::ContentCapacityExceeded);
    }
    let layout = input.layout;
    if layout.source() != input.text.as_ref() {
        return Err(UiMountedSemanticTextCompletionDenial::QualifiedLayoutSourceMismatch);
    }
    if layout.font_collection_generation().get() == 0
        || layout.profile_generation().get() == 0
        || layout.text_scale_generation().get() == 0
    {
        return Err(UiMountedSemanticTextCompletionDenial::QualifiedLayoutGenerationMismatch);
    }
    if matches!(input.slot, UiSemanticTextSlot::CollectionValue { .. })
        != input.collection_row.is_some()
    {
        return Err(UiMountedSemanticTextCompletionDenial::CollectionIdentityMismatch);
    }
    if !matching_foregrounds(input) {
        return Err(UiMountedSemanticTextCompletionDenial::ForegroundSpanMismatch);
    }
    Ok(())
}

pub(super) fn semantic_digest(input: &UiMountedSemanticTextCompletionInput) -> u64 {
    let mut digest = 0x7365_6d61_6e74_6578_u64;
    for value in identity_values(input) {
        digest = fold(digest, value);
    }
    for byte in input.text.bytes() {
        digest = fold(digest, u64::from(byte));
    }
    for byte in input.layout.identity().digest() {
        digest = fold(digest, u64::from(byte));
    }
    if let Some(row) = &input.collection_row {
        for byte in row.0 {
            digest = fold(digest, u64::from(byte));
        }
    }
    for foreground in input.foregrounds.iter() {
        digest = fold(digest, u64::from(foreground.original_range().start()));
        digest = fold(digest, u64::from(foreground.original_range().end()));
        for byte in foreground.identity().digest() {
            digest = fold(digest, u64::from(byte));
        }
        for channel in foreground.color().channels() {
            digest = fold(digest, u64::from(channel));
        }
    }
    for value in profile_values(input) {
        digest = fold(digest, value);
    }
    digest
}

pub(super) fn semantic_digest_mechanic(mechanic: &UiMountedSemanticTextMechanic) -> u64 {
    let mut digest = 0x7365_6d61_6e74_6578_u64;
    for value in [
        u64::from(UiMountedTextSchemaVersion::current().revision()),
        mechanic.content_generation.diagnostic_value(),
        mechanic.frame.diagnostic_value(),
        mechanic.surface.diagnostic_value(),
        mechanic.binding.diagnostic_value(),
        mechanic.mounted_instance.diagnostic_value(),
        mechanic
            .portal_group
            .map_or(0, UiMountedInstanceIdentity::diagnostic_value),
        mechanic.node_receipt.diagnostic_value(),
        mechanic.allocation_basis.receipt_identity(),
        mechanic.allocation_basis.receipt_generation(),
        u64::from(mechanic.origin_x.to_bits()),
        u64::from(mechanic.origin_y.to_bits()),
        u64::from(mechanic.layer_semantic_order),
        mechanic.capability_generation.as_u64(),
        mechanic.capability_profile_digest,
        slot_digest(mechanic.slot),
    ] {
        digest = fold(digest, value);
    }
    for byte in mechanic.text.bytes() {
        digest = fold(digest, u64::from(byte));
    }
    for byte in mechanic.layout_identity.digest() {
        digest = fold(digest, u64::from(byte));
    }
    if let Some(row) = &mechanic.collection_row {
        for byte in row.0 {
            digest = fold(digest, u64::from(byte));
        }
    }
    for foreground in mechanic.foregrounds.iter() {
        digest = fold(digest, u64::from(foreground.original_range().start()));
        digest = fold(digest, u64::from(foreground.original_range().end()));
        for byte in foreground.identity().digest() {
            digest = fold(digest, u64::from(byte));
        }
        for channel in foreground.color().channels() {
            digest = fold(digest, u64::from(channel));
        }
    }
    for value in [
        u64::from(mechanic.profile.size_millipoints()),
        u64::from(mechanic.profile.weight()),
        match mechanic.profile.wrap() {
            UiSemanticTextWrapPosture::Clip => 1,
        },
        match mechanic.profile.baseline() {
            UiSemanticTextBaselinePosture::Alphabetic => 1,
        },
    ] {
        digest = fold(digest, value);
    }
    digest
}

fn matching_foregrounds(input: &UiMountedSemanticTextCompletionInput) -> bool {
    let styles = input.layout.styles();
    if input.text.is_empty() {
        return styles.is_empty() && input.foregrounds.is_empty();
    }
    styles.len() == input.foregrounds.len()
        && styles
            .iter()
            .zip(input.foregrounds.iter())
            .all(|(style, foreground)| style.original_range() == foreground.original_range())
}

fn identity_values(input: &UiMountedSemanticTextCompletionInput) -> [u64; 16] {
    [
        u64::from(UiMountedTextSchemaVersion::current().revision()),
        input.content_generation.diagnostic_value(),
        input.frame.diagnostic_value(),
        input.surface.diagnostic_value(),
        input.binding.diagnostic_value(),
        input.mounted_instance.diagnostic_value(),
        input
            .portal_group
            .map_or(0, UiMountedInstanceIdentity::diagnostic_value),
        input.node_receipt.diagnostic_value(),
        input.allocation_basis.receipt_identity(),
        input.allocation_basis.receipt_generation(),
        u64::from(input.origin_x.to_bits()),
        u64::from(input.origin_y.to_bits()),
        u64::from(input.layer_semantic_order),
        input.capability_generation.as_u64(),
        input.capability_profile_digest,
        slot_digest(input.slot),
    ]
}

fn slot_digest(slot: UiSemanticTextSlot) -> u64 {
    match slot {
        UiSemanticTextSlot::Value => 1,
        UiSemanticTextSlot::CollectionValue {
            selected_field_ordinal,
        } => 2 + u64::from(selected_field_ordinal),
        UiSemanticTextSlot::Posture => u64::from(u16::MAX) + 2,
    }
}

fn profile_values(input: &UiMountedSemanticTextCompletionInput) -> [u64; 4] {
    [
        u64::from(input.profile.size_millipoints()),
        u64::from(input.profile.weight()),
        match input.profile.wrap() {
            UiSemanticTextWrapPosture::Clip => 1,
        },
        match input.profile.baseline() {
            UiSemanticTextBaselinePosture::Alphabetic => 1,
        },
    ]
}

fn fold(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(0x100000001b3)
}
