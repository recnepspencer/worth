use worth_foundational::PhysicalArtifactFamily;

use super::{
    OfflineArtifactFamily, OfflineIndeterminatePhysicalReason, OfflineIntegrityReportCompleteness,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflinePhysicalFormatField,
    OfflineUnknownPhysicalReason, OfflineUnsupportedVersionAxis,
};

pub(crate) fn completeness(value: OfflineIntegrityReportCompleteness) -> &'static str {
    match value {
        OfflineIntegrityReportCompleteness::Complete => "complete",
        OfflineIntegrityReportCompleteness::BoundExhausted => "bound_exhausted",
        OfflineIntegrityReportCompleteness::Indeterminate => "indeterminate",
    }
}

pub(crate) fn family(value: OfflineArtifactFamily) -> &'static str {
    match value {
        OfflineArtifactFamily::Unrecognized => "unrecognized",
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::NamespaceIdentity) => {
            "namespace_identity"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::CurrentRootSelector) => {
            "current_root_selector"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::PreviousRootSelector) => {
            "previous_root_selector"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::RootManifest) => "root_manifest",
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::PhysicalWorkObligation) => {
            "physical_work_obligation"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::BootstrapCatalog) => {
            "bootstrap_catalog"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::RootRoutingBlock) => {
            "root_routing_block"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::SegmentMembershipBlock) => {
            "segment_membership_block"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::PageFrame) => "page_frame",
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::ExtentManifest) => {
            "extent_manifest"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::ExtentChunkFrame) => {
            "extent_chunk_frame"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::FreeSpaceHeader) => {
            "free_space_header"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::FreeSpaceMembershipBlock) => {
            "free_space_membership_block"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::WalFrame) => "wal_frame",
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::CheckpointStreamHeader) => {
            "checkpoint_stream_header"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::CheckpointDirtyBasis) => {
            "checkpoint_dirty_basis"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::CheckpointBindingCompaction) => {
            "checkpoint_binding_compaction"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::CheckpointBinding) => {
            "checkpoint_binding"
        }
        OfflineArtifactFamily::Declared(PhysicalArtifactFamily::CheckpointFooter) => {
            "checkpoint_footer"
        }
    }
}

pub(crate) fn damage_cause(value: OfflinePhysicalDamageCause) -> &'static str {
    match value {
        OfflinePhysicalDamageCause::ChecksumMismatch => "checksum_mismatch",
        OfflinePhysicalDamageCause::Framing => "framing",
        OfflinePhysicalDamageCause::ScopeMismatch => "scope_mismatch",
        OfflinePhysicalDamageCause::Pointer => "pointer",
        OfflinePhysicalDamageCause::Truncation => "truncation",
        OfflinePhysicalDamageCause::MissingArtifact => "missing_artifact",
        OfflinePhysicalDamageCause::DuplicateIdentity => "duplicate_identity",
        OfflinePhysicalDamageCause::MalformedPayload => "malformed_payload",
    }
}

pub(crate) fn blast(value: OfflinePhysicalBlastRadius) -> &'static str {
    match value {
        OfflinePhysicalBlastRadius::Field => "field",
        OfflinePhysicalBlastRadius::Frame => "frame",
        OfflinePhysicalBlastRadius::Artifact => "artifact",
        OfflinePhysicalBlastRadius::ReachableRootSubtree => "reachable_root_subtree",
    }
}

pub(crate) fn format_field(value: OfflinePhysicalFormatField) -> &'static str {
    match value {
        OfflinePhysicalFormatField::Magic => "magic",
        OfflinePhysicalFormatField::EncodingVersion => "encoding_version",
        OfflinePhysicalFormatField::NamespaceVersion => "namespace_version",
        OfflinePhysicalFormatField::RecordLength => "record_length",
        OfflinePhysicalFormatField::FieldCount => "field_count",
        OfflinePhysicalFormatField::IdentityField => "identity_field",
        OfflinePhysicalFormatField::FamilyKind => "family_kind",
        OfflinePhysicalFormatField::EnvelopeSchema => "envelope_schema",
        OfflinePhysicalFormatField::FormatVersion => "format_version",
        OfflinePhysicalFormatField::PageSize => "page_size",
        OfflinePhysicalFormatField::ByteOrder => "byte_order",
        OfflinePhysicalFormatField::RootProtocol => "root_protocol",
        OfflinePhysicalFormatField::IntegrityAlgorithm => "integrity_algorithm",
        OfflinePhysicalFormatField::RecordIdentityWidth => "record_identity_width",
        OfflinePhysicalFormatField::HeaderLength => "header_length",
        OfflinePhysicalFormatField::PayloadLength => "payload_length",
        OfflinePhysicalFormatField::FrameIdentity => "frame_identity",
        OfflinePhysicalFormatField::Checksum => "checksum",
        OfflinePhysicalFormatField::StoreIdentity => "store_identity",
        OfflinePhysicalFormatField::SelectorRole => "selector_role",
        OfflinePhysicalFormatField::RootGeneration => "root_generation",
        OfflinePhysicalFormatField::LinkedSelector => "linked_selector",
        OfflinePhysicalFormatField::EmbeddedFormat => "embedded_format",
        OfflinePhysicalFormatField::ManifestGeneration => "manifest_generation",
        OfflinePhysicalFormatField::ManifestPointer => "manifest_pointer",
        OfflinePhysicalFormatField::Reserved => "reserved",
        OfflinePhysicalFormatField::TreeIdentity => "tree_identity",
        OfflinePhysicalFormatField::TreeLevel => "tree_level",
        OfflinePhysicalFormatField::WalLsn => "wal_lsn",
        OfflinePhysicalFormatField::CheckpointAggregate => "checkpoint_aggregate",
    }
}

pub(crate) fn unsupported_axis(value: OfflineUnsupportedVersionAxis) -> &'static str {
    match value {
        OfflineUnsupportedVersionAxis::NamespaceEncoding => "namespace_encoding",
        OfflineUnsupportedVersionAxis::NamespaceSchema => "namespace_schema",
        OfflineUnsupportedVersionAxis::EnvelopeSchema => "envelope_schema",
        OfflineUnsupportedVersionAxis::PhysicalRecordFormat => "physical_record_format",
        OfflineUnsupportedVersionAxis::PageSize => "page_size",
        OfflineUnsupportedVersionAxis::ByteOrder => "byte_order",
        OfflineUnsupportedVersionAxis::RootProtocol => "root_protocol",
        OfflineUnsupportedVersionAxis::IntegrityAlgorithm => "integrity_algorithm",
        OfflineUnsupportedVersionAxis::RecordIdentityWidth => "record_identity_width",
        OfflineUnsupportedVersionAxis::WalFrame => "wal_frame",
        OfflineUnsupportedVersionAxis::CheckpointRecord => "checkpoint_record",
        OfflineUnsupportedVersionAxis::PhysicalWork => "physical_work",
    }
}

pub(crate) fn unknown(value: OfflineUnknownPhysicalReason) -> &'static str {
    match value {
        OfflineUnknownPhysicalReason::UnrecognizedFile => "unrecognized_file",
        OfflineUnknownPhysicalReason::UnrecognizedDirectory => "unrecognized_directory",
        OfflineUnknownPhysicalReason::UnrecognizedOtherEntry => "unrecognized_other_entry",
        OfflineUnknownPhysicalReason::SelectorUnavailable => "selector_unavailable",
        OfflineUnknownPhysicalReason::RootNotAddressed => "root_not_addressed",
        OfflineUnknownPhysicalReason::StoreIdentityUnavailable => "store_identity_unavailable",
        OfflineUnknownPhysicalReason::FilesystemEntryUnavailable => "filesystem_entry_unavailable",
        OfflineUnknownPhysicalReason::ParentScopeUnavailable => "parent_scope_unavailable",
        OfflineUnknownPhysicalReason::WalCoverageUnavailable => "wal_coverage_unavailable",
        OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected => {
            "physical_alias_not_reinspected"
        }
    }
}

pub(crate) fn indeterminate(value: OfflineIndeterminatePhysicalReason) -> &'static str {
    match value {
        OfflineIndeterminatePhysicalReason::SourceChanged => "source_changed",
        OfflineIndeterminatePhysicalReason::EntryBoundExceeded => "entry_bound_exceeded",
        OfflineIndeterminatePhysicalReason::ByteBoundExceeded => "byte_bound_exceeded",
        OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded => "open_file_bound_exceeded",
        OfflineIndeterminatePhysicalReason::DepthBoundExceeded => "depth_bound_exceeded",
        OfflineIndeterminatePhysicalReason::SymlinkRefused => "symlink_refused",
        OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded => "symlink_bound_exceeded",
        OfflineIndeterminatePhysicalReason::ElapsedBoundExceeded => "elapsed_bound_exceeded",
        OfflineIndeterminatePhysicalReason::PhysicalIdentityUnavailable => {
            "physical_identity_unavailable"
        }
        OfflineIndeterminatePhysicalReason::IoFailure => "io_failure",
    }
}
