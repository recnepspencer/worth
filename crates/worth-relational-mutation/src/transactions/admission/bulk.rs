//! Admission checks for planned bulk mutation batches.

use crate::contracts::symbols::data::client_key_symbol_policy::ClientKeySymbolPolicy;
use crate::contracts::transactions::data::bulk_plan_digest::{bulk_lineage_plan_digest, bulk_naming_plan_digest};
use crate::contracts::transactions::data::outcomes::commit_conflict::{CommitConflict, CommitConflictOwnerPort as _};
use crate::contracts::transactions::data::outcomes::conflict_class::mutation_state_inconsistency::{BulkMutationAdmissionDenial, MutationStateInconsistencyEvidence};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::{PlannedBulkMutationBatch, WorkerIntentBatch};

use crate::transactions::planning::bulk::{
    bulk_mutation_lineage, bulk_mutation_naming, bulk_mutation_provenance,
};

pub fn validate_naming_plan(
    planned: &PlannedBulkMutationBatch,
    client_key_symbol_policy: ClientKeySymbolPolicy,
) -> Result<(), CommitConflict> {
    let mut expected = bulk_mutation_naming(planned.intents.as_ref())
        .normalized_client_keys
        .to_vec();
    let actual = planned.naming.normalized_client_keys.as_ref().to_vec();
    if expected != actual {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation naming plan no longer matches canonicalized intents"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::NamingPlanMismatch {
                        expected_count: expected.len(),
                        actual_count: actual.len(),
                    },
                ),
            },
        ));
    }

    if client_key_symbol_policy.requires_interned_strings()
        && planned
            .naming
            .normalized_client_keys
            .iter()
            .any(|value| value.as_symbol().is_none())
    {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation naming admission requires normalized interned client keys"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::NamingPolicyViolation {
                        client_key_symbol_policy,
                    },
                ),
            },
        ));
    }

    let expected_digest = bulk_naming_plan_digest(&expected);
    if planned.naming.naming_digest != expected_digest {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation naming digest does not match canonical naming set"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::NamingDigestMismatch {
                        expected_digest,
                        actual_digest: planned.naming.naming_digest.clone(),
                    },
                ),
            },
        ));
    }

    expected.clear();
    Ok(())
}

pub fn validate_lineage_plan(planned: &PlannedBulkMutationBatch) -> Result<(), CommitConflict> {
    let expected_transitions = bulk_mutation_lineage(planned.intents.as_ref())
        .transitions
        .to_vec();
    let actual_transitions = planned.lineage.transitions.as_ref().to_vec();
    if expected_transitions != actual_transitions {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation lineage plan no longer matches canonicalized intents"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::LineagePlanMismatch {
                        expected_count: expected_transitions.len(),
                        actual_count: actual_transitions.len(),
                    },
                ),
            },
        ));
    }

    let expected_digest = bulk_lineage_plan_digest(&expected_transitions);
    if planned.lineage.lineage_scope_digest != expected_digest {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation lineage digest does not match canonical lineage transitions"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::LineageDigestMismatch {
                        expected_digest,
                        actual_digest: planned.lineage.lineage_scope_digest.clone(),
                    },
                ),
            },
        ));
    }

    if matches!(
        planned.scope,
        crate::contracts::transactions::data::primitives::BulkMutationScope::TopologyRegionRewrite
    ) && planned.lineage.transitions.is_empty()
    {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "topology rewrite admission requires explicit lineage transitions"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::TopologyRewriteRequiresLineage,
                ),
            },
        ));
    }

    Ok(())
}

pub fn validate_provenance_plan(
    planned: &PlannedBulkMutationBatch,
    batches: &[WorkerIntentBatch],
) -> Result<(), CommitConflict> {
    let expected = bulk_mutation_provenance(
        planned.transaction_id,
        planned.provenance.target_branch.clone(),
        batches,
    );
    if planned.provenance.batch_name != expected.batch_name
        || planned.provenance.worker_batch_names != expected.worker_batch_names
        || planned.provenance.worker_partition_keys != expected.worker_partition_keys
        || planned.provenance.worker_local_only_flags != expected.worker_local_only_flags
    {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation provenance plan no longer matches staged worker evidence"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::ProvenancePlanMismatch {
                        expected_batch_count: expected.worker_batch_names.len(),
                        actual_batch_count: planned.provenance.worker_batch_names.len(),
                    },
                ),
            },
        ));
    }

    if planned.provenance.provenance_digest != expected.provenance_digest {
        return Err(CommitConflict::new(
            ConflictClass::MutationStateInconsistency {
                detail: "bulk mutation provenance digest does not match staged worker evidence"
                    .to_string(),
                evidence: bulk_admission_evidence(
                    planned,
                    BulkMutationAdmissionDenial::ProvenanceDigestMismatch {
                        expected_digest: expected.provenance_digest,
                        actual_digest: planned.provenance.provenance_digest.clone(),
                    },
                ),
            },
        ));
    }

    Ok(())
}

fn bulk_admission_evidence(
    planned: &PlannedBulkMutationBatch,
    denial: BulkMutationAdmissionDenial,
) -> MutationStateInconsistencyEvidence {
    MutationStateInconsistencyEvidence::BulkMutationAdmission {
        transaction_id: planned.transaction_id,
        denial,
    }
}

#[cfg(test)]
pub mod tests {
    use super::{validate_lineage_plan, validate_naming_plan, validate_provenance_plan};
    use crate::contracts::identity::data::{EntityId, KindId, PartitionId};
    use crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
    use crate::contracts::transactions::data::intents::{
        BulkEntityCreateIntent, BulkRelationCreateIntent, CreateIntent, EntityMutationIntent,
        MutationIntent, ReplaceEntityIntent,
    };
    use crate::contracts::transactions::data::outcomes::conflict_class::{
        BulkMutationAdmissionDenial, ConflictClass, MutationStateInconsistencyEvidence,
    };
    use crate::contracts::transactions::data::primitives::{
        EntityReference, EntitySpec, PlannedBulkMutationBatch, TransactionId, WorkerIntentBatch,
    };
    use worth_relational_branch::internal_port::symbols::data::{
        ClientKey, ClientKeySymbolPolicy, StringInterner,
    };

    fn plan(batches: &[WorkerIntentBatch]) -> PlannedBulkMutationBatch {
        crate::transactions::planning::bulk::plan_batch(TransactionId(1), None, batches)
            .expect("the fixture contains a mutation")
    }

    #[test]
    fn naming_admission_rejects_tampered_normalized_key_plan() {
        let policy = ClientKeySymbolPolicy::PreferInterned;
        let mut interner = StringInterner::default();
        let normalized = ClientKey::raw("raw-key")
            .normalize_with(&mut interner, policy)
            .unwrap();
        assert!(normalized.as_symbol().is_some());
        let mut planned = plan(
            &[WorkerIntentBatch::new("bulk").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: vec![normalized],
                    field_patches: vec![AspectFieldPatch::default()],
                }),
            ))],
        );
        validate_naming_plan(&planned, policy).expect("the unchanged key plan is admitted");
        planned.naming.normalized_client_keys =
            std::sync::Arc::<[ClientKey]>::from(vec![ClientKey::raw("raw-key")]);

        let error = validate_naming_plan(&planned, policy)
            .expect_err("naming admission should reject tampered normalized key plan");
        assert!(matches!(
            error.class,
            ConflictClass::MutationStateInconsistency {
                evidence: MutationStateInconsistencyEvidence::BulkMutationAdmission {
                    denial: BulkMutationAdmissionDenial::NamingPlanMismatch { .. },
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn lineage_admission_rejects_tampered_transition_digest() {
        let mut planned = plan(&[
            WorkerIntentBatch::new("rewrite").push(MutationIntent::Entity(
                EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: EntityId::new(PartitionId::main(), 1, 1),
                    replacement: EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw("replacement"),
                        fields: AspectFieldPatch::default(),
                    },
                }),
            )),
        ]);
        validate_lineage_plan(&planned).expect("the unchanged lineage digest is admitted");
        planned.lineage.lineage_scope_digest = "tampered".to_string();

        let error = validate_lineage_plan(&planned).expect_err("lineage admission should reject");
        assert!(matches!(
            error.class,
            ConflictClass::MutationStateInconsistency {
                evidence: MutationStateInconsistencyEvidence::BulkMutationAdmission {
                    denial: BulkMutationAdmissionDenial::LineageDigestMismatch { .. },
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn provenance_admission_rejects_tampered_worker_evidence() {
        let batches = [
            WorkerIntentBatch::new("worker-a").push(MutationIntent::Create(
                CreateIntent::BulkRelations(BulkRelationCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_keys: vec![ClientKey::raw("edge")],
                    endpoints: vec![(
                        EntityReference::Existing(EntityId::new(PartitionId::main(), 1, 1)),
                        EntityReference::Existing(EntityId::new(PartitionId::main(), 2, 1)),
                    )],
                    field_patches: vec![AspectFieldPatch::default()],
                }),
            )),
        ];
        let mut planned = plan(&batches);
        validate_provenance_plan(&planned, &batches)
            .expect("the unchanged worker evidence is admitted");
        planned.provenance.worker_batch_names =
            std::sync::Arc::<[String]>::from(vec!["tampered".to_string()]);

        let error = validate_provenance_plan(&planned, &batches)
            .expect_err("provenance admission should reject");
        assert!(matches!(
            error.class,
            ConflictClass::MutationStateInconsistency {
                evidence: MutationStateInconsistencyEvidence::BulkMutationAdmission {
                    denial: BulkMutationAdmissionDenial::ProvenancePlanMismatch { .. },
                    ..
                },
                ..
            }
        ));
    }
}
