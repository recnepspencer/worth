//! A cause-free certification rule used by the real runtime registration court.
use super::*;
pub(super) struct CertificationBoundaryViolationRule;

impl CustomInvariantRule for CertificationBoundaryViolationRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("query.test.certification-violation"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Query Test Certification Violation"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1).unwrap()),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract::default(),
                execution_point: InvariantExecutionPoint::CertificationBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::PublicationCoherence),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockPublication,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Violation(
            worth_relational::facade::runtime::CustomInvariantRefusal::Empty,
        ))
    }
}
