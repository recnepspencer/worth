use worth_store_physical_integrity::*;

pub(super) fn family(
    value: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily,
) -> &'static str {
    use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::*;
    match value {
        NamespaceIdentity => "namespace_identity",
        PhysicalWorkObligation => "physical_work_obligation",
        BootstrapCatalog => "bootstrap_catalog",
        CurrentRootSelector => "current_root_selector",
        PreviousRootSelector => "previous_root_selector",
        RootManifest => "root_manifest",
        RootRoutingBlock => "root_routing_block",
        SegmentMembership => "segment_membership_block",
        PageFrame => "page_frame",
        ExtentManifest => "extent_manifest",
        ExtentChunk => "extent_chunk_frame",
        FreeSpaceHeader => "free_space_header",
        FreeSpaceMembershipBlock => "free_space_membership_block",
        WalFrame => "wal_frame",
        CheckpointStreamHeader => "checkpoint_stream_header",
        CheckpointDirtyBasis => "checkpoint_dirty_basis",
        CheckpointBindingCompaction => "checkpoint_binding_compaction",
        CheckpointBinding => "checkpoint_binding",
        CheckpointFooter => "checkpoint_footer",
    }
}

pub(super) fn cause(value: PhysicalDamageCause) -> &'static str {
    use PhysicalDamageCause::*;
    match value {
        WrongMagic => "wrong_magic",
        FamilyMismatch => "family_mismatch",
        FramingLengthMismatch => "framing_length_mismatch",
        ChecksumMismatch => "checksum_mismatch",
        FormatMismatch => "format_mismatch",
        StoreIdentityMismatch => "store_identity_mismatch",
        ArtifactIdentityMismatch => "artifact_identity_mismatch",
        PhysicalGenerationMismatch => "physical_generation_mismatch",
        SelectorRoleMismatch => "selector_role_mismatch",
        RecordKindMismatch => "record_kind_mismatch",
        ChildReferenceMismatch => "child_reference_mismatch",
        SequenceMismatch => "sequence_mismatch",
        AggregateMismatch => "aggregate_mismatch",
        MalformedStructure => "malformed_structure",
        Truncated => "truncated",
        MissingArtifact => "missing_artifact",
        DuplicateArtifact => "duplicate_artifact",
    }
}

pub(super) fn field(value: PhysicalFormatField) -> &'static str {
    use PhysicalFormatField::*;
    match value {
        Magic => "magic",
        EnvelopeSchema => "envelope_schema",
        FormatVersion => "format_version",
        FormatDeclaration => "format_declaration",
        EncodedLength => "encoded_length",
        Checksum => "checksum",
        StoreIdentity => "store_identity",
        ArtifactFamily => "artifact_family",
        ArtifactIdentity => "artifact_identity",
        PhysicalGeneration => "physical_generation",
        RuntimeIdentity => "runtime_identity",
        OperationIdentity => "operation_identity",
        OperationFamily => "operation_family",
        TargetShape => "target_shape",
        PayloadDigestPresence => "payload_digest_presence",
        SelectorRole => "selector_role",
        RootGeneration => "root_generation",
        TreeIdentity => "tree_identity",
        BlockIdentity => "block_identity",
        SegmentIdentity => "segment_identity",
        PageIdentity => "page_identity",
        ExtentIdentity => "extent_identity",
        RecordIdentity => "record_identity",
        ChunkOrdinal => "chunk_ordinal",
        WalLsnRange => "wal_lsn_range",
        CheckpointIdentity => "checkpoint_identity",
        CheckpointRecordKind => "checkpoint_record_kind",
        RoutingNodeKind => "routing_node_kind",
        CheckpointAggregate => "checkpoint_aggregate",
        LinkedSelector => "linked_selector",
        ChildReference => "child_reference",
        CompleteChildChecksum => "complete_child_checksum",
        NodeCapacity => "node_capacity",
        SegmentPageCapacity => "segment_page_capacity",
        FreeSpaceEntryCount => "free_space_entry_count",
        AllocationFrontier => "allocation_frontier",
        MembershipKind => "membership_kind",
        MembershipCount => "membership_count",
        MembershipRange => "membership_range",
        Reserved => "reserved",
        Payload => "payload",
    }
}

pub(super) fn blast(value: PhysicalBlastRadius) -> &'static str {
    use PhysicalBlastRadius::*;
    match value {
        DamagedRange => "field",
        CanonicalFrame => "frame",
        CompleteArtifact => "artifact",
        ReachableSubtree => "reachable_root_subtree",
    }
}

pub(super) fn axis(value: PhysicalIntegrityVersionAxis) -> &'static str {
    use PhysicalIntegrityVersionAxis::*;
    match value {
        EnvelopeSchema => "envelope_schema",
        PhysicalFormat => "physical_format",
        PhysicalWorkObligation => "physical_work_obligation",
        WalFrame => "wal_frame",
        CheckpointRecordSchema => "checkpoint_record_schema",
    }
}

pub(super) fn unknown(value: UnknownPhysicalIntegrityCause) -> &'static str {
    use UnknownPhysicalIntegrityCause::*;
    match value {
        ExpectedArtifactAbsent => "expected_artifact_absent",
        UnrecognizedArtifact => "unrecognized_artifact",
        ExpectedScopeUnavailable => "expected_scope_unavailable",
    }
}

pub(super) fn indeterminate(value: IndeterminatePhysicalIntegrityCause) -> &'static str {
    use IndeterminatePhysicalIntegrityCause::*;
    match value {
        SourceChangedDuringInspection => "source_changed_during_inspection",
        ObservationBoundExhausted => "observation_bound_exhausted",
        StableRangeNotProven => "stable_range_not_proven",
    }
}
