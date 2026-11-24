//! Run the real native intent limit with the configured failure effect.
mod publication;
use super::*;
use crate::validation::data::execution::InvariantExecutionPoint as Point;
use crate::validation::data::execution::InvariantFailureEffect as Effect;
use crate::validation::data::execution::InvariantVerdict;

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
        let catalog = InvariantCatalog {
            registrations: vec![InvariantRegistration::for_rule(
                InvariantRule::MaxMergedIntents(1),
                profile.execution_point(),
                effect,
            )],
        };
        let rule = CustomInvariantRegistration::new(AtPoint {
            probe: ScopeProbe(Arc::clone(&hooks)),
            point: profile.execution_point(),
            checked: true,
        })
        .unwrap();
        let fixture = InvariantDag::new(3, Some(catalog), Some(rule));
        let before = fixture.head();
        let plan = fixture.plan(1);
        assert_eq!(plan.merged_intents.len(), 2);
        let request = InvariantExecutionRequest::from_profile_with_contract(
            profile,
            &fixture.view(),
            fixture.observation(),
            fixture.version(),
            Some(&plan),
            None,
        );
        let lease = paid_dag_request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
        let result = InvariantEngine::from_view(&fixture.view())
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
