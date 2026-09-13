use worth_foundational::{FoundationalBoundaryArtifactCategory, FoundationalBoundaryArtifactRole};
use worth_proof::TransitionOutcome;
use worth_relational::facade::{
    history::{BranchId, CommitId},
    snapshots::{SnapshotHandle, SnapshotId},
    transactions::TransactionId,
    visibility::{RelationalStoreCorrelationReference, RelationalStoreCorrelationReferenceKind},
};
use worth_store_authority::{
    deny_lower_authority_source_readmission_as_current_authority, StoreAuthorityReadmissionDenial,
    StoreLowerAuthoritySource,
};
use worth_store_physical_isolation::{
    correlate_semantic_visibility_with_physical_snapshot,
    deny_semantic_visibility_as_physical_stability, physical_epoch_vector_for_current_root,
    CurrentPhysicalRoot, PhysicalEpochDriftKind, PhysicalEpochVector, PhysicalOrderingContract,
    PhysicalReadStabilityAuthority, PhysicalSemanticBoundaryDenial,
    PhysicalSemanticBoundaryRoleEvidence, SemanticVisibilityReference,
};

#[test]
fn semantic_visibility_references_cannot_mint_physical_stability() {
    for semantic in semantic_visibility_references() {
        let denial = deny_semantic_visibility_as_physical_stability(&semantic);

        assert!(matches!(
            denial,
            TransitionOutcome::Denied(
                PhysicalSemanticBoundaryDenial::SemanticVisibilityCannotMintPhysicalStability(_)
            )
        ));
        assert!(!semantic.is_store_physical_stability_authority());
    }
}

#[test]
fn store_authority_denies_semantic_lower_authority_sources() {
    for source in semantic_lower_authority_sources() {
        let denial = deny_lower_authority_source_readmission_as_current_authority(source);

        assert_eq!(
            denial,
            TransitionOutcome::Denied(
                StoreAuthorityReadmissionDenial::LowerAuthoritySourceRequiresOwnerReadmission {
                    source
                }
            )
        );
    }
}

#[test]
fn correlation_is_diagnostic_and_equivalence_includes_physical_basis() {
    let authority = physical_authority_from_complete_closeout();
    let same_physical_authority = physical_authority_from_complete_closeout();
    let different_physical_authority = physical_authority_from_operation_digest_closeout("op-21");
    let semantic = SemanticVisibilityReference::relational_snapshot("runtime-a", "snapshot-7");
    let first = correlate_semantic_visibility_with_physical_snapshot(
        semantic.clone(),
        authority.correlation_basis(),
    )
    .unwrap();
    let same_physical = correlate_semantic_visibility_with_physical_snapshot(
        semantic.clone(),
        same_physical_authority.correlation_basis(),
    )
    .unwrap();
    let different_physical = correlate_semantic_visibility_with_physical_snapshot(
        semantic.clone(),
        different_physical_authority.correlation_basis(),
    )
    .unwrap();
    let different_semantic = correlate_semantic_visibility_with_physical_snapshot(
        SemanticVisibilityReference::branch("runtime-a", "branch/main"),
        authority.correlation_basis(),
    )
    .unwrap();

    assert_eq!(first.semantic(), same_physical.semantic());
    assert_eq!(
        first.physical().root_epoch_basis().epoch().get(),
        same_physical.physical().root_epoch_basis().epoch().get()
    );
    assert_ne!(
        authority.root_epoch_basis().epoch().get(),
        different_physical_authority
            .root_epoch_basis()
            .epoch()
            .get()
    );
    assert_ne!(
        first.physical().root_epoch_basis().epoch().get(),
        different_physical
            .physical()
            .root_epoch_basis()
            .epoch()
            .get()
    );
    assert_ne!(first.semantic(), different_semantic.semantic());
    assert!(first.is_diagnostic_only());
    assert!(!first.is_store_physical_stability_authority());
    assert!(authority.is_store_physical_stability_authority());
}

#[test]
fn physical_epoch_basis_consumes_store_physical_authority() {
    let authority = physical_authority_from_complete_closeout();
    let different_authority = physical_authority_from_operation_digest_closeout("op-21");
    let root = current_root_from_authority(&authority);
    let different_root = current_root_from_authority(&different_authority);

    let expected = physical_epoch_vector_for_current_root(root).unwrap();
    let observed_drift = PhysicalEpochVector::for_scope(expected.scope())
        .with_root(different_root.epoch())
        .with_manifest(root.manifest_epoch())
        .seal()
        .unwrap();

    assert_eq!(
        expected
            .compare_against(physical_epoch_vector_for_current_root(root).unwrap())
            .decision(),
        worth_store_physical_isolation::EpochRetryDecision::Current
    );
    assert_eq!(
        expected.compare_against(observed_drift).drift(),
        Some(PhysicalEpochDriftKind::RootEpoch)
    );
}

fn current_root_from_authority(authority: &PhysicalReadStabilityAuthority) -> CurrentPhysicalRoot {
    CurrentPhysicalRoot::from_physical_isolation_entry(
        authority.root_epoch_basis().current_root_basis(),
        PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .unwrap()
}

#[test]
fn foundational_roles_preserve_semantic_and_physical_authority_boundaries() {
    let authority = physical_authority_from_complete_closeout();
    let correlation = correlate_semantic_visibility_with_physical_snapshot(
        SemanticVisibilityReference::current_basis("runtime-a", "basis/current"),
        authority.correlation_basis(),
    )
    .unwrap();
    let roles = PhysicalSemanticBoundaryRoleEvidence::from_correlation_and_authority(
        &correlation,
        &authority,
    );

    assert_eq!(
        roles.semantic_support().role(),
        FoundationalBoundaryArtifactRole::SupportOnly
    );
    assert_eq!(
        roles.semantic_projection().role(),
        FoundationalBoundaryArtifactRole::DerivedProjection
    );
    assert_eq!(
        roles.correlation_receipt().role(),
        FoundationalBoundaryArtifactRole::ReceiptEvidence
    );
    assert_eq!(
        roles.store_physical_authority().claim().role(),
        FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
    assert_eq!(
        roles.store_physical_authority().claim().category(),
        FoundationalBoundaryArtifactCategory::Artifact
    );
    assert!(roles
        .store_physical_authority()
        .surface()
        .payload()
        .is_store_physical_stability_authority());
}

#[test]
fn relational_exports_are_semantic_diagnostics_not_store_authority() {
    let exports = [
        RelationalStoreCorrelationReference::transaction(10, TransactionId(1)),
        RelationalStoreCorrelationReference::branch(10, BranchId("main".to_string())),
        RelationalStoreCorrelationReference::snapshot(10, SnapshotId(2)),
        RelationalStoreCorrelationReference::snapshot_handle(&SnapshotHandle::new(
            3,
            4,
            BranchId("main".to_string()),
        )),
        RelationalStoreCorrelationReference::projection(10, "projection/users"),
        RelationalStoreCorrelationReference::current_basis(10, "current-basis"),
        RelationalStoreCorrelationReference::commit(10, CommitId(5)),
    ];

    assert!(exports.iter().any(|export| {
        export.kind() == RelationalStoreCorrelationReferenceKind::Snapshot
            && export.semantic_id() == "2"
    }));
    for export in exports {
        let semantic = semantic_reference_from_relational_export(&export);
        assert_eq!(semantic.semantic_id(), export.semantic_id());
        assert!(!semantic.is_store_physical_stability_authority());
    }
}

fn semantic_visibility_references() -> Vec<SemanticVisibilityReference> {
    vec![
        SemanticVisibilityReference::transaction("runtime-a", "tx-1"),
        SemanticVisibilityReference::branch("runtime-a", "branch/main"),
        SemanticVisibilityReference::relational_snapshot("runtime-a", "snapshot-1"),
        SemanticVisibilityReference::projection("runtime-a", "projection/a"),
        SemanticVisibilityReference::current_basis("runtime-a", "current"),
        SemanticVisibilityReference::commit("runtime-a", "commit-1"),
    ]
}

fn semantic_lower_authority_sources() -> [StoreLowerAuthoritySource; 6] {
    [
        StoreLowerAuthoritySource::SemanticTransactionVisibility,
        StoreLowerAuthoritySource::SemanticBranchVisibility,
        StoreLowerAuthoritySource::SemanticSnapshotVisibility,
        StoreLowerAuthoritySource::SemanticProjectionVisibility,
        StoreLowerAuthoritySource::SemanticCurrentBasisExport,
        StoreLowerAuthoritySource::SemanticCommitVisibility,
    ]
}

fn physical_authority_from_complete_closeout() -> PhysicalReadStabilityAuthority {
    worth_store_physical_isolation::physical_read_stability_authority_for_certification_test(
        20,
        worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
            .authority_identity(),
    )
}

fn physical_authority_from_operation_digest_closeout(
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

fn semantic_reference_from_relational_export(
    export: &RelationalStoreCorrelationReference,
) -> SemanticVisibilityReference {
    let runtime = export.runtime_instance_id().to_string();
    match export.kind() {
        RelationalStoreCorrelationReferenceKind::Transaction => {
            SemanticVisibilityReference::transaction(runtime, export.semantic_id())
        }
        RelationalStoreCorrelationReferenceKind::Branch => {
            SemanticVisibilityReference::branch(runtime, export.semantic_id())
        }
        RelationalStoreCorrelationReferenceKind::Snapshot => {
            SemanticVisibilityReference::relational_snapshot(runtime, export.semantic_id())
        }
        RelationalStoreCorrelationReferenceKind::Projection => {
            SemanticVisibilityReference::projection(runtime, export.semantic_id())
        }
        RelationalStoreCorrelationReferenceKind::CurrentBasis => {
            SemanticVisibilityReference::current_basis(runtime, export.semantic_id())
        }
        RelationalStoreCorrelationReferenceKind::Commit => {
            SemanticVisibilityReference::commit(runtime, export.semantic_id())
        }
    }
}
