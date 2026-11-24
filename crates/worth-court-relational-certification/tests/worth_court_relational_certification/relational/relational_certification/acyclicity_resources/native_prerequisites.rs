//! Native refusal and execution-admission precedence on real transactions.
use super::fixture::{
    acyclicity_and_connectivity_registry, authority, request, serial_request, DagFixture,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
    CustomInvariantExecutionError, CustomInvariantLeaseBudget, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantRuleId, CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
    InvariantReportedRule, InvariantRule, RelationalRuntimeApi,
};
use worth_relational::facade::transactions::{
    CommitExecutionDenialKind, ConflictClass, DeleteRelationIntent, EntityReference,
    InvariantViolationFields, MutationIntent, RecordRef, RelationMutationIntent,
    RelationalExecutionDenialCause, TransactionCommitError,
};

#[derive(Default)]
struct Hooks {
    prepared: AtomicUsize,
    evaluated: AtomicUsize,
}

impl Hooks {
    fn observed(&self) -> (usize, usize) {
        (
            self.prepared.load(Ordering::SeqCst),
            self.evaluated.load(Ordering::SeqCst),
        )
    }
}

struct ScopeProbe {
    hooks: Arc<Hooks>,
    checked: bool,
}

impl CustomInvariantRule for ScopeProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.native-prerequisite.scope"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Test Custom Violation"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(3)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: Vec::new(),
                    affected_relation_kinds: vec![KindId(2)],
                    include_relation_endpoint_entity_touches: true,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        planner
            .check_live()
            .map_err(|stop| CustomInvariantPreparationError::new(format!("{stop:?}")))?;
        self.hooks.prepared.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn supports_checked_preparation(&self) -> bool {
        self.checked
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(self.prepare_scope(planner))
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        context
            .check_live()
            .map_err(|stop| CustomInvariantExecutionError::new(format!("{stop:?}")))?;
        self.hooks.evaluated.fetch_add(1, Ordering::SeqCst);
        Ok(CustomInvariantVerdict::Pass)
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate(context, scope))
    }
}

#[test]
fn a_native_cycle_refuses_before_custom_preparation_and_its_one_edge_twin_commits() {
    for posture in [
        None,
        Some(ExecutionPosture::Serial),
        Some(ExecutionPosture::Automatic),
    ] {
        let hooks = Arc::new(Hooks::default());
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(acyclicity_and_connectivity_registry())
            .custom_invariant(
                CustomInvariantRegistration::new(ScopeProbe {
                    hooks: Arc::clone(&hooks),
                    checked: true,
                })
                .unwrap(),
            )
            .build()
            .expect("valid native runtime configuration");
        let fixture = DagFixture::from_runtime(runtime, 2);
        let source = fixture.nodes[0];
        let target = fixture.nodes[1];
        let lease = authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture.unwrap_or(ExecutionPosture::Serial),
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(
                        std::num::NonZeroUsize::new(4).unwrap(),
                        Some(8 * 1024 * 1024),
                        Some(4_000_000),
                    ),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let serial = worth_execution::SerialRequest::from_policy(
            lease.policy(),
            CancellationToken::new(),
            None,
        );
        let execution = if posture.is_some() {
            ExecutionRequest::leased(&lease)
        } else {
            ExecutionRequest::serial(&serial)
        };
        let charged_memory = || match posture {
            Some(_) => lease.charged_memory_bytes(),
            None => serial
                .memory()
                .expect("this finite serial request owns its memory budget")
                .charged_memory_bytes(),
        };
        let seeded = fixture
            .publish(
                &execution,
                fixture.edge_batch(target, source, "existing-reverse"),
            )
            .unwrap();
        let reverse = seeded
            .changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Relation(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(reverse.len(), 1);
        fixture.release_snapshot(&seeded);
        let retained_seed_memory = charged_memory();
        let before = fixture.head();
        let baseline = hooks.observed();
        assert_eq!(
            baseline,
            (1, 1),
            "the same installed hook handles the lawful seed"
        );

        let failure = fixture
            .validate(&execution, fixture.edge_batch(source, target, "closing"))
            .expect_err("the single proposed edge closes the committed reverse edge");
        let TransactionCommitError::Conflict { error, .. } = failure else {
            panic!("original native conflict required: {failure:?}");
        };
        let ConflictClass::InvariantViolation {
            fields:
                InvariantViolationFields::Acyclicity {
                    contract_id,
                    relation_kind_id,
                    source: refused_source,
                    target: refused_target,
                },
            ..
        } = error.class
        else {
            panic!("original acyclicity fields required: {error:?}");
        };
        assert_eq!(contract_id.as_str(), "no_cycles");
        assert_eq!(relation_kind_id, KindId(2));
        assert_eq!(refused_source, EntityReference::Existing(source));
        assert_eq!(refused_target, EntityReference::Existing(target));
        assert_eq!(
            hooks.observed(),
            baseline,
            "native denial must protect custom preparation, {posture:?}"
        );
        assert_eq!(fixture.head(), before);
        assert_eq!(charged_memory(), retained_seed_memory);

        let twin = fixture
            .edge_batch(source, target, "closing")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: reverse[0],
                },
            )));
        let committed = fixture
            .publish(&execution, twin)
            .expect("removing one edge makes the same proposal acyclic");
        assert_ne!(fixture.head(), before);
        assert_eq!(hooks.observed(), (baseline.0 + 1, baseline.1 + 1));
        let boundary = committed
            .invariant_executions()
            .iter()
            .filter(|result| {
                result.metadata().execution_point() == InvariantExecutionPoint::ProposedState
            })
            .collect::<Vec<_>>();
        assert_eq!(boundary.len(), 1, "both stages retain one boundary result");
        let result = boundary[0];
        let results = result
            .results()
            .iter()
            .filter(|r| r.execution_point == InvariantExecutionPoint::CommitBoundary)
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert!(matches!(
            &results[0].rule,
            InvariantReportedRule::Native(InvariantRule::AcyclicityContract(_))
        ));
        assert!(
            matches!(&results[1].rule, InvariantReportedRule::Custom(identity)
            if identity.rule_id.as_str() == "test.native-prerequisite.scope")
        );
        assert_eq!(result.decision_log().len(), result.results().len());
        assert_eq!(
            result.metadata().proof_boundary().unwrap().packet_count(),
            result.results().len()
        );
        assert_eq!(
            result
                .metadata()
                .proof_boundary()
                .unwrap()
                .touched_partition_count(),
            1
        );
        fixture.release_snapshot(&committed);
        drop(boundary);
        drop((fixture, seeded, committed));
        assert_eq!(charged_memory(), 0);
    }
}

#[test]
fn unchecked_automatic_admission_retains_precedence_over_a_real_native_cycle() {
    let hooks = Arc::new(Hooks::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(ScopeProbe {
                hooks: Arc::clone(&hooks),
                checked: false,
            })
            .unwrap(),
        )
        .build()
        .expect("valid native runtime configuration");
    let fixture = DagFixture::from_runtime(runtime, 2);
    let original = serial_request();
    let seed = fixture
        .publish(
            &ExecutionRequest::serial(&original),
            fixture.edge_batch(fixture.nodes[1], fixture.nodes[0], "reverse"),
        )
        .unwrap();
    fixture.release_snapshot(&seed);
    assert_eq!(hooks.observed(), (1, 1));
    let before = fixture.head();
    let automatic = authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::new(4).unwrap(),
                    Some(8 * 1024 * 1024),
                    Some(4_000_000),
                ),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let failure = fixture
        .validate(
            &ExecutionRequest::leased(&automatic),
            fixture.edge_batch(fixture.nodes[0], fixture.nodes[1], "closing"),
        )
        .unwrap_err();
    let TransactionCommitError::Execution { denial, .. } = failure else {
        panic!("metadata admission retains its original priority");
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::UncheckedCustomKernel,)
    );
    assert_eq!(denial.partition_identity, None);
    let serial = request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let failure = fixture
        .validate(
            &ExecutionRequest::leased(&serial),
            fixture.edge_batch(fixture.nodes[0], fixture.nodes[1], "closing"),
        )
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = failure else {
        panic!("serial admission reaches native adjudication");
    };
    assert!(matches!(&error.class, ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::Acyclicity { contract_id, .. }, ..
    } if contract_id.as_str() == "no_cycles"));
    assert_eq!(hooks.observed(), (1, 1));
    assert_eq!(fixture.head(), before);
    assert_eq!(automatic.charged_memory_bytes(), 0);
    assert_eq!(serial.charged_memory_bytes(), 0);
}

mod precancellation;
