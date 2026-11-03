use crate::process::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::stops::leased_requests::*;
use std::num::NonZeroU64;
use std::sync::Arc;
use worth_relational::facade::runtime::CustomInvariantAccessContract;
use worth_relational::facade::runtime::CustomInvariantDescriptor;
use worth_relational::facade::runtime::CustomInvariantExecutionContext;
use worth_relational::facade::runtime::CustomInvariantExecutionError;
use worth_relational::facade::runtime::CustomInvariantLeaseBudget;
use worth_relational::facade::runtime::CustomInvariantOperationalMetadata;
use worth_relational::facade::runtime::CustomInvariantPreparationError;
use worth_relational::facade::runtime::CustomInvariantRegistration;
use worth_relational::facade::runtime::CustomInvariantRule;
use worth_relational::facade::runtime::CustomInvariantRuleId;
use worth_relational::facade::runtime::CustomInvariantScopePlanner;
use worth_relational::facade::runtime::CustomInvariantSemanticIdentity;
use worth_relational::facade::runtime::CustomInvariantSemanticVersion;
use worth_relational::facade::runtime::CustomInvariantVerdict;
use worth_relational::facade::runtime::InvariantCostClass;
use worth_relational::facade::runtime::InvariantExecutionPoint;
use worth_relational::facade::runtime::InvariantFailureEffect;
use worth_relational::facade::runtime::InvariantGroup;
use worth_relational::facade::runtime::InvariantGroupSet;

/// The probe lease's policy memory.
pub const POLICY: u64 = 32 * 1024 * 1024;
/// A scratch claim beyond the probe lease's policy, within the authority's.
pub const SCRATCH_CLAIM: u64 = 48 * 1024 * 1024;

#[derive(Clone, Copy)]
pub enum Probe {
    Unchecked,
    Scratch,
    Nested,
}

impl CustomInvariantRule for Probe {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("provider-stop-probe"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Provider stop probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(NonZeroU64::new(4096).unwrap()),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: Vec::new(),
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: Vec::new(),
                    include_relation_endpoint_entity_touches: false,
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
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn supports_checked_preparation(&self) -> bool {
        !matches!(self, Self::Unchecked)
    }
    fn prepare_scope_checked(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        match self {
            Self::Unchecked => None,
            Self::Scratch => {
                // Claim before allocating; a refusal must survive an owner's
                // successful return, not depend on the owner returning an error.
                assert!(!budget.claim_scratch(SCRATCH_CLAIM));
                Some(Ok(()))
            }
            Self::Nested => {
                use worth_execution::ExecutionScan;
                use worth_execution::MapKernelFailure;
                use worth_execution::ScanOutcome;
                let identity = worth_foundational::PartitionIdentity::new(7);
                let scan =
                    ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
                let outcome = scan.run(None, (), 0, 0, 0, 0, |_, _, _| {
                    Ok::<_, MapKernelFailure<()>>(((), ()))
                });
                assert!(matches!(outcome, ScanOutcome::Stopped { .. }));
                Some(Ok(()))
            }
        }
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

pub fn commit_probe(probe: Probe) -> Outcome {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .custom_invariant(CustomInvariantRegistration::new(probe).unwrap())
        .build()
        .expect("valid native runtime configuration");
    let mut lease_request = request(POLICY, 1_000_000);
    if matches!(probe, Probe::Unchecked) {
        // Unchecked kernels are lawful on serial backing; this refusal needs
        // a request that can actually dispatch more than one worker.
        lease_request.policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), None, Some(1_000_000)),
        );
    }
    let lease = authority().request_lease(lease_request).unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let error = runtime
        .prepare_branch_transaction(
            &worth_execution::ExecutionRequest::leased(&lease),
            transaction(&runtime, "custom-target"),
        )
        .expect_err("custom preparation refused");
    assert!(matches!(error, TransactionCommitError::Execution { .. }));
    let stop = worth_query_execution::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::stops::transaction_commit_stop(error);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    let Stop::PreEffectDenied(failure) = stop else {
        panic!("custom cause must reach pre-effect refusal");
    };
    let outcome = provider_compare_denied(Compare::ProviderSession(failure));
    let Outcome::Denied(denial) = &outcome else {
        panic!("custom preparation refusal folded");
    };
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::conditional_operation::application_operation_reentry::sealed_commit::denial::tests::assert_preparation_retry(
        denial,
    );
    outcome
}

#[test]
fn unchecked_custom_through_public_commit() {
    in_isolated_process(
        concat!(module_path!(), "::unchecked_custom_through_public_commit"),
        || {
            let Outcome::Denied(failure) = commit_probe(Probe::Unchecked) else {
                panic!("wrong unchecked stop")
            };
            assert_eq!(
                failure.kind(),
                Kind::ExecutionUncheckedCustomKernel {
                    partition_identity: None
                }
            );
        },
    );
}

#[test]
fn scratch_through_public_commit() {
    in_isolated_process(
        concat!(module_path!(), "::scratch_through_public_commit"),
        || {
            // The request's own policy refuses the claim, quoting at least it.
            let denial = resource(commit_probe(Probe::Scratch));
            assert!(
                matches!(
                    denial,
                    Resource::MemoryLimit {
                        requested,
                        admitted,
                        level: Level::Policy,
                    } if requested >= SCRATCH_CLAIM && admitted <= POLICY
                ),
                "the request policy refuses the scratch claim: {denial:?}"
            );
        },
    );
}

#[test]
fn nested_stop_through_public_commit() {
    in_isolated_process(
        concat!(module_path!(), "::nested_stop_through_public_commit"),
        || {
            let Outcome::Denied(failure) = commit_probe(Probe::Nested) else {
                panic!("wrong nested stop")
            };
            assert!(matches!(
                failure.kind(),
                Kind::ExecutionNestedPatternStopped { .. }
            ));
        },
    );
}
