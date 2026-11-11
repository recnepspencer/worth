//! Pure merge wire owners, original finite request, and final backing release.
use super::*;
mod optional_fields;
use crate::history::data::{
    BranchId, CommitId, MergeBaseSelectionRule, RelationalCommitReceipt,
    RelationalMergeBranchBasis, ResolvedMergeBase,
};
use crate::identity::data::{EntityId, PartitionId, VersionId};
use crate::merge::data::*;
use crate::transactions::data::{
    MergeExecutionStructuralSummary, MergeExecutionSummary, PublishedMergeExecutionAuthority,
    RecordRef,
};
use serde::Serialize;
use std::sync::Arc;
use worth_execution::{
    CancellationSource, ExecutionMemoryReservation, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::facade::AspectKey;

#[test]
fn merge_readmission_refuses_zero_memory_before_decode() {
    let budget = SerialMemoryBudget::new(Some(0));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let source = fixture();
    let payload = payload(&source);
    assert!(matches!(
        prepare_readmission(&payload, &request),
        Err(Refusal::Request(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn decoded_identity_name_keeps_only_its_own_original_grant() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let decoded = decode(&fixture(), &request).unwrap();
    let total = budget.charged_memory_bytes();
    let IdentityBasisKind::Custom(custom) = decoded
        .value
        .execution_summary
        .correspondence_witness
        .rows()[0]
        .authority_basis()
    else {
        panic!("fixture carries a custom identity name")
    };
    let name = custom.name.clone();
    let expected = IdentityBasisName::storage_bytes(name.len()).unwrap();
    drop(decoded);
    assert_eq!(budget.charged_memory_bytes(), expected);
    assert!(expected < total);
    assert_eq!(name.as_str(), "identity-λ");
    drop(name);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn decoded_identity_keys_keep_only_their_original_grant() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let decoded = decode(&fixture(), &request).unwrap();
    let link = decoded
        .value
        .execution_summary
        .schema_reconciliation_witness
        .rows()[0]
        .correspondence_linkage()
        .unwrap();
    let IdentityBasisKind::DeclaredKeySet(keys) = &link.basis else {
        panic!("fixture carries declared keys")
    };
    let keys = keys.clone();
    let expected = DeclaredIdentityKeys::storage_bytes(keys.len()).unwrap()
        + keys
            .iter()
            .map(|key| key.owned_allocation_capacity_bytes() as u64)
            .sum::<u64>();
    drop(decoded);
    assert_eq!(budget.charged_memory_bytes(), expected);
    assert_eq!(keys[0].as_str(), "identity-key");
    drop(keys);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn decoded_witness_and_basis_clones_keep_original_backings_funded() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let decoded = decode(&fixture(), &request).unwrap();
    let witness = decoded.value.execution_summary.strategy_witness.clone();
    let base = decoded
        .value
        .execution_summary
        .branch_basis
        .merge_base()
        .clone();
    drop(decoded);
    assert!(budget.charged_memory_bytes() > 0);
    assert_eq!(
        witness.aspect_policy_rows()[0].applied_policies()[0]
            .aspect_key
            .as_str(),
        "counter"
    );
    drop(witness);
    let expected = MergeBasisAncestors::storage_bytes(base.supporting_left_ancestors().len())
        .unwrap()
        + MergeBasisAncestors::storage_bytes(base.supporting_right_ancestors().len()).unwrap();
    assert_eq!(budget.charged_memory_bytes(), expected);
    assert_eq!(
        base.supporting_left_ancestors(),
        &[CommitId(1), CommitId(2)]
    );
    drop(base);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let source = fixture_with_options(false);
    let before = rmp_serde::to_vec_named(&source).unwrap();
    let after = optional_fields::omit(&source);
    let omitted_payload =
        crate::durability::canonical_commit_artifact::borrowed_payload::replace_record(
            &payload(&source),
            &source,
            &after,
        );
    for (body, selected) in [(&before, payload(&source)), (&after, omitted_payload)] {
        let decoded = decode_bytes(body, &selected, &request).unwrap();
        assert_eq!(decoded.value, source);
        drop(decoded);
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
}

struct Decoded {
    value: PublishedMergeExecutionAuthority,
    _inline: Option<ExecutionMemoryReservation>,
}
fn decode(
    source: &PublishedMergeExecutionAuthority,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Decoded, Refusal> {
    let bytes = rmp_serde::to_vec_named(source).unwrap();
    decode_bytes(&bytes, &payload(source), request)
}
fn decode_bytes(
    bytes: &[u8],
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Decoded, Refusal> {
    let quote = super::super::quote(BorrowedPayload(bytes), request)?;
    let inline = request
        .reserve_retained_memory(quote.retained)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let _scratch = request
        .reserve_retained_memory(quote.transient)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut memory = prepare_readmission(payload, request)?.unwrap();
    let value = rmp_serde::from_slice(bytes).map_err(|_| malformed())?;
    memory.attach(&value).map_err(|_| malformed())?;
    Ok(Decoded {
        value,
        _inline: inline,
    })
}

fn payload(source: &PublishedMergeExecutionAuthority) -> Vec<u8> {
    #[derive(Serialize)]
    struct SelectedField<'a> {
        merge_execution_authority: &'a PublishedMergeExecutionAuthority,
    }
    #[derive(Serialize)]
    struct Payload<'a> {
        canonical: SelectedField<'a>,
    }
    // This is only the private preparer's selected pure field grammar, never a
    // fabricated complete Native artifact, receipt, or source admission.
    rmp_serde::to_vec_named(&Payload {
        canonical: SelectedField {
            merge_execution_authority: source,
        },
    })
    .unwrap()
}

fn fixture() -> PublishedMergeExecutionAuthority {
    fixture_with_options(true)
}
fn fixture_with_options(with_link: bool) -> PublishedMergeExecutionAuthority {
    let record = RecordRef::Entity(EntityId::new(PartitionId(1), 2, 1));
    let request = NormalizedRelationalMergeRequest::admit_full_branch(
        BranchId("target".into()),
        BranchId("source".into()),
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeCorrespondencePosture::Advisory,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
    )
    .unwrap();
    let receipt = RelationalCommitReceipt {
        commit_id: CommitId(3),
        version_id: VersionId(3),
        branch_id: BranchId("source".into()),
        parents: vec![CommitId(1)],
    };
    let basis = RelationalMergeBranchBasis {
        source_head: receipt.clone(),
        target_head: receipt.clone(),
        merge_base: ResolvedMergeBase {
            rule: MergeBaseSelectionRule::MaxCommitIdCommonAncestor,
            commit: receipt,
            supporting_left_ancestors: vec![CommitId(1), CommitId(2)].into(),
            supporting_right_ancestors: Vec::new().into(),
        },
    };
    let request_digest = request.request_digest().to_owned();
    let basis_digest = basis.basis_digest();
    let correspondence = RelationalMergeCorrespondenceWitness::retained(
        request_digest.clone(),
        basis_digest.clone(),
        Arc::from([row_for_candidate(
            &IdentityMatchCandidate {
                scope: None,
                source_record: record.clone(),
                target_record: None,
                source: MergeRecordIdentity::StorageRecord(record.clone()),
                target: None,
                match_class: IdentityMatchClass::Exact,
                reason: IdentityResolutionReason::ExactStorageIdentity,
                basis: IdentityBasisKind::Custom(CustomIdentityBasisIdentity {
                    name: "identity-λ".into(),
                    semantic_version: 1,
                }),
            },
            RelationalMergeCorrespondenceWitnessPosture::Admitted,
        )]),
    );
    let schema = RelationalSchemaReconciliationWitness::retained(
        request_digest.clone(),
        basis_digest.clone(),
        Arc::from([RelationalSchemaReconciliationWitnessRow::retained(
            RelationalSchemaReconciliationWitnessRowInput {
                record: record.clone(),
                target_record: None,
                basis: RelationalSchemaReconciliationBasisRow {
                    source_kind_id: None,
                    target_kind_id: None,
                    source_schema_id: None,
                    source_schema_version_id: None,
                    target_schema_id: None,
                    target_schema_version_id: None,
                    registry_digest: "a".repeat(64),
                },
                source_only_aspect_count: 0,
                target_only_aspect_count: 0,
                divergent_aspect_count: 0,
                unavailable_aspect_count: 0,
                decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
                relation_endpoint_divergence: false,
                correspondence_linkage: with_link.then(|| {
                    RelationalSchemaReconciliationCorrespondenceLinkRow {
                        scope: IdentityBasisScope::AspectKey(
                            AspectKey::new("identity-key").unwrap(),
                        ),
                        basis: IdentityBasisKind::DeclaredKeySet(
                            vec![AspectKey::new("identity-key").unwrap()].into(),
                        ),
                        source_record: record.clone(),
                        target_record: record.clone(),
                    }
                }),
            },
        )]),
    );
    let strategy = RelationalMergeStrategyWitness::retained(
        request_digest,
        basis_digest,
        crate::merge::lowered_artifact_execution_authority_contract(),
        Arc::from([RelationalMergeAspectPolicyWitnessRow::retained(
            record.clone(),
            None,
            MergeConflictClass::SourceOnlyAddition,
            if with_link {
                Arc::from([])
            } else {
                Arc::from([AspectPolicyResolutionRecord {
                    aspect_key: AspectKey::new("optional-policy").unwrap(),
                    comparison: AspectComparisonState::Equal,
                    applied_policy: None,
                    decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
                    resolved_value_strategy: None,
                }])
            },
            Arc::from([ResolvedAspectMergePolicy {
                aspect_key: AspectKey::new("counter").unwrap(),
                policy: AspectMergePolicyKind::Custom(CustomMergePolicyIdentity {
                    name: "policy-λ".into(),
                    semantic_version: 1,
                }),
            }]),
            MergePolicyProofBoundary {
                ownership_surface: MergePolicyOwnershipSurface::ContainsCustomPolicy,
                decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
            },
        )]),
        Arc::from([]),
        Arc::from([]),
    );
    let packet = RelationalMergeProofPacket::retained_execution_admitted(
        request.clone(),
        basis.clone(),
        Arc::from([RelationalMergeAdmittedSurfaceRow::new(record, None)]),
        correspondence.witness_digest().into(),
        schema.witness_digest().into(),
        strategy.witness_digest().into(),
        "d".repeat(64),
        "e".repeat(64),
        "f".repeat(64),
    );
    PublishedMergeExecutionAuthority {
        execution_summary: MergeExecutionSummary {
            request,
            branch_basis: basis,
            correspondence_witness: correspondence,
            schema_reconciliation_witness: schema,
            strategy_witness: strategy,
            proof_packet: packet,
            target_head_commit_id: CommitId(3),
            source_head_commit_id: CommitId(3),
            merge_base_commit_id: CommitId(3),
            executed_record_count: 0,
            adopted_source_record_count: 0,
            preserved_shared_record_count: 0,
            reconciled_record_count: 0,
            converged_deleted_on_both_sides_count: 0,
            deleted_on_both_sides_lineage_unchanged_count: 0,
            emitted_mutation_intent_count: 0,
            diagnostics_digest: "e".repeat(64),
            execution_digest: "f".repeat(64),
        },
        structural_summary: MergeExecutionStructuralSummary {
            executed_record_count: 0,
            adopted_source_record_count: 0,
            preserved_shared_record_count: 0,
            reconciled_record_count: 0,
            converged_deleted_on_both_sides_count: 0,
            deleted_on_both_sides_lineage_unchanged_count: 0,
            emitted_mutation_intent_count: 0,
            emitted_entity_create_count: 0,
            emitted_relation_create_count: 0,
            emitted_entity_update_count: 0,
        },
    }
}
