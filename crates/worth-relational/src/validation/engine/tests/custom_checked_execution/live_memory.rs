//! A real checked kernel's workspace is independent of unrelated rule count.
use super::*;

struct WorkspaceRule {
    identity: usize,
    bytes: usize,
    effects: Arc<AtomicUsize>,
}
impl CustomInvariantRule for WorkspaceRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = AlwaysViolatesCustomRule.descriptor();
        descriptor.identity.rule_id =
            CustomInvariantRuleId::new(format!("test.workspace.{}", self.identity));
        descriptor.operational.maximum_work_units = NonZeroU64::new(100_000).unwrap();
        descriptor
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
        panic!("the checked kernel is required")
    }
    fn evaluate_checked(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        if !budget.checkpoint(1) || !budget.claim_scratch(self.bytes as u64) {
            return Some(Ok(CustomInvariantVerdict::Pass));
        }
        if self.bytes != 0 {
            let mut workspace = vec![0u8; self.bytes];
            workspace[0] = 3;
            workspace[self.bytes - 1] = 7;
            assert_eq!(workspace[0] + workspace[self.bytes - 1], 10);
            self.effects.fetch_add(1, Ordering::SeqCst);
        }
        Some(Ok(CustomInvariantVerdict::Pass))
    }
}

fn workspace_run(
    posture: ExecutionPosture,
    count: usize,
    memory: u64,
) -> (
    usize,
    Result<
        crate::validation::engine::InvariantExecutionResult,
        crate::transactions::data::TransactionCommitError,
    >,
) {
    let effects = Arc::new(AtomicUsize::new(0));
    let mut builder =
        RelationalRuntimeApi::builder().schema_registry(RelationalSchemaRegistry::new());
    for identity in 0..count {
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(WorkspaceRule {
                identity,
                bytes: if identity == 0 { 1024 * 1024 } else { 0 },
                effects: effects.clone(),
            })
            .unwrap(),
        );
    }
    let runtime = builder.build();
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), memory, 1_000_000),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request);
    assert_eq!(
        lease.charged_memory_bytes(),
        0,
        "packet and reduction custody releases on every terminal path"
    );
    (effects.load(Ordering::SeqCst), result)
}

#[test]
fn checked_kernel_workspace_uses_shared_lease_instead_of_catalog_quota() {
    for count in [1, 96] {
        let mut verdicts = Vec::new();
        for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
            let (effects, result) = workspace_run(posture, count, 16 * 1024 * 1024);
            assert_eq!(effects, 1, "the actual allocated kernel ran");
            let result =
                result.expect("the funded workspace survives unrelated rule registrations");
            assert_eq!(result.results().len(), count);
            assert!(result.results().iter().all(|result| matches!(
                result.verdict,
                crate::validation::data::InvariantVerdict::Pass
            )));
            verdicts.push(
                result
                    .results()
                    .iter()
                    .map(|result| result.verdict.clone())
                    .collect::<Vec<_>>(),
            );
        }
        assert_eq!(
            verdicts[0], verdicts[1],
            "funded serial and parallel executions agree"
        );
    }
    for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
        let (effects, result) = workspace_run(posture, 1, 512 * 1024);
        assert_eq!(
            effects, 0,
            "actual memory refusal precedes kernel allocation and effects"
        );
        assert!(matches!(result, Err(crate::transactions::data::TransactionCommitError::Execution {
            denial: crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { ancestor: 0, .. }), ..
            }, ..
        })), "the actual refusing lease level survives: {result:?}");
    }
}
