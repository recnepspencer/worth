//! An external target rule records actual execution without authoring effects.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_relational::facade::runtime::*;

pub(super) struct TargetRule(pub(super) Arc<AtomicUsize>);
impl CustomInvariantRule for TargetRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("epoch-target-execution"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Actual Home target executor".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(100),
                access: CustomInvariantAccessContract::default(),
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
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(CustomInvariantVerdict::Pass)
    }
}
