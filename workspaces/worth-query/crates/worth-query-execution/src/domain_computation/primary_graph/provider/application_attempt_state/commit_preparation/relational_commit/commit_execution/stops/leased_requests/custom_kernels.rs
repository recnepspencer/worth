use super::*;
use std::{num::NonZeroU64, sync::Arc};
use worth_relational::facade::runtime::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
    CustomInvariantExecutionError, CustomInvariantLeaseBudget, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantRuleId, CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
};

#[derive(Clone, Copy)]
enum Probe {
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
                maximum_work_units: NonZeroU64::new(4096).unwrap(),
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
                assert!(!budget.claim_scratch(u64::MAX));
                Some(Ok(()))
            }
            Self::Nested => {
                use worth_execution::{ExecutionScan, MapKernelFailure, ScanOutcome};
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

fn commit_probe(probe: Probe) -> Outcome {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .custom_invariant(CustomInvariantRegistration::new(probe).unwrap())
        .build();
    let lease = authority()
        .request_lease(request(32 * 1024 * 1024, 1_000_000))
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let error = runtime
        .prepare_branch_transaction_with_lease(transaction(&runtime, "custom-target"), &lease)
        .expect_err("custom preparation refused");
    assert!(matches!(error, TransactionCommitError::Execution { .. }));
    let stop = super::super::transaction_commit_stop(error);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    let outcome = application_outcome(stop);
    let Outcome::Denied(denial) = &outcome else {
        panic!("custom preparation refusal folded");
    };
    crate::domain_computation::primary_graph::conditional_operation::assert_preparation_retry(
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
            assert_eq!(
                resource(commit_probe(Probe::Scratch)),
                Resource::ScratchCapacityExceeded
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
