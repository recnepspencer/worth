use std::sync::{Arc, OnceLock};
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView, InternedString};

use super::validation_engine_fixtures::*;
use crate::identity::data::EntityId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::validation::engine::test_view::invariant_view_with_custom_rules;
use worth_harness::facade::execution_fixture::test_execution_lease;

mod summary_image;
use summary_image::{aspect_key, field_key, SummaryImage};

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
                maximum_work_units: Some(std::num::NonZeroU64::new(4096).unwrap()),
                access: crate::validation::data::descriptor::CustomInvariantAccessContract {
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
                execution_point:
                    crate::validation::data::execution::InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::groups::InvariantCostClass::Touched,
                failure_effect:
                    crate::validation::data::execution::InvariantFailureEffect::BlockCommit,
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
            CustomInvariantVerdict::Violation(
                crate::validation::data::custom_rule::refusal::CustomInvariantRefusal::Empty,
            )
        } else {
            CustomInvariantVerdict::Pass
        })
    }
}

#[test]
fn overlapping_field_reads_are_shared_without_sharing_verdicts_or_changed_basis() {
    assert_overlapping_field_reads(false);
}

#[test]
fn leased_opaque_field_read_rules_are_denied_before_execution() {
    assert_overlapping_field_reads(true);
}

fn assert_overlapping_field_reads(leased: bool) {
    let mut image = SummaryImage::new();
    let target = Arc::new(OnceLock::from(image.entity));
    let rules = vec![
        CustomInvariantRegistration::new(ReadsSummary {
            id: "test.summary.accept",
            target: Arc::clone(&target),
            reject_title: None,
            allow_read: true,
        })
        .unwrap(),
        CustomInvariantRegistration::new(ReadsSummary {
            id: "test.summary.reject",
            target,
            reject_title: Some("before"),
            allow_read: true,
        })
        .unwrap(),
    ];
    assert_shared_field_reads(&image, &rules, true, leased);
    image.replace_title("after");
    image.assert_summary("after", "open");
    assert_shared_field_reads(&image, &rules, false, leased);
}

#[test]
fn cached_candidate_metadata_does_not_bypass_sibling_access_denial() {
    let image = SummaryImage::new();
    let target = Arc::new(OnceLock::from(image.entity));
    let rules = vec![
        CustomInvariantRegistration::new(ReadsSummary {
            id: "test.summary.allowed",
            target: Arc::clone(&target),
            reject_title: None,
            allow_read: true,
        })
        .unwrap(),
        CustomInvariantRegistration::new(ReadsSummary {
            id: "test.summary.denied",
            target,
            reject_title: None,
            allow_read: false,
        })
        .unwrap(),
    ];
    let instrumentation = RuntimeInstrumentation::default();
    let view = invariant_view_with_custom_rules(
        &instrumentation,
        image.config.clone(),
        image.version,
        rules,
    );
    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed(image.storage.acquire()),
            image.version,
            None,
            None,
        ),
    );
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::execution::InvariantVerdict::Pass
    ));
    assert!(matches!(
        results.results()[1].verdict,
        crate::validation::data::execution::InvariantVerdict::Violation(_)
    ));
    assert!(format!("{:?}", results.results()[1].verdict).contains("OutsideDeclaredAccess"));
    assert_eq!(
        instrumentation
            .complexity_snapshot()
            .custom_invariant_candidate_entity_aspect_reads,
        1
    );
}

fn assert_shared_field_reads(
    image: &SummaryImage,
    rules: &[CustomInvariantRegistration],
    expect_violation: bool,
    leased: bool,
) {
    let instrumentation = RuntimeInstrumentation::default();
    let view = invariant_view_with_custom_rules(
        &instrumentation,
        image.config.clone(),
        image.version,
        rules.to_vec(),
    );
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed(image.storage.acquire()),
        image.version,
        None,
        None,
    );
    let engine = InvariantEngine::from_view(&view);
    let results = if leased {
        let denial = engine
            .execute(
                &worth_execution::ExecutionRequest::leased(&test_execution_lease()),
                request,
            )
            .expect_err("opaque custom rules cannot execute under a lease");
        assert!(matches!(
            denial,
            crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                denial: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                    kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel
                    ),
                    ..
                },
                ..
            }
        ));
        let counters = instrumentation.complexity_snapshot();
        assert_eq!(counters.custom_invariant_execution_count, 0);
        return;
    } else {
        engine.execute_fixture(request)
    };
    assert_eq!(results.results().len(), 2);
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::execution::InvariantVerdict::Pass
    ));
    assert_eq!(
        matches!(
            results.results()[1].verdict,
            crate::validation::data::execution::InvariantVerdict::Violation(_)
        ),
        expect_violation,
    );
    let counters = instrumentation.complexity_snapshot();
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
        usize::from(leased)
    );
}
