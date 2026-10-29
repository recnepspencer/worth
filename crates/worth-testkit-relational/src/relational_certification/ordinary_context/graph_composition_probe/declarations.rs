pub use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroupSet};

pub struct LargeAdmissionProbeRegistration {
    pub registration: CustomInvariantRegistration,
    pub preparation_calls: Arc<AtomicUsize>,
    pub evaluation_calls: Arc<AtomicUsize>,
}

pub fn registration(
    id: &'static str,
    point: InvariantExecutionPoint,
    cost_class: InvariantCostClass,
) -> LargeAdmissionProbeRegistration {
    let preparation_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let evaluation_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let registration = worth_relational::facade::runtime::CustomInvariantRegistration::new(LargeAdmissionProbe {
        id,
        point,
        cost_class,
        preparation_calls: std::sync::Arc::clone(&preparation_calls),
        evaluation_calls: std::sync::Arc::clone(&evaluation_calls),
    })
    .expect("Large admission probe registers");
    LargeAdmissionProbeRegistration {
        registration,
        preparation_calls,
        evaluation_calls,
    }
}

#[derive(Clone)]
pub struct LargeAdmissionProbe {
    pub id: &'static str,
    pub point: InvariantExecutionPoint,
    pub cost_class: InvariantCostClass,
    pub preparation_calls: Arc<AtomicUsize>,
    pub evaluation_calls: Arc<AtomicUsize>,
}

impl CustomInvariantRule for LargeAdmissionProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(self.id),
                semantic_version: worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: self.id.into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1_000_000).unwrap()),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: true,
                }
                .canonicalize(),
                execution_point: self.point,
                groups: worth_relational::facade::runtime::InvariantGroupSet::all(),
                cost_class: self.cost_class,
                failure_effect: match self.point {
                    worth_relational::facade::runtime::InvariantExecutionPoint::SnapshotPublication => {
                        worth_relational::facade::runtime::InvariantFailureEffect::BlockPublication
                    }
                    worth_relational::facade::runtime::InvariantExecutionPoint::GraphComposition
                    | worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary => {
                        worth_relational::facade::runtime::InvariantFailureEffect::BlockCommit
                    }
                    worth_relational::facade::runtime::InvariantExecutionPoint::MutationSensitive
                    | worth_relational::facade::runtime::InvariantExecutionPoint::CertificationBoundary
                    | worth_relational::facade::runtime::InvariantExecutionPoint::HarnessAudit => worth_relational::facade::runtime::InvariantFailureEffect::AuditOnly,
                    _ => worth_relational::facade::runtime::InvariantFailureEffect::AuditOnly,
                },
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        self.preparation_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.evaluation_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(worth_relational::facade::runtime::CustomInvariantVerdict::Pass)
    }
}