//! Candidate applicability belongs to the request, never to an unrelated rule.
use super::*;
use crate::facade::{
    identity::{KindId, PartitionId},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
};
use crate::transactions::data::{
    CreateIntent, EntitySpec, MergedCommitPlan, MutationIntent, TransactionId, WorkerIntentBatch,
};
use crate::validation::{
    custom_rule::{CustomPreparationBudget, CustomPreparationStop},
    data::*,
    engine::{InvariantObservation, InvariantRequestProfile},
};
use std::{
    num::{NonZeroU64, NonZeroUsize},
    sync::Arc,
};
use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

struct TinyRule(u8);
impl CustomInvariantRule for TinyRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(format!("narrowing.tiny.{}", self.0)),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Untouched tiny rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: NonZeroU64::new(1).unwrap(),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(2)],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![KindId(2)],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn prepare_scope_checked(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(Ok(()))
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
    fn evaluate_checked(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(Ok(CustomInvariantVerdict::Pass))
    }
}
fn runtime() -> RelationalRuntime {
    let mut schema = RelationalSchemaRegistry::new();
    for (id, name) in [(1, "unrelated"), (2, "rule-kind")] {
        schema = schema
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(id),
                kind_name: name.into(),
                schema_id: SchemaId("narrowing".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .unwrap();
    }
    RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![],
        })
        .custom_invariant(CustomInvariantRegistration::new(TinyRule(0)).unwrap())
        .custom_invariant(CustomInvariantRegistration::new(TinyRule(1)).unwrap())
        .build()
}
fn request(serial: bool) -> ExecutionResourceLease<'static> {
    let request = LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 16 * 1024 * 1024, 1_000_000),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    };
    if serial {
        ExecutionResourceLease::serial(request).unwrap()
    } else {
        crate::tests::support::test_execution_authority()
            .request_lease(request)
            .unwrap()
    }
}
fn intent(kind: u32, key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: crate::symbols::data::ClientKey::raw(key),
        fields: Default::default(),
    }))
}
#[test]
fn an_untouched_rule_spends_zero_and_narrowing_is_paid_once_at_both_postures() {
    let mut totals = Vec::new();
    for serial in [false, true] {
        let runtime = runtime();
        let lease = request(serial);
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![intent(1, "one"), intent(1, "two")],
        };
        let candidate = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            None,
        );
        let view = InvariantRuntimeView::from_runtime(&runtime);
        let budget = CustomPreparationBudget::new(lease.status(), Some(1_000_000), Some(1_000_000));
        let selected = eligible_registrations(&view, &candidate, Some(&budget));
        assert_eq!(
            budget.stop(),
            None,
            "unrelated rules cannot refuse preparation"
        );
        assert_eq!(selected.len(), 2);
        // Two catalog visits + two retained registration visits; the candidate
        // has two declared intent visits and no touched-slot journal. One
        // footprint serves both rules, so narrowing is two units, not four.
        assert_eq!(budget.charged_work(), 2 + 2 + 2);
        for selected in selected {
            let InvariantPacketRegistration::CustomNotApplicable { work, .. } = selected else {
                panic!("owner must prove non-applicability")
            };
            assert_eq!(work.charged_work(), 0);
        }
        let evaluation_budget =
            CustomPreparationBudget::new(lease.status(), Some(1_000_000), Some(1_000_000));
        let planned = super::super::execution_plan::plan_invariant_execution_checked(
            &view,
            &candidate,
            &lease,
            &evaluation_budget,
        );
        for packet in planned.packets {
            let InvariantPacketRegistration::CustomNotApplicable { work, .. } =
                &packet.registration
            else {
                panic!("untouched packet")
            };
            crate::validation::execution::evaluate_invariant_packet(&view, &packet);
            assert_eq!(
                work.charged_work(),
                0,
                "reporting a skipped rule cannot spend its allowance"
            );
        }
        let mut tx = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        tx.push_batch(
            WorkerIntentBatch::new("untouched").push(intent(1, "committed")),
            ExecutionRequest::leased(&lease).allocation_policy(),
        )
        .unwrap();
        let (_, report) = ExecutionRequest::leased(&lease)
            .run(ExecutionWorkCeiling::new(1_000_000), |_| {
                tx.commit(&ExecutionRequest::leased(&lease), &runtime)
                    .expect("untouched tiny rules accept the real commit")
            })
            .unwrap();
        totals.push((report.charged_work(), report.charged_span()));
    }
    assert_eq!(totals[0], totals[1]);
}
#[test]
fn a_touched_rule_keeps_its_typed_local_ceiling() {
    for serial in [false, true] {
        let runtime = runtime();
        let lease = request(serial);
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(2),
            merged_intents: vec![intent(2, "affected")],
        };
        let candidate = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            None,
        );
        let budget = CustomPreparationBudget::new(lease.status(), Some(1_000_000), Some(1_000_000));
        eligible_registrations(
            &InvariantRuntimeView::from_runtime(&runtime),
            &candidate,
            Some(&budget),
        );
        assert_eq!(budget.stop(), Some(CustomPreparationStop::WorkExhausted));
        assert!(
            budget.charged_work() < 1_000_000,
            "the request has room; the rule's one-unit bound refused capture"
        );
    }
}

#[test]
fn narrowing_keeps_request_work_and_cancellation_stops() {
    let runtime = runtime();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(3),
        merged_intents: vec![intent(1, "one"), intent(1, "two")],
    };
    let candidate = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        Some(&plan),
        None,
    );
    let lease = request(true);
    let budget = CustomPreparationBudget::new(lease.status(), Some(1), Some(1_000_000));
    assert!(
        super::super::custom_applicability::CandidateTouchedKinds::from_request(
            &candidate,
            Some(&budget)
        )
        .is_none()
    );
    assert_eq!(budget.stop(), Some(CustomPreparationStop::WorkExhausted));
    assert_eq!(budget.charged_work(), 1);
    let cancellation = worth_execution::CancellationSource::new();
    let child = lease.controlled_child(cancellation.token(), None);
    let budget = CustomPreparationBudget::new(child.status(), Some(1_000_000), Some(1_000_000));
    cancellation.cancel();
    assert!(
        super::super::custom_applicability::CandidateTouchedKinds::from_request(
            &candidate,
            Some(&budget)
        )
        .is_none()
    );
    assert_eq!(budget.stop(), Some(CustomPreparationStop::Cancelled));
    assert_eq!(budget.charged_work(), 0);
}
