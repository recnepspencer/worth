//! Run the real native intent limit with the configured failure effect.
mod administration;
mod publication;
use super::*;
use crate::validation::data::{
    InvariantExecutionPoint as Point, InvariantFailureEffect as Effect, InvariantVerdict,
};

struct AtPoint {
    probe: ScopeProbe,
    point: Point,
    checked: bool,
}

impl CustomInvariantRule for AtPoint {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = self.probe.descriptor();
        descriptor.operational.execution_point = self.point;
        if self.point == Point::SnapshotPublication {
            descriptor.operational.groups =
                InvariantGroupSet::of(InvariantGroup::PublicationCoherence);
            descriptor.operational.access.affected_entity_kinds = vec![KindId(3)];
            descriptor
                .operational
                .access
                .affected_relation_kinds
                .clear();
        }
        descriptor
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        self.probe.prepare_scope(planner)
    }
    fn supports_checked_preparation(&self) -> bool {
        self.checked
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        self.probe.prepare_scope_checked(planner, budget)
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.probe.evaluate(context, scope)
    }
    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &(),
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        self.probe.evaluate_checked(context, scope, budget)
    }
}

#[test]
fn native_advisory_continues_custom_preparation_and_harness_audit_collects_even_blocking_evidence()
{
    for (profile, effect, custom_runs) in [
        (
            InvariantRequestProfile::CommitBoundary,
            Effect::BlockCommit,
            false,
        ),
        (
            InvariantRequestProfile::CommitBoundary,
            Effect::AuditOnly,
            true,
        ),
        (
            InvariantRequestProfile::HarnessAudit,
            Effect::BlockCommit,
            true,
        ),
    ] {
        let hooks = Arc::new(Hooks::default());
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(acyclicity_and_connectivity_registry())
            .invariant_catalog(InvariantCatalog {
                registrations: vec![InvariantRegistration::for_rule(
                    InvariantRule::MaxMergedIntents(1),
                    profile.execution_point(),
                    effect,
                )],
            })
            .custom_invariant(
                CustomInvariantRegistration::new(AtPoint {
                    probe: ScopeProbe(Arc::clone(&hooks)),
                    point: profile.execution_point(),
                    checked: true,
                })
                .unwrap(),
            )
            .build();
        // The ordinary one-intent node bootstrap satisfies the same limit.
        let fixture = DagFixture::from_runtime(runtime, 3);
        let before = fixture.head();
        let plan = fixture.merged_plan(fixture.chain_batch(1));
        assert_eq!(plan.merged_intents.len(), 2);
        let request = InvariantExecutionRequest::from_profile_with_contract(
            profile,
            &fixture.runtime,
            InvariantObservation::committed(fixture.runtime.storage_access().current_edition()),
            fixture.runtime.current_version_id(),
            Some(&plan),
            None,
        );
        let lease = super::super::fixture::request(
            8 * 1024 * 1024,
            4_000_000,
            CancellationToken::new(),
            None,
        );
        let result = InvariantEngine::new(&fixture.runtime)
            .execute(&ExecutionRequest::leased(&lease), request)
            .unwrap();
        let limit = result
            .results()
            .iter()
            .find(|check| {
                matches!(
                    check.rule,
                    InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1))
                )
            })
            .unwrap();
        assert_eq!(limit.execution_point, profile.execution_point());
        assert_eq!(limit.failure_effect, effect);
        assert!(matches!(&limit.verdict,
            InvariantVerdict::Violation(violation) | InvariantVerdict::Advisory { violation, .. }
            if matches!(violation.fields, InvariantViolationFields::MergedIntentLimit { merged_intent_count: 2, limit: 1 })
        ));
        assert_eq!(hooks.observed(), if custom_runs { (1, 1) } else { (0, 0) });
        assert_eq!(result.decision_log().len(), result.results().len());
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
