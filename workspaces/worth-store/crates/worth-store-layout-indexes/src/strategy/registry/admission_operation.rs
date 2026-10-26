//! Strategy registry admission operation and owner-issued outcome.

use super::{
    compatibility::{
        family_lane_supports_operation, maintenance_mode_supports_lane,
        mutation_shape_is_compatible, require_key_law_compatibility,
        require_materialization_compatibility, require_requested_key_law_compatibility,
    },
    LayoutAdmissionDenial, LayoutAdmissionRequest, LayoutStrategyCapability,
};
use crate::keyspace::{CompositeKeyOrderingLaw, HashCollisionLaw};
use crate::strategy::{admit_strategy_from_basis, AdmittedLayoutStrategy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutStrategyRegistrySnapshot {
    inner: std::sync::Arc<LayoutStrategyRegistrySnapshotData>,
}

#[derive(Debug, PartialEq, Eq)]
struct LayoutStrategyRegistrySnapshotData {
    admitted_strategy: AdmittedLayoutStrategy,
    request: LayoutAdmissionRequest,
    granted_capability: LayoutStrategyCapability,
    hash_equality_law: Option<HashCollisionLaw>,
    composite_ordering_law: Option<CompositeKeyOrderingLaw>,
}

impl LayoutStrategyRegistrySnapshot {
    fn issue(
        admitted_strategy: AdmittedLayoutStrategy,
        request: LayoutAdmissionRequest,
        granted_capability: LayoutStrategyCapability,
        hash_equality_law: Option<HashCollisionLaw>,
        composite_ordering_law: Option<CompositeKeyOrderingLaw>,
    ) -> Self {
        Self {
            inner: std::sync::Arc::new(LayoutStrategyRegistrySnapshotData {
                admitted_strategy,
                request,
                granted_capability,
                hash_equality_law,
                composite_ordering_law,
            }),
        }
    }

    pub fn admitted_strategy(&self) -> &AdmittedLayoutStrategy {
        &self.inner.admitted_strategy
    }

    pub fn request(&self) -> &LayoutAdmissionRequest {
        &self.inner.request
    }

    pub fn granted_capability(&self) -> LayoutStrategyCapability {
        self.inner.granted_capability
    }

    pub fn hash_equality_law(&self) -> Option<HashCollisionLaw> {
        self.inner.hash_equality_law
    }

    pub fn composite_ordering_law(&self) -> Option<CompositeKeyOrderingLaw> {
        self.inner.composite_ordering_law
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LayoutAdmissionPayload {
    Success(LayoutStrategyRegistrySnapshot),
    Denied(Box<LayoutAdmissionDenial>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutAdmissionOutcome {
    case: LayoutAdmissionPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayoutAdmissionCaseId {
    Admitted,
    Denied(super::LayoutAdmissionDenialCase),
}

impl LayoutAdmissionCaseId {
    pub const fn as_str(self) -> &'static str {
        use super::LayoutAdmissionDenialCase as Denial;
        match self {
            Self::Admitted => "layout.strategy.admission.admitted",
            Self::Denied(Denial::StrategyVocabularyDenied) => {
                "layout.strategy.admission.denied.strategy_vocabulary"
            }
            Self::Denied(Denial::RequestedLaneDoesNotMatchFamilyLane) => {
                "layout.strategy.admission.denied.family_lane"
            }
            Self::Denied(Denial::RequestedScopeDoesNotMatchKeyDomain) => {
                "layout.strategy.admission.denied.scope"
            }
            Self::Denied(Denial::MaintenanceModeIncompatibleWithRequestedLane) => {
                "layout.strategy.admission.denied.maintenance_mode"
            }
            Self::Denied(Denial::MutationShapeIncompatibleWithStrategy) => {
                "layout.strategy.admission.denied.mutation_shape"
            }
            Self::Denied(Denial::MigrationPostureIncompatibleWithStrategy) => {
                "layout.strategy.admission.denied.migration_posture"
            }
            Self::Denied(Denial::StrategyDoesNotSupportRequestedCapability) => {
                "layout.strategy.admission.denied.capability"
            }
            Self::Denied(Denial::ComparatorLawRequired) => {
                "layout.strategy.admission.denied.comparator_law"
            }
            Self::Denied(Denial::PrefixLawRequired) => {
                "layout.strategy.admission.denied.prefix_law"
            }
            Self::Denied(Denial::RangeBoundLawRequired) => {
                "layout.strategy.admission.denied.range_law"
            }
            Self::Denied(Denial::HashEqualityLawDoesNotMatchKeyDomain) => {
                "layout.strategy.admission.denied.hash_equality_law"
            }
            Self::Denied(Denial::CompositeOrderingLawDoesNotMatchKeyDomain) => {
                "layout.strategy.admission.denied.composite_ordering_law"
            }
            Self::Denied(Denial::CoverageFamilyDoesNotMatchStrategy) => {
                "layout.strategy.admission.denied.coverage_family"
            }
            Self::Denied(Denial::ExactMaterializationRequired) => {
                "layout.strategy.admission.denied.exact_materialization"
            }
            Self::Denied(Denial::ExactCoverageDenied) => {
                "layout.strategy.admission.denied.exact_coverage"
            }
        }
    }
}

pub fn layout_admission_cases() -> impl Iterator<Item = LayoutAdmissionCaseId> {
    std::iter::once(LayoutAdmissionCaseId::Admitted).chain(
        super::LayoutAdmissionDenialCase::ALL
            .into_iter()
            .map(LayoutAdmissionCaseId::Denied),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAdmissionView<'a> {
    Admitted(&'a LayoutStrategyRegistrySnapshot),
    Denied(&'a LayoutAdmissionDenial),
}

impl LayoutAdmissionOutcome {
    pub fn view(&self) -> LayoutAdmissionView<'_> {
        match &self.case {
            LayoutAdmissionPayload::Success(snapshot) => LayoutAdmissionView::Admitted(snapshot),
            LayoutAdmissionPayload::Denied(denial) => LayoutAdmissionView::Denied(denial),
        }
    }

    pub fn case_id(&self) -> LayoutAdmissionCaseId {
        match &self.case {
            LayoutAdmissionPayload::Success(_) => LayoutAdmissionCaseId::Admitted,
            LayoutAdmissionPayload::Denied(denial) => LayoutAdmissionCaseId::Denied(denial.case()),
        }
    }

    fn success(value: LayoutStrategyRegistrySnapshot) -> Self {
        Self::from_owner_payload(LayoutAdmissionPayload::Success(value))
    }

    fn denied(value: LayoutAdmissionDenial) -> Self {
        Self::from_owner_payload(LayoutAdmissionPayload::Denied(Box::new(value)))
    }

    fn from_owner_payload(case: LayoutAdmissionPayload) -> Self {
        Self { case }
    }

    fn into_owner_payload(self) -> LayoutAdmissionPayload {
        self.case
    }
}

impl LayoutAdmissionOutcome {
    pub fn into_result(self) -> Result<LayoutStrategyRegistrySnapshot, LayoutAdmissionDenial> {
        match self.into_owner_payload() {
            LayoutAdmissionPayload::Success(value) => Ok(value),
            LayoutAdmissionPayload::Denied(denial) => Err(*denial),
        }
    }
    #[cfg(test)]
    pub fn unwrap(self) -> LayoutStrategyRegistrySnapshot {
        self.into_result().unwrap()
    }
    #[cfg(test)]
    pub fn unwrap_err(self) -> LayoutAdmissionDenial {
        self.into_result().unwrap_err()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutAdmissionRegistryFacade;

impl LayoutAdmissionRegistryFacade {
    pub fn admit(&self, request: LayoutAdmissionRequest) -> LayoutAdmissionOutcome {
        match derive_registry_snapshot(request) {
            Ok(snapshot) => LayoutAdmissionOutcome::success(snapshot),
            Err(denial) => LayoutAdmissionOutcome::denied(denial),
        }
    }
}

fn derive_registry_snapshot(
    request: LayoutAdmissionRequest,
) -> Result<LayoutStrategyRegistrySnapshot, LayoutAdmissionDenial> {
    let admitted = match admit_strategy_from_basis(request.authority_basis(), request.family()) {
        Ok(admitted) => admitted,
        Err(denial) => {
            return Err(LayoutAdmissionDenial::StrategyVocabularyDenied(denial));
        }
    };

    if !family_lane_supports_operation(admitted.declared_access_lane(), request.requested_lane()) {
        return Err(LayoutAdmissionDenial::RequestedLaneDoesNotMatchFamilyLane {
            family: admitted.family(),
            requested_lane: request.requested_lane(),
            declared_lane: admitted.declared_access_lane(),
        });
    }

    if request.required_scope_partition() != request.key_domain().scope() {
        return Err(LayoutAdmissionDenial::RequestedScopeDoesNotMatchKeyDomain {
            requested_scope: request.required_scope_partition(),
            key_domain_scope: request.key_domain().scope(),
        });
    }

    if !maintenance_mode_supports_lane(request.maintenance_mode(), request.requested_lane()) {
        return Err(
            LayoutAdmissionDenial::MaintenanceModeIncompatibleWithRequestedLane {
                family: admitted.family(),
                maintenance_mode: request.maintenance_mode(),
                requested_lane: request.requested_lane(),
            },
        );
    }

    if !mutation_shape_is_compatible(admitted.family(), request.mutation_shape()) {
        return Err(
            LayoutAdmissionDenial::MutationShapeIncompatibleWithStrategy {
                family: admitted.family(),
                mutation_shape: request.mutation_shape(),
            },
        );
    }

    if let Some(required_posture) = request.required_migration_posture()
        && admitted.migration_posture() != required_posture
    {
        return Err(
            LayoutAdmissionDenial::MigrationPostureIncompatibleWithStrategy {
                family: admitted.family(),
                required_migration_posture: required_posture,
                admitted_migration_posture: admitted.migration_posture(),
            },
        );
    }

    require_materialization_compatibility(admitted, &request)?;

    require_key_law_compatibility(admitted, &request)?;

    let (hash_equality_law, composite_ordering_law) =
        require_requested_key_law_compatibility(admitted, &request)?;

    let granted_capability =
        LayoutStrategyCapability::from_requested(request.requested_capability());
    Ok(LayoutStrategyRegistrySnapshot::issue(
        admitted,
        request,
        granted_capability,
        hash_equality_law,
        composite_ordering_law,
    ))
}

pub const fn layout_admission_registry() -> LayoutAdmissionRegistryFacade {
    LayoutAdmissionRegistryFacade
}
