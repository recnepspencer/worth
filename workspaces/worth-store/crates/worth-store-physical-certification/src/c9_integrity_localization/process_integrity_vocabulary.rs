use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalFormatField};

use super::process_recovery_observation::{
    ProcessDamageCause, ProcessFormatField, ProcessIntegrityArtifactFamily,
};

pub(super) fn project_damage_cause(cause: PhysicalDamageCause) -> ProcessDamageCause {
    use PhysicalDamageCause as Source;
    use ProcessDamageCause as Target;

    match cause {
        Source::WrongMagic => Target::WrongMagic,
        Source::FamilyMismatch => Target::FamilyMismatch,
        Source::FramingLengthMismatch => Target::FramingLengthMismatch,
        Source::ChecksumMismatch => Target::ChecksumMismatch,
        Source::FormatMismatch => Target::FormatMismatch,
        Source::StoreIdentityMismatch => Target::StoreIdentityMismatch,
        Source::ArtifactIdentityMismatch => Target::ArtifactIdentityMismatch,
        Source::PhysicalGenerationMismatch => Target::PhysicalGenerationMismatch,
        Source::SelectorRoleMismatch => Target::SelectorRoleMismatch,
        Source::RecordKindMismatch => Target::RecordKindMismatch,
        Source::ChildReferenceMismatch => Target::ChildReferenceMismatch,
        Source::SequenceMismatch => Target::SequenceMismatch,
        Source::AggregateMismatch => Target::AggregateMismatch,
        Source::MalformedStructure => Target::MalformedStructure,
        Source::Truncated => Target::Truncated,
        Source::MissingArtifact => Target::MissingArtifact,
        Source::DuplicateArtifact => Target::DuplicateArtifact,
    }
}

pub(super) fn project_format_field(field: PhysicalFormatField) -> ProcessFormatField {
    use PhysicalFormatField as Source;
    use ProcessFormatField as Target;

    match field {
        Source::Magic => Target::Magic,
        Source::EnvelopeSchema => Target::EnvelopeSchema,
        Source::FormatVersion => Target::FormatVersion,
        Source::FormatDeclaration => Target::FormatDeclaration,
        Source::EncodedLength => Target::EncodedLength,
        Source::Checksum => Target::Checksum,
        Source::StoreIdentity => Target::StoreIdentity,
        Source::ArtifactFamily => Target::ArtifactFamily,
        Source::ArtifactIdentity => Target::ArtifactIdentity,
        Source::PhysicalGeneration => Target::PhysicalGeneration,
        Source::RuntimeIdentity => Target::RuntimeIdentity,
        Source::OperationIdentity => Target::OperationIdentity,
        Source::OperationFamily => Target::OperationFamily,
        Source::TargetShape => Target::TargetShape,
        Source::PayloadDigestPresence => Target::PayloadDigestPresence,
        Source::SelectorRole => Target::SelectorRole,
        Source::RootGeneration => Target::RootGeneration,
        Source::TreeIdentity => Target::TreeIdentity,
        Source::BlockIdentity => Target::BlockIdentity,
        Source::SegmentIdentity => Target::SegmentIdentity,
        Source::PageIdentity => Target::PageIdentity,
        Source::ExtentIdentity => Target::ExtentIdentity,
        Source::RecordIdentity => Target::RecordIdentity,
        Source::ChunkOrdinal => Target::ChunkOrdinal,
        Source::WalLsnRange => Target::WalLsnRange,
        Source::CheckpointIdentity => Target::CheckpointIdentity,
        Source::CheckpointRecordKind => Target::CheckpointRecordKind,
        Source::RoutingNodeKind => Target::RoutingNodeKind,
        Source::CheckpointAggregate => Target::CheckpointAggregate,
        Source::LinkedSelector => Target::LinkedSelector,
        Source::ChildReference => Target::ChildReference,
        Source::CompleteChildChecksum => Target::CompleteChildChecksum,
        Source::NodeCapacity => Target::NodeCapacity,
        Source::SegmentPageCapacity => Target::SegmentPageCapacity,
        Source::FreeSpaceEntryCount => Target::FreeSpaceEntryCount,
        Source::AllocationFrontier => Target::AllocationFrontier,
        Source::MembershipKind => Target::MembershipKind,
        Source::MembershipCount => Target::MembershipCount,
        Source::MembershipRange => Target::MembershipRange,
        Source::Reserved => Target::Reserved,
        Source::Payload => Target::Payload,
    }
}

pub(super) fn project_artifact_family(
    family: PhysicalIntegrityArtifactFamily,
) -> ProcessIntegrityArtifactFamily {
    use PhysicalIntegrityArtifactFamily as Source;
    use ProcessIntegrityArtifactFamily as Target;

    match family {
        Source::NamespaceIdentity => Target::NamespaceIdentity,
        Source::PhysicalWorkObligation => Target::PhysicalWorkObligation,
        Source::PageFrame => Target::PageFrame,
        Source::ExtentChunk => Target::ExtentChunk,
        Source::WalFrame => Target::WalFrame,
        Source::CheckpointStreamHeader => Target::CheckpointStreamHeader,
        Source::CheckpointDirtyBasis => Target::CheckpointDirtyBasis,
        Source::CheckpointBindingCompaction => Target::CheckpointBindingCompaction,
        Source::CheckpointBinding => Target::CheckpointBinding,
        Source::CheckpointFooter => Target::CheckpointFooter,
        Source::BootstrapCatalog => Target::BootstrapCatalog,
        Source::CurrentRootSelector => Target::CurrentRootSelector,
        Source::PreviousRootSelector => Target::PreviousRootSelector,
        Source::RootManifest => Target::RootManifest,
        Source::RootRoutingBlock => Target::RootRoutingBlock,
        Source::SegmentMembership => Target::SegmentMembership,
        Source::ExtentManifest => Target::ExtentManifest,
        Source::FreeSpaceHeader => Target::FreeSpaceHeader,
        Source::FreeSpaceMembershipBlock => Target::FreeSpaceMembershipBlock,
    }
}
