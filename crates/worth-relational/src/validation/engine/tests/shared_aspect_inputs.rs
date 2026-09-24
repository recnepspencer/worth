use std::sync::{Arc, OnceLock};
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView, InternedString};

use super::aspect_field_uniqueness::{assert_entity_summary, whole_summary_patch};
use super::validation_engine_fixtures::*;
use crate::capabilities::AspectPlanSource;
use crate::identity::data::EntityId;
use crate::tests::support::test_owner_begin_transaction_for_main;
use crate::transactions::data::{ApplyEntityAspectPatchIntent, EntityMutationIntent};

struct ReadsSummary {
    id: &'static str,
    target: Arc<OnceLock<EntityId>>,
    reject_title: Option<&'static str>,
    allow_read: bool,
}

impl CustomInvariantRule for ReadsSummary {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(self.id),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from(self.id),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: if self.allow_read {
                        vec![KindId(1)]
                    } else {
                        vec![]
                    },
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::InvariantCostClass::Touched,
                failure_effect: crate::validation::data::InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        if let Some(target) = self.target.get() {
            let state = planner
                .aspect_states()
                .entity_aspect_state(*target)
                .map_err(|error| {
                    CustomInvariantPreparationError::new(format!(
                        "summary preparation read: {error:?}"
                    ))
                })?;
            if state.get(&aspect_key("summary")).is_none() {
                return Err(CustomInvariantPreparationError::new("summary absent"));
            }
            let committed = planner
                .committed_aspect_states()
                .entity_aspect_state(*target)
                .map_err(|error| {
                    CustomInvariantPreparationError::new(format!(
                        "committed summary read: {error:?}"
                    ))
                })?;
            if committed.get(&aspect_key("summary")).is_none() {
                return Err(CustomInvariantPreparationError::new(
                    "committed summary absent",
                ));
            }
        }
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let rejects = if let Some(target) = self.target.get() {
            let state = context
                .aspect_states()
                .entity_aspect_state(*target)
                .map_err(|error| {
                    CustomInvariantExecutionError::new(format!("summary execution read: {error:?}"))
                })?;
            let summary = state
                .get(&aspect_key("summary"))
                .ok_or_else(|| CustomInvariantExecutionError::new("summary absent"))?;
            let committed = context
                .committed_aspect_states()
                .entity_aspect_state(*target)
                .map_err(|error| {
                    CustomInvariantExecutionError::new(format!("committed summary read: {error:?}"))
                })?;
            if committed.get(&aspect_key("summary")).is_none() {
                return Err(CustomInvariantExecutionError::new(
                    "committed summary absent",
                ));
            }
            let ContractValidatedAspectValueView::Struct(summary) = summary.view() else {
                return Err(CustomInvariantExecutionError::new("summary changed shape"));
            };
            self.reject_title.is_some_and(|title| {
                summary.get(&field_key("title"))
                    == Some(&AspectValue::String(InternedString::Raw(title.to_owned())))
            })
        } else {
            false
        };
        Ok(if rejects {
            CustomInvariantVerdict::Violation
        } else {
            CustomInvariantVerdict::Pass
        })
    }
}

#[test]
fn overlapping_field_reads_are_shared_without_sharing_verdicts_or_changed_basis() {
    assert_overlapping_field_reads(
        crate::facade::runtime::RelationalExecutionModel::SingleLaneExecution,
    );
}

#[test]
fn parallel_overlapping_field_reads_are_materialized_once_per_basis() {
    assert_overlapping_field_reads(
        crate::facade::runtime::RelationalExecutionModel::ParallelPreparation,
    );
}

fn assert_overlapping_field_reads(model: crate::facade::runtime::RelationalExecutionModel) {
    let target = Arc::new(OnceLock::new());
    let schema = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .execution_model(model)
        .custom_invariant(
            CustomInvariantRegistration::new(ReadsSummary {
                id: "test.summary.accept",
                target: Arc::clone(&target),
                reject_title: None,
                allow_read: true,
            })
            .unwrap(),
        )
        .custom_invariant(
            CustomInvariantRegistration::new(ReadsSummary {
                id: "test.summary.reject",
                target: Arc::clone(&target),
                reject_title: Some("before"),
                allow_read: true,
            })
            .unwrap(),
        )
        .build();
    let created = commit_entity_with_summary(&runtime, "subject", "before", "open")
        .expect("subject publication");
    let entity = created
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .expect("created subject");
    target.set(entity).expect("bind subject after creation");
    assert_shared_field_reads(&runtime, true, model);

    let contract = runtime
        .entity_aspect_plan(KindId(1))
        .expect("entity aspect plan")
        .contract_for(&aspect_key("summary"))
        .expect("summary contract")
        .clone();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("change-summary").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: whole_summary_patch(&contract, "after", "open"),
                }),
            )),
        )
        .expect("stage summary change");
    transaction
        .commit(&runtime)
        .expect("summary change publication");
    assert_entity_summary(&runtime, entity, "after", "open");
    assert_shared_field_reads(&runtime, false, model);
}

#[test]
fn cached_candidate_metadata_does_not_bypass_sibling_access_denial() {
    let target = Arc::new(OnceLock::new());
    let schema = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(
            CustomInvariantRegistration::new(ReadsSummary {
                id: "test.summary.allowed",
                target: Arc::clone(&target),
                reject_title: None,
                allow_read: true,
            })
            .unwrap(),
        )
        .custom_invariant(
            CustomInvariantRegistration::new(ReadsSummary {
                id: "test.summary.denied",
                target: Arc::clone(&target),
                reject_title: None,
                allow_read: false,
            })
            .unwrap(),
        )
        .build();
    let created = commit_entity_with_summary(&runtime, "subject", "before", "open").unwrap();
    let entity = created
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    target.set(entity).unwrap();
    runtime.performance_access().reset_counters();

    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
    assert!(matches!(
        results.results()[1].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
    assert!(format!("{:?}", results.results()[1].verdict).contains("OutsideDeclaredAccess"));
    assert_eq!(
        runtime
            .performance_access()
            .counters()
            .custom_invariant_candidate_entity_aspect_reads,
        1
    );
}

fn assert_shared_field_reads(
    runtime: &RelationalRuntime,
    expect_violation: bool,
    model: crate::facade::runtime::RelationalExecutionModel,
) {
    runtime.performance_access().reset_counters();
    let results = InvariantEngine::new(runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );
    assert_eq!(results.results().len(), 2);
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
    assert_eq!(
        matches!(
            results.results()[1].verdict,
            crate::validation::data::InvariantVerdict::Violation(_)
        ),
        expect_violation,
    );
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.custom_invariant_preparation_count, 2);
    assert_eq!(counters.custom_invariant_execution_count, 2);
    assert_eq!(counters.custom_invariant_candidate_entity_aspect_reads, 1);
    assert_eq!(counters.custom_invariant_candidate_relation_aspect_reads, 0);
    assert_eq!(counters.custom_invariant_candidate_entity_reads, 1);
    assert_eq!(
        counters.custom_invariant_candidate_touched_entity_gathers,
        0
    );
    assert_eq!(counters.custom_invariant_candidate_reuse_hits, 17);
    assert_eq!(
        counters.preparation_staged_parallel_strategy_count,
        usize::from(matches!(
            model,
            crate::facade::runtime::RelationalExecutionModel::ParallelPreparation
        ))
    );
}
