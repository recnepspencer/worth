#[allow(unused_imports)]
use crate::physical_semantic_isolation::*;
use worth_store_authority::deny_lower_authority_source_readmission_as_current_authority;
use {
    worth_relational::facade::history::BranchId, worth_relational::facade::history::CommitId,
    worth_relational::facade::snapshots::SnapshotId,
    worth_relational::facade::transactions::TransactionId,
};
use {
    worth_store_physical_isolation::correlate_semantic_visibility_with_physical_snapshot,
    worth_store_physical_isolation::deny_semantic_visibility_as_physical_stability,
    worth_store_physical_isolation::physical_epoch_vector_for_current_root,
};

#[test]
fn semantic_visibility_references_cannot_mint_physical_stability() {
    for semantic in semantic_visibility_references() {
        let denial = deny_semantic_visibility_as_physical_stability(&semantic);

        assert!(matches!(
            denial,
            worth_proof::TransitionOutcome::Denied(
                worth_store_physical_isolation::PhysicalSemanticBoundaryDenial::SemanticVisibilityCannotMintPhysicalStability(_)
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
            worth_proof::TransitionOutcome::Denied(
                worth_store_authority::StoreAuthorityReadmissionDenial::LowerAuthoritySourceRequiresOwnerReadmission {
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
    let semantic = worth_store_physical_isolation::SemanticVisibilityReference::relational_snapshot(
        "runtime-a",
        "snapshot-7",
    );
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
        worth_store_physical_isolation::SemanticVisibilityReference::branch(
            "runtime-a",
            "branch/main",
        ),
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
    let observed_drift =
        worth_store_physical_isolation::PhysicalEpochVector::for_scope(expected.scope())
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
        Some(worth_store_physical_isolation::PhysicalEpochDriftKind::RootEpoch)
    );
}

#[test]
fn foundational_roles_preserve_semantic_and_physical_authority_boundaries() {
    let authority = physical_authority_from_complete_closeout();
    let correlation = correlate_semantic_visibility_with_physical_snapshot(
        worth_store_physical_isolation::SemanticVisibilityReference::current_basis(
            "runtime-a",
            "basis/current",
        ),
        authority.correlation_basis(),
    )
    .unwrap();
    let roles = worth_store_physical_isolation::PhysicalSemanticBoundaryRoleEvidence::from_correlation_and_authority(
        &correlation,
        &authority,
    );

    assert_eq!(
        roles.semantic_support().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::SupportOnly
    );
    assert_eq!(
        roles.semantic_projection().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::DerivedProjection
    );
    assert_eq!(
        roles.correlation_receipt().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::ReceiptEvidence
    );
    assert_eq!(
        roles.store_physical_authority().claim().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
    assert_eq!(
        roles.store_physical_authority().claim().category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Artifact
    );
    assert!(roles
        .store_physical_authority()
        .surface()
        .payload()
        .is_store_physical_stability_authority());
}

#[test]
fn relational_exports_are_semantic_diagnostics_not_store_authority() {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeBuilder::new()
        .build()
        .expect("valid native runtime configuration");
    let branch = runtime
        .branch_identity(&BranchId("main".to_owned()))
        .unwrap();
    let (_, basis) = runtime.observe_branch(&branch).unwrap();
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let exports = [
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::transaction(
            10,
            TransactionId(1),
        ),
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::branch(
            10,
            BranchId("main".to_string()),
        ),
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::snapshot(
            10,
            SnapshotId(2),
        ),
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::snapshot_handle(
            &snapshot,
        ),
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::projection(
            10,
            "projection/users",
        ),
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::current_basis(
            10,
            "current-basis",
        ),
        worth_relational::facade::visibility::RelationalStoreCorrelationReference::commit(
            10,
            CommitId(5),
        ),
    ];

    assert!(exports.iter().any(|export| {
        export.kind() == worth_relational::facade::visibility::RelationalStoreCorrelationReferenceKind::Snapshot
            && export.semantic_id() == "2"
    }));
    for export in exports {
        let semantic = semantic_reference_from_relational_export(&export);
        assert_eq!(semantic.semantic_id(), export.semantic_id());
        assert!(!semantic.is_store_physical_stability_authority());
    }
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
