use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    DurableRootSelector, PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
    PhysicalRecordFormatVersion, RootSelectorDecodeDenial, RootSelectorRole,
};

use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::validation::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UnsupportedPhysicalIntegrityVersion, UntrustedPhysicalArtifact,
};

use crate::artifact::durable_frame_rejection::{
    field_damage, from_frame_denial, input_length, wrong_scope, DurableFrameFieldRange,
};

const ENVELOPE_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const STORE_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 16);
const SELECTOR_ROLE_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 1);
const ROOT_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(65, 8);
const LINKED_SELECTOR_FIELDS: DurableFrameFieldRange = DurableFrameFieldRange::new(73, 16);
const EMBEDDED_FORMAT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(89, 10);
const SELECTOR_RESERVED_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(99, 8);
const PAYLOAD_LENGTH_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(24, 4);
const ENVELOPE_FORMAT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);

pub(super) fn validate_selector_envelope(
    artifact: UntrustedPhysicalArtifact<'_>,
    scope: PhysicalArtifactScope,
    expected_family: PhysicalIntegrityArtifactFamily,
    expected_role: RootSelectorRole,
) -> Result<DurableRootSelector, PhysicalIntegrityRejection> {
    if scope.artifact_family() != expected_family {
        return Err(wrong_scope(scope));
    }
    if let Some(rejection) = input_length(scope, artifact.byte_count()) {
        return Err(rejection);
    }
    let selector = DurableRootSelector::decode(artifact.bytes())
        .map_err(|denial| selector_denial(scope, artifact.bytes(), denial))?;
    if selector.store_identity() != scope.store_identity() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            STORE_IDENTITY_FIELD,
            PhysicalFormatField::StoreIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if selector.role() != expected_role {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::SelectorRoleMismatch,
            SELECTOR_ROLE_FIELD,
            PhysicalFormatField::SelectorRole,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if selector.format() != scope.record_format() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            ENVELOPE_FORMAT_FIELD,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    Ok(selector)
}

fn selector_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: RootSelectorDecodeDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        RootSelectorDecodeDenial::Frame(denial) => from_frame_denial(scope, denial),
        RootSelectorDecodeDenial::PayloadLength => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            PAYLOAD_LENGTH_FIELD,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        RootSelectorDecodeDenial::ReservedFieldNonZero => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            SELECTOR_RESERVED_FIELD,
            PhysicalFormatField::Reserved,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        RootSelectorDecodeDenial::ZeroStoreIdentity => field_damage(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            STORE_IDENTITY_FIELD,
            PhysicalFormatField::StoreIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        RootSelectorDecodeDenial::ZeroSelectorIdentity => field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            ENVELOPE_IDENTITY_FIELD,
            PhysicalFormatField::ArtifactIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        RootSelectorDecodeDenial::UnknownRole(_) => field_damage(
            scope,
            PhysicalDamageCause::SelectorRoleMismatch,
            SELECTOR_ROLE_FIELD,
            PhysicalFormatField::SelectorRole,
            PhysicalBlastRadius::ReachableSubtree,
        ),
        RootSelectorDecodeDenial::FormatMismatch => embedded_format_denial(scope, bytes),
        RootSelectorDecodeDenial::InvalidLinkage => invalid_linkage(scope, bytes),
    }
}

fn invalid_linkage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let root_generation = u64::from_le_bytes(
        ROOT_GENERATION_FIELD
            .bytes(bytes)
            .try_into()
            .expect("selector framing was validated before linkage"),
    );
    if root_generation == 0 {
        return field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            ROOT_GENERATION_FIELD,
            PhysicalFormatField::RootGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    field_damage(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        LINKED_SELECTOR_FIELDS,
        PhysicalFormatField::LinkedSelector,
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn embedded_format_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    let encoded: [u8; 10] = EMBEDDED_FORMAT_FIELD
        .bytes(bytes)
        .try_into()
        .expect("selector framing fixes the embedded format width");
    let observed_version = u16::from_le_bytes([encoded[0], encoded[1]]);
    if observed_version != PhysicalRecordFormatVersion::V1 as u16 {
        return PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
            scope,
            PhysicalIntegrityVersionAxis::PhysicalFormat,
            u32::from(observed_version),
        ));
    }
    let page_bytes = u32::from_le_bytes(
        encoded[2..6]
            .try_into()
            .expect("physical format page width is fixed"),
    );
    let is_supported_declaration =
        PhysicalPageSizeClass::from_bytes(page_bytes).is_ok_and(|page_size| {
            PhysicalRecordFormatDeclaration::builder()
                .page_size(page_size)
                .admit()
                .expect("canonical page sizes admit")
                .canonical_identity_bytes()
                == encoded
        });
    if is_supported_declaration {
        field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            EMBEDDED_FORMAT_FIELD,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        )
    } else {
        field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            EMBEDDED_FORMAT_FIELD,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        )
    }
}
