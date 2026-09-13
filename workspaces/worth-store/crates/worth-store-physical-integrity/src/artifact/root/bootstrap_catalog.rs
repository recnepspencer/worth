use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, BootstrapCatalog, BootstrapCatalogDenial, DurableFrameDenial,
    PhysicalRecordFormatDenial,
};

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, input_length, wrong_scope, DurableFrameFieldRange,
};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedBootstrapCatalog, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

const ENVELOPE_FORMAT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const PAYLOAD_LENGTH_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(24, 4);
const GENERATION_COPIES: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 44);
const STORE_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 16);
const PAYLOAD_FORMAT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 10);

#[derive(Debug)]
pub enum BootstrapCatalogIntegrityValidation<'media> {
    Intact(IntegrityValidatedBootstrapCatalog<'media>),
    ScopeMismatch(BootstrapCatalogScopeMismatch),
    UnsupportedFormat(BootstrapCatalogUnsupportedFormat),
    Rejected(PhysicalIntegrityRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapCatalogUnsupportedFormat {
    rejection: PhysicalIntegrityRejection,
    reason: PhysicalRecordFormatDenial,
}

impl BootstrapCatalogUnsupportedFormat {
    pub const fn rejection(self) -> PhysicalIntegrityRejection {
        self.rejection
    }

    pub const fn reason(self) -> PhysicalRecordFormatDenial {
        self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapCatalogScopeMismatch {
    rejection: PhysicalIntegrityRejection,
    observed_store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    expected_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    observed_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
}

impl BootstrapCatalogScopeMismatch {
    pub const fn rejection(self) -> PhysicalIntegrityRejection {
        self.rejection
    }

    pub const fn observed_store(
        self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.observed_store
    }

    pub const fn observed_format(
        self,
    ) -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        self.observed_format
    }

    pub const fn expected_format(
        self,
    ) -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        self.expected_format
    }
}

pub fn validate_bootstrap_catalog<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    BootstrapCatalogIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::BootstrapCatalog {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let catalog = match BootstrapCatalog::decode(artifact.bytes()) {
        Ok(catalog) => catalog,
        Err(BootstrapCatalogDenial::Frame(DurableFrameDenial::UnsupportedFormat(reason))) => {
            return unsupported_format(
                BootstrapCatalogUnsupportedFormat {
                    rejection: from_frame_denial(
                        scope,
                        DurableFrameDenial::UnsupportedFormat(reason),
                    ),
                    reason,
                },
                byte_count,
            );
        }
        Err(denial) => {
            return rejected(catalog_denial(scope, artifact.bytes(), denial), byte_count)
        }
    };
    if catalog.store_identity() != scope.store_identity() {
        return scope_mismatch(
            BootstrapCatalogScopeMismatch {
                rejection: field_damage(
                    scope,
                    PhysicalDamageCause::StoreIdentityMismatch,
                    STORE_IDENTITY_FIELD,
                    PhysicalFormatField::StoreIdentity,
                    PhysicalBlastRadius::CompleteArtifact,
                ),
                observed_store: catalog.store_identity(),
                expected_format: scope.record_format(),
                observed_format: catalog.format(),
            },
            byte_count,
        );
    }
    if catalog.format() != scope.record_format() {
        return scope_mismatch(
            BootstrapCatalogScopeMismatch {
                rejection: field_damage(
                    scope,
                    PhysicalDamageCause::FormatMismatch,
                    ENVELOPE_FORMAT_FIELD,
                    PhysicalFormatField::FormatDeclaration,
                    PhysicalBlastRadius::CompleteArtifact,
                ),
                observed_store: catalog.store_identity(),
                expected_format: scope.record_format(),
                observed_format: catalog.format(),
            },
            byte_count,
        );
    }
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated =
        IntegrityValidatedBootstrapCatalog::new(scope, catalog, byte_range_checksum, artifact)
            .expect("validated bootstrap catalog satisfies the sealed-view contract");
    (
        BootstrapCatalogIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            byte_count,
        ),
    )
}

fn unsupported_format<'media>(
    unsupported: BootstrapCatalogUnsupportedFormat,
    byte_count: u64,
) -> (
    BootstrapCatalogIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            byte_count,
            unsupported.rejection(),
        ),
    )
}

fn scope_mismatch<'media>(
    mismatch: BootstrapCatalogScopeMismatch,
    byte_count: u64,
) -> (
    BootstrapCatalogIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            byte_count,
            mismatch.rejection(),
        ),
    )
}

fn catalog_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: BootstrapCatalogDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        BootstrapCatalogDenial::Frame(denial) => from_frame_denial(scope, denial),
        BootstrapCatalogDenial::PayloadLength => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            PAYLOAD_LENGTH_FIELD,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        BootstrapCatalogDenial::ZeroStoreIdentity => field_damage(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            STORE_IDENTITY_FIELD,
            PhysicalFormatField::StoreIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        BootstrapCatalogDenial::IdentityMismatch => identity_mismatch(scope, bytes),
    }
}

fn identity_mismatch(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let envelope_generation = read_u64(bytes, 28);
    let payload_generation = read_u64(bytes, 64);
    if envelope_generation == 0
        || payload_generation == 0
        || envelope_generation != payload_generation
    {
        return field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            GENERATION_COPIES,
            PhysicalFormatField::RootGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    let expected = scope.record_format().canonical_identity_bytes();
    let envelope_matches = bytes[10..20] == expected;
    let payload_matches = bytes[72..82] == expected;
    let field = match (envelope_matches, payload_matches) {
        (true, false) => Some(PAYLOAD_FORMAT_FIELD),
        (false, true) => Some(ENVELOPE_FORMAT_FIELD),
        _ => None,
    };
    match field {
        Some(field) => field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            field,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        None => damaged(
            scope,
            PhysicalDamageCause::FormatMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::FormatDeclaration),
            PhysicalBlastRadius::CompleteArtifact,
        ),
    }
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("bootstrap framing fixes generation widths"),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    BootstrapCatalogIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        BootstrapCatalogIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            byte_count,
            rejection,
        ),
    )
}
