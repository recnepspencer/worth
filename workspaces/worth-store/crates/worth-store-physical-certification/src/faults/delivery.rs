use super::boundary::ObservedFaultBoundary;
use super::denial::{deny_unbound_fault_yieldpoint, FaultDeliveryDenial};
use super::event::{PhysicalFaultEvent, PhysicalFaultEventKind};
use super::locus::{ExpectedFaultLocalization, PhysicalArtifactFaultLocus};
use crate::{PhysicalBoundarySeam, PhysicalBoundaryYieldpoint, YieldpointScheduleBinding};
use worth_proof::{
    AssumptionBasis, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    CurrentValidity, ExecutedRecipe, ExecutionReadyRecipe, ExecutionReadyRecipeDxExt,
    FreshnessScopedBasis, Lowered, LoweredRecipeDxExt, Recipe,
};

pub type FaultDeliveryProofBasis =
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<FaultDeliveryBoundaryProof>>;
pub type LoweredFaultDeliveryRecipe = Recipe<Lowered, FaultDeliveryPlan, FaultDeliveryProofBasis>;
pub type ExecutionReadyFaultDeliveryRecipe =
    ExecutionReadyRecipe<FaultDeliveryPlan, FaultDeliveryProofBasis>;
pub type BoundaryObservedFaultDeliveryRecipe =
    ExecutedRecipe<FaultDeliveryPlan, FaultDeliveryProofBasis>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultDeliveryPlan {
    event: PhysicalFaultEvent,
    yieldpoint: PhysicalBoundaryYieldpoint,
    actual_boundary: Option<ObservedFaultBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultDeliveryBoundaryProof {
    event_kind: PhysicalFaultEventKind,
    scheduled_yieldpoint: String,
    delivery_yieldpoint: PhysicalBoundaryYieldpoint,
    seam: PhysicalBoundarySeam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultDeliveryReceipt {
    event_kind: PhysicalFaultEventKind,
    locus: Option<PhysicalArtifactFaultLocus>,
    seam: PhysicalBoundarySeam,
    yieldpoint: PhysicalBoundaryYieldpoint,
    expected_localization: Option<ExpectedFaultLocalization>,
    actual_boundary: ObservedFaultBoundary,
}

struct FaultDeliveryResolutionAuthority;
impl AuthorityMarker for FaultDeliveryResolutionAuthority {}

struct FaultDeliveryLoweringCapability;
impl CapabilityMarker for FaultDeliveryLoweringCapability {}

struct FaultDeliveryReadinessAuthority;
impl AuthorityMarker for FaultDeliveryReadinessAuthority {}

impl FaultDeliveryPlan {
    pub fn lower(
        event: PhysicalFaultEvent,
        binding: &YieldpointScheduleBinding,
        yieldpoint: PhysicalBoundaryYieldpoint,
    ) -> Result<LoweredFaultDeliveryRecipe, FaultDeliveryDenial> {
        require_bound_yieldpoint(binding, &yieldpoint)?;
        require_event_yieldpoint_seam(&event, &yieldpoint)?;
        let proof = FaultDeliveryBoundaryProof {
            event_kind: event.kind(),
            scheduled_yieldpoint: binding.scheduled_yieldpoint().to_owned(),
            delivery_yieldpoint: yieldpoint.clone(),
            seam: yieldpoint.seam(),
        };
        let plan = Self {
            event,
            yieldpoint,
            actual_boundary: None,
        };

        Ok(Recipe::new(plan)
            .resolve_with_authority(
                proof,
                AuthorityWitness::from_authority_marker(FaultDeliveryResolutionAuthority),
            )
            .lower_with_capability(CapabilityWitness::from_capability_marker(
                FaultDeliveryLoweringCapability,
            )))
    }

    pub fn admit_execution_ready(
        lowered: LoweredFaultDeliveryRecipe,
        actual_boundary: ObservedFaultBoundary,
    ) -> Result<ExecutionReadyFaultDeliveryRecipe, FaultDeliveryDenial> {
        let (mut plan, basis) = lowered.into_parts();
        require_observed_boundary_matches_event(&plan.event, &actual_boundary)?;
        let proof = basis.basis().value().clone();
        plan.actual_boundary = Some(actual_boundary);
        Ok(Recipe::new(plan)
            .resolve_with_authority(
                proof,
                AuthorityWitness::from_authority_marker(FaultDeliveryResolutionAuthority),
            )
            .lower_with_capability(CapabilityWitness::from_capability_marker(
                FaultDeliveryLoweringCapability,
            ))
            .ready_with(
                AuthorityWitness::from_authority_marker(FaultDeliveryReadinessAuthority),
                "fault-delivery-runtime-boundary",
            ))
    }

    pub fn record_observed_boundary(
        ready: ExecutionReadyFaultDeliveryRecipe,
    ) -> BoundaryObservedFaultDeliveryRecipe {
        ready.execute()
    }

    pub fn receipt_from_observed_boundary(
        executed: BoundaryObservedFaultDeliveryRecipe,
    ) -> Result<FaultDeliveryReceipt, FaultDeliveryDenial> {
        let (plan, _) = executed.into_parts();
        let actual_boundary = plan
            .actual_boundary
            .ok_or(FaultDeliveryDenial::MissingObservedFaultBoundary)?;
        Ok(FaultDeliveryReceipt {
            event_kind: plan.event.kind(),
            locus: plan.event.locus().cloned(),
            seam: plan.yieldpoint.seam(),
            yieldpoint: plan.yieldpoint,
            expected_localization: plan
                .event
                .locus()
                .map(PhysicalArtifactFaultLocus::expected_localization),
            actual_boundary,
        })
    }

    pub fn deliver(
        event: PhysicalFaultEvent,
        binding: &YieldpointScheduleBinding,
        yieldpoint: PhysicalBoundaryYieldpoint,
        actual_boundary: ObservedFaultBoundary,
    ) -> Result<FaultDeliveryReceipt, FaultDeliveryDenial> {
        let lowered = Self::lower(event, binding, yieldpoint)?;
        let ready = Self::admit_execution_ready(lowered, actual_boundary)?;
        Self::receipt_from_observed_boundary(Self::record_observed_boundary(ready))
    }

    pub const fn proof_event_kind(&self) -> PhysicalFaultEventKind {
        self.event.kind()
    }

    pub const fn proof_yieldpoint(&self) -> &PhysicalBoundaryYieldpoint {
        &self.yieldpoint
    }
}

impl PhysicalFaultEvent {
    pub fn deliver_through(
        self,
        binding: &YieldpointScheduleBinding,
        yieldpoint: PhysicalBoundaryYieldpoint,
        actual_boundary: ObservedFaultBoundary,
    ) -> Result<FaultDeliveryReceipt, FaultDeliveryDenial> {
        FaultDeliveryPlan::deliver(self, binding, yieldpoint, actual_boundary)
    }
}

impl FaultDeliveryBoundaryProof {
    pub const fn event_kind(&self) -> PhysicalFaultEventKind {
        self.event_kind
    }

    pub fn scheduled_yieldpoint(&self) -> &str {
        &self.scheduled_yieldpoint
    }

    pub const fn delivery_yieldpoint(&self) -> &PhysicalBoundaryYieldpoint {
        &self.delivery_yieldpoint
    }

    pub const fn seam(&self) -> PhysicalBoundarySeam {
        self.seam
    }
}

impl FaultDeliveryReceipt {
    pub const fn event_kind(&self) -> PhysicalFaultEventKind {
        self.event_kind
    }

    pub const fn locus(&self) -> Option<&PhysicalArtifactFaultLocus> {
        self.locus.as_ref()
    }

    pub const fn seam(&self) -> PhysicalBoundarySeam {
        self.seam
    }

    pub const fn yieldpoint(&self) -> &PhysicalBoundaryYieldpoint {
        &self.yieldpoint
    }

    pub const fn expected_localization(&self) -> Option<ExpectedFaultLocalization> {
        self.expected_localization
    }

    pub const fn actual_boundary(&self) -> &ObservedFaultBoundary {
        &self.actual_boundary
    }
}

fn require_bound_yieldpoint(
    binding: &YieldpointScheduleBinding,
    yieldpoint: &PhysicalBoundaryYieldpoint,
) -> Result<(), FaultDeliveryDenial> {
    if binding.scheduled_yieldpoint() != yieldpoint.name()
        || binding.declared_yieldpoint() != yieldpoint
    {
        return Err(deny_unbound_fault_yieldpoint(
            binding.scheduled_yieldpoint(),
            yieldpoint,
        ));
    }
    Ok(())
}

fn require_event_yieldpoint_seam(
    event: &PhysicalFaultEvent,
    yieldpoint: &PhysicalBoundaryYieldpoint,
) -> Result<(), FaultDeliveryDenial> {
    let expected = event.required_seam();
    let actual = yieldpoint.seam();
    if expected == actual {
        Ok(())
    } else {
        Err(FaultDeliveryDenial::FaultYieldpointSeamMismatch { expected, actual })
    }
}

fn require_observed_boundary_matches_event(
    event: &PhysicalFaultEvent,
    actual_boundary: &ObservedFaultBoundary,
) -> Result<(), FaultDeliveryDenial> {
    if event.kind() == PhysicalFaultEventKind::NoFaultControl {
        return require_no_fault_observed_boundary(event, actual_boundary);
    }
    let Some(locus) = event.locus() else {
        return Ok(());
    };
    let expected = locus.expected_localization();
    if observed_boundary_satisfies_localization(expected, actual_boundary) {
        Ok(())
    } else {
        Err(FaultDeliveryDenial::ObservedFaultBoundaryMismatch {
            expected,
            actual: actual_boundary.boundary_kind(),
        })
    }
}

fn require_no_fault_observed_boundary(
    event: &PhysicalFaultEvent,
    actual_boundary: &ObservedFaultBoundary,
) -> Result<(), FaultDeliveryDenial> {
    if actual_boundary.no_fault_parity().is_some() {
        Ok(())
    } else {
        Err(FaultDeliveryDenial::ObservedFaultBoundaryMismatch {
            expected: event
                .locus()
                .map(PhysicalArtifactFaultLocus::expected_localization)
                .unwrap_or(ExpectedFaultLocalization::ProductionDriverBoundary),
            actual: actual_boundary.boundary_kind(),
        })
    }
}

const fn observed_boundary_satisfies_localization(
    expected: ExpectedFaultLocalization,
    actual_boundary: &ObservedFaultBoundary,
) -> bool {
    matches!(
        (expected, actual_boundary),
        (
            ExpectedFaultLocalization::FreshRuntimeRecoveryBoundary,
            ObservedFaultBoundary::FreshRuntimeCrashRecovery { .. }
        ) | (
            ExpectedFaultLocalization::ProductionDriverBoundary,
            ObservedFaultBoundary::IoPressureBoundary { .. }
        )
    )
}
