//! Candidate applicability belongs to the request, never to an unrelated rule.
use super::*;
use crate::config::data::RelationalRuntimeConfig;
use crate::identity::data::KindId;
use crate::identity::data::PartitionId;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::EntityKindRegistration;
use crate::schema::data::KindAspectContractDeclarations;
use crate::schema::data::RelationalSchemaRegistry;
use crate::schema::data::SchemaId;
use crate::schema::data::SchemaVersionId;
use crate::transactions::data::intents::mutation_intent::CreateIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::TransactionId;
use crate::validation::engine::InvariantRuntimeView;
use crate::validation::{
    custom_rule::{
        CustomInvariantExecutionContext, CustomInvariantLeaseBudget, CustomInvariantRegistration,
        CustomInvariantRule, CustomInvariantScopePlanner, CustomPreparationBudget,
        CustomPreparationStop,
    },
    data::{
        catalog::InvariantCatalog,
        custom_rule::errors::{
            CustomInvariantExecutionError, CustomInvariantPreparationError, CustomInvariantVerdict,
        },
        descriptor::{
            CustomInvariantAccessContract, CustomInvariantDescriptor,
            CustomInvariantOperationalMetadata,
        },
        execution::{InvariantExecutionPoint, InvariantFailureEffect},
        groups::{InvariantCostClass, InvariantGroup, InvariantGroupSet},
        rule_id::{
            CustomInvariantRuleId, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
        },
    },
    engine::{InvariantObservation, InvariantRequestProfile},
};
use std::collections::BTreeMap;
use std::{
    num::{NonZeroU64, NonZeroUsize},
    sync::Arc,
};
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
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
                maximum_work_units: Some(NonZeroU64::new(1).unwrap()),
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
fn view(instrumentation: &RuntimeInstrumentation) -> InvariantRuntimeView<'_> {
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
    let mut config = RelationalRuntimeConfig::default();
    config.schema.registry = schema;
    config.schema.invariant_catalog = InvariantCatalog {
        registrations: vec![],
    };
    crate::validation::engine::test_view::invariant_view_with_custom_rules(
        instrumentation,
        config,
        VersionId(0),
        vec![
            CustomInvariantRegistration::new(TinyRule(0)).unwrap(),
            CustomInvariantRegistration::new(TinyRule(1)).unwrap(),
        ],
    )
}

fn request(serial: bool) -> ExecutionResourceLease<'static> {
    let request = LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1_000_000)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    };
    if serial {
        ExecutionResourceLease::serial(request).unwrap()
    } else {
        crate::execution::test_authority::test_execution_authority()
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
    for serial in [false, true] {
        let instrumentation = RuntimeInstrumentation::new();
        let view = view(&instrumentation);
        let partitions = BTreeMap::new();
        let lease = request(serial);
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![intent(1, "one"), intent(1, "two")],
        };
        let candidate = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            VersionId(0),
            Some(&plan),
            None,
        );
        let budget = CustomPreparationBudget::new(lease.status(), Some(1_000_000), Some(1_000_000));
        let selected = eligible_registrations(
            &view,
            ExecutionRequest::leased(&lease),
            &candidate,
            Some(&budget),
            super::super::InvariantStage::All,
        );
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
            let InvariantPacketRegistration::CustomNotApplicable {
                work,
                basis:
                    crate::validation::data::non_applicability::CustomInvariantNonApplicability::CandidateOutsideClosure,
                ..
            } = selected
            else {
                panic!("owner must prove the candidate outside the closure")
            };
            assert_eq!(work.charged_work(), 0);
        }
        let evaluation_budget =
            CustomPreparationBudget::new(lease.status(), Some(1_000_000), Some(1_000_000));
        let planned = super::super::execution_plan::plan_invariant_execution_checked(
            &view,
            ExecutionRequest::leased(&lease),
            &candidate,
            &lease,
            &evaluation_budget,
            super::super::InvariantStage::All,
        );
        for packet in planned.packets {
            let InvariantPacketRegistration::CustomNotApplicable { work, .. } =
                &packet.registration
            else {
                panic!("untouched packet")
            };
            crate::validation::execution::evaluate_invariant_packet(
                &view,
                ExecutionRequest::leased(&lease),
                &packet,
            );
            assert_eq!(
                work.charged_work(),
                0,
                "reporting a skipped rule cannot spend its allowance"
            );
        }
    }
}

#[test]
fn a_touched_rule_keeps_its_typed_local_ceiling() {
    for serial in [false, true] {
        let instrumentation = RuntimeInstrumentation::new();
        let view = view(&instrumentation);
        let partitions = BTreeMap::new();
        let lease = request(serial);
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(2),
            merged_intents: vec![intent(2, "affected")],
        };
        let candidate = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            VersionId(0),
            Some(&plan),
            None,
        );
        let budget = CustomPreparationBudget::new(lease.status(), Some(1_000_000), Some(1_000_000));
        eligible_registrations(
            &view,
            ExecutionRequest::leased(&lease),
            &candidate,
            Some(&budget),
            super::super::InvariantStage::All,
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
    let instrumentation = RuntimeInstrumentation::new();
    let view = view(&instrumentation);
    let partitions = BTreeMap::new();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(3),
        merged_intents: vec![intent(1, "one"), intent(1, "two")],
    };
    let candidate = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        VersionId(0),
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
