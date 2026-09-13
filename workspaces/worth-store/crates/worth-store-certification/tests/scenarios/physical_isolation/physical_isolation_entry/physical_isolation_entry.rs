use worth_store_test_support::harness::recovery::closeout as closeout_fixture;

use worth_foundational::{
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceReceiptKind,
    FoundationalBoundaryEvidenceSourceBasisKind,
};
use worth_proof::{RecipeStageDxExt, RecipeStageKind};
use worth_store_physical_certification::{
    admit_physical_isolation_entry, reject_copied_recovery_fields_as_physical_isolation_entry,
    reject_foundational_or_proof_projection_as_physical_isolation_entry,
    reject_json_authority_as_physical_isolation_entry,
    reject_live_runtime_state_as_physical_isolation_entry,
    reject_semantic_snapshot_as_physical_isolation_entry,
    reject_stale_recovery_readiness_as_physical_isolation_entry,
    reject_terminal_projection_as_physical_isolation_entry,
    require_rebound_recovery_readiness_for_physical_isolation_entry,
    PhysicalIsolationEntryCheckedOutcome, PhysicalIsolationEntryDenial,
    PhysicalIsolationEntryRequest,
};

#[test]
fn physical_isolation_entry_admits_a_real_recovery_completion() {
    let completion = closeout_fixture::recovery_completion();
    let entry = admit_physical_isolation_entry(
        PhysicalIsolationEntryRequest::from_recovery_completion(&completion),
    )
    .unwrap();

    assert_eq!(entry.recovered_root(), completion.recovered_root());
    assert_eq!(
        entry.admitted_page_lsn_frontier(),
        completion.admitted_page_lsn_frontier()
    );
    assert_eq!(
        entry.root_epoch_basis(),
        entry.identity().root_epoch_basis()
    );
    assert_eq!(
        entry
            .evidence()
            .foundational()
            .executed_receipt()
            .receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Execution
    );
    assert_eq!(
        entry.evidence().foundational().freshness_posture(),
        FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay
    );
    assert_eq!(
        entry.evidence().foundational().source_basis().kind(),
        FoundationalBoundaryEvidenceSourceBasisKind::BoundaryArtifact
    );
    let progression = entry.evidence().proof_progression();
    assert_eq!(
        progression.unresolved_recipe().stage(),
        RecipeStageKind::Unresolved
    );
    assert_eq!(
        progression.resolved_recipe().stage(),
        RecipeStageKind::Resolved
    );
    assert_eq!(
        progression.lowered_recipe().stage(),
        RecipeStageKind::Lowered
    );
    assert_eq!(
        progression.admitted_recipe().stage(),
        RecipeStageKind::Admitted
    );
}

#[test]
fn repeated_recovery_execution_produces_the_same_entry_identity() {
    let first = closeout_fixture::recovery_completion();
    let second = closeout_fixture::recovery_completion();
    let first = admit_physical_isolation_entry(
        PhysicalIsolationEntryRequest::from_recovery_completion(&first),
    )
    .unwrap();
    let second = admit_physical_isolation_entry(
        PhysicalIsolationEntryRequest::from_recovery_completion(&second),
    )
    .unwrap();

    assert_eq!(first.identity(), second.identity());
    assert_eq!(first.root_epoch_basis(), second.root_epoch_basis());
}

#[test]
fn physical_isolation_entry_rejects_authority_substitutes() {
    assert_eq!(
        reject_copied_recovery_fields_as_physical_isolation_entry(),
        Err(PhysicalIsolationEntryDenial::CopiedRecoveryFields)
    );
    assert_eq!(
        reject_live_runtime_state_as_physical_isolation_entry(),
        Err(PhysicalIsolationEntryDenial::LiveRuntimeState)
    );
    assert_eq!(
        reject_terminal_projection_as_physical_isolation_entry(),
        Err(PhysicalIsolationEntryDenial::TerminalProjection)
    );
    assert_eq!(
        reject_semantic_snapshot_as_physical_isolation_entry(),
        Err(PhysicalIsolationEntryDenial::SemanticSnapshot)
    );
    assert_eq!(
        reject_json_authority_as_physical_isolation_entry(),
        Err(PhysicalIsolationEntryDenial::JsonAuthority)
    );
    assert_eq!(
        reject_foundational_or_proof_projection_as_physical_isolation_entry(),
        Err(PhysicalIsolationEntryDenial::FoundationalOrProofProjection)
    );
    assert_eq!(
        reject_stale_recovery_readiness_as_physical_isolation_entry(),
        PhysicalIsolationEntryCheckedOutcome::Stale(
            PhysicalIsolationEntryDenial::StaleRecoveryReadiness
        )
    );
    assert!(matches!(
        require_rebound_recovery_readiness_for_physical_isolation_entry(),
        PhysicalIsolationEntryCheckedOutcome::RebindRequired(_)
    ));
}
