pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::FoundationalBoundaryArtifactCategory,
    worth_foundational::FoundationalBoundaryArtifactRole,
};
pub(crate) use {
    worth_relational::facade::history::BranchId, worth_relational::facade::history::CommitId,
    worth_relational::facade::runtime::RelationalRuntimeBuilder,
    worth_relational::facade::snapshots::SnapshotId,
    worth_relational::facade::transactions::TransactionId,
    worth_relational::facade::visibility::RelationalStoreCorrelationReference,
    worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind,
};
pub(crate) use {
    worth_store_authority::deny_lower_authority_source_readmission_as_current_authority,
    worth_store_authority::StoreAuthorityReadmissionDenial,
    worth_store_authority::StoreLowerAuthoritySource,
};
pub(crate) use {
    worth_store_physical_isolation::correlate_semantic_visibility_with_physical_snapshot,
    worth_store_physical_isolation::deny_semantic_visibility_as_physical_stability,
    worth_store_physical_isolation::physical_epoch_vector_for_current_root,
    worth_store_physical_isolation::CurrentPhysicalRoot,
    worth_store_physical_isolation::PhysicalEpochDriftKind,
    worth_store_physical_isolation::PhysicalEpochVector,
    worth_store_physical_isolation::PhysicalOrderingContract,
    worth_store_physical_isolation::PhysicalReadStabilityAuthority,
    worth_store_physical_isolation::PhysicalSemanticBoundaryDenial,
    worth_store_physical_isolation::PhysicalSemanticBoundaryRoleEvidence,
    worth_store_physical_isolation::SemanticVisibilityReference,
};

pub(crate) fn current_root_from_authority(
    authority: &PhysicalReadStabilityAuthority,
) -> CurrentPhysicalRoot {
    worth_store_physical_isolation::CurrentPhysicalRoot::from_physical_isolation_entry(
        authority.root_epoch_basis().current_root_basis(),
        worth_store_physical_isolation::PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .unwrap()
}

pub(crate) fn semantic_visibility_references() -> Vec<SemanticVisibilityReference> {
    vec![
        worth_store_physical_isolation::SemanticVisibilityReference::transaction(
            "runtime-a",
            "tx-1",
        ),
        worth_store_physical_isolation::SemanticVisibilityReference::branch(
            "runtime-a",
            "branch/main",
        ),
        worth_store_physical_isolation::SemanticVisibilityReference::relational_snapshot(
            "runtime-a",
            "snapshot-1",
        ),
        worth_store_physical_isolation::SemanticVisibilityReference::projection(
            "runtime-a",
            "projection/a",
        ),
        worth_store_physical_isolation::SemanticVisibilityReference::current_basis(
            "runtime-a",
            "current",
        ),
        worth_store_physical_isolation::SemanticVisibilityReference::commit(
            "runtime-a",
            "commit-1",
        ),
    ]
}

pub(crate) fn semantic_lower_authority_sources() -> [StoreLowerAuthoritySource; 6] {
    [
        worth_store_authority::StoreLowerAuthoritySource::SemanticTransactionVisibility,
        worth_store_authority::StoreLowerAuthoritySource::SemanticBranchVisibility,
        worth_store_authority::StoreLowerAuthoritySource::SemanticSnapshotVisibility,
        worth_store_authority::StoreLowerAuthoritySource::SemanticProjectionVisibility,
        worth_store_authority::StoreLowerAuthoritySource::SemanticCurrentBasisExport,
        worth_store_authority::StoreLowerAuthoritySource::SemanticCommitVisibility,
    ]
}

pub(crate) fn physical_authority_from_complete_closeout() -> PhysicalReadStabilityAuthority {
    worth_store_physical_isolation::physical_read_stability_authority_for_certification_test(
        20,
        worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
            .authority_identity(),
    )
}

pub(crate) fn physical_authority_from_operation_digest_closeout(
    operation_digest: &str,
) -> PhysicalReadStabilityAuthority {
    let root_seed = operation_digest
        .bytes()
        .fold(0xcbf29ce484222325_u64, |seed, byte| {
            seed.wrapping_mul(0x100000001b3)
                .wrapping_add(u64::from(byte))
        });
    worth_store_physical_isolation::physical_read_stability_authority_for_certification_test(
        root_seed,
        worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
            .authority_identity(),
    )
}

pub(crate) fn semantic_reference_from_relational_export(
    export: &RelationalStoreCorrelationReference,
) -> SemanticVisibilityReference {
    let runtime = export.runtime_instance_id().to_string();
    match export.kind() {
        worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::Transaction => {
            worth_store_physical_isolation::SemanticVisibilityReference::transaction(runtime, export.semantic_id())
        }
        worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::Branch => {
            worth_store_physical_isolation::SemanticVisibilityReference::branch(runtime, export.semantic_id())
        }
        worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::Snapshot => {
            worth_store_physical_isolation::SemanticVisibilityReference::relational_snapshot(runtime, export.semantic_id())
        }
        worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::Projection => {
            worth_store_physical_isolation::SemanticVisibilityReference::projection(runtime, export.semantic_id())
        }
        worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::CurrentBasis => {
            worth_store_physical_isolation::SemanticVisibilityReference::current_basis(runtime, export.semantic_id())
        }
        worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::Commit => {
            worth_store_physical_isolation::SemanticVisibilityReference::commit(runtime, export.semantic_id())
        }
    }
}
