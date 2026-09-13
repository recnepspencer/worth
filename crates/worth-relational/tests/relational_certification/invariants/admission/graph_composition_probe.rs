use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, CustomInvariantVerdict,
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantGroupSet,
};

pub(super) struct LargeAdmissionProbeRegistration {
    pub(super) registration: CustomInvariantRegistration,
    pub(super) preparation_calls: Arc<AtomicUsize>,
    pub(super) evaluation_calls: Arc<AtomicUsize>,
}

pub(super) fn registration(
    id: &'static str,
    point: InvariantExecutionPoint,
    cost_class: InvariantCostClass,
) -> LargeAdmissionProbeRegistration {
    let preparation_calls = Arc::new(AtomicUsize::new(0));
    let evaluation_calls = Arc::new(AtomicUsize::new(0));
    let registration = CustomInvariantRegistration::new(LargeAdmissionProbe {
        id,
        point,
        cost_class,
        preparation_calls: Arc::clone(&preparation_calls),
        evaluation_calls: Arc::clone(&evaluation_calls),
    })
    .expect("Large admission probe registers");
    LargeAdmissionProbeRegistration {
        registration,
        preparation_calls,
        evaluation_calls,
    }
}

#[derive(Clone)]
struct LargeAdmissionProbe {
    id: &'static str,
    point: InvariantExecutionPoint,
    cost_class: InvariantCostClass,
    preparation_calls: Arc<AtomicUsize>,
    evaluation_calls: Arc<AtomicUsize>,
}

impl CustomInvariantRule for LargeAdmissionProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(self.id),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: self.id.into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1_000_000).unwrap(),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![],
                    affected_relation_kinds: vec![],
                }
                .canonicalize(),
                execution_point: self.point,
                groups: InvariantGroupSet::all(),
                cost_class: self.cost_class,
                failure_effect: match self.point {
                    InvariantExecutionPoint::SnapshotPublication => {
                        InvariantFailureEffect::BlockPublication
                    }
                    InvariantExecutionPoint::GraphComposition
                    | InvariantExecutionPoint::CommitBoundary => {
                        InvariantFailureEffect::BlockCommit
                    }
                    InvariantExecutionPoint::MutationSensitive
                    | InvariantExecutionPoint::CertificationBoundary
                    | InvariantExecutionPoint::HarnessAudit => InvariantFailureEffect::AuditOnly,
                    _ => InvariantFailureEffect::AuditOnly,
                },
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        self.preparation_calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.evaluation_calls.fetch_add(1, Ordering::Relaxed);
        Ok(CustomInvariantVerdict::Pass)
    }
}
