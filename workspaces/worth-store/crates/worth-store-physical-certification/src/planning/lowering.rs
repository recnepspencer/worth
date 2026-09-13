use crate::{CertifiedPhysicalScenario, PhysicalScenarioFaultKind};

use super::forbidden_shortcuts::ROADMAP_2_BASELINE_SHORTCUTS;
use super::plan::PhysicalSimulationPlanParts;
use super::proof_progression::admit_simulation_plan;
use super::requirements::RequiredSimulationPlanShape;
use super::{
    ForbiddenShortcutSet, PhysicalSimulationPlan, SimulationPlanDenial, SimulationPlanningContext,
};
use crate::{AdmittedDriverContractSet, YieldpointScheduleBinding};

pub fn lower_physical_simulation_plan(
    scenario: CertifiedPhysicalScenario,
    context: SimulationPlanningContext,
) -> Result<PhysicalSimulationPlan, SimulationPlanDenial> {
    require_supported_profile(&context)?;
    require_resource_envelope_profile(&context)?;
    require_non_ambiguous_fault_scope(&scenario)?;
    let required_shape = RequiredSimulationPlanShape::from_scenario(scenario.definition())?;
    require_capabilities(&context, &required_shape)?;
    let (yieldpoint_binding, driver_contracts) =
        require_supported_plan_requirements(&context, &required_shape, &scenario)?;
    let evidence_policy = context
        .evidence_policy()
        .ok_or(SimulationPlanDenial::MissingEvidencePolicy)?;
    let forbidden_shortcuts = context
        .forbidden_shortcuts()
        .ok_or(SimulationPlanDenial::AbsentForbiddenShortcutSet)?;
    require_certification_forbidden_shortcuts(forbidden_shortcuts)?;
    let plan = PhysicalSimulationPlan::from_parts(PhysicalSimulationPlanParts {
        scenario_identity: scenario.identity().clone(),
        scenario_family: scenario.definition().family(),
        profile: context.profile(),
        resource_envelope: context.resource_envelope(),
        required_capabilities: required_shape.capabilities,
        actors: required_shape.actors,
        drivers: required_shape.drivers,
        driver_contracts,
        yieldpoint_binding,
        observers: required_shape.observers,
        oracle_families: required_shape.oracle_families,
        counter_contracts: required_shape.counter_contracts,
        fixture_classes: required_shape.fixture_classes,
        evidence_policy,
        forbidden_shortcuts: forbidden_shortcuts.clone(),
        blob_harness_metadata: scenario.definition().expectation().blob_harness_metadata(),
        blob_harness_topology: scenario.definition().expectation().blob_harness_topology(),
    })?;
    admit_simulation_plan(plan)
}

fn require_resource_envelope_profile(
    context: &SimulationPlanningContext,
) -> Result<(), SimulationPlanDenial> {
    if context.resource_envelope().profile() != context.profile() {
        return Err(SimulationPlanDenial::ResourceEnvelopeProfileMismatch {
            expected: context.profile(),
            actual: context.resource_envelope().profile(),
        });
    }
    Ok(())
}

fn require_supported_profile(
    context: &SimulationPlanningContext,
) -> Result<(), SimulationPlanDenial> {
    if !context.supported_profiles().contains(context.profile()) {
        return Err(SimulationPlanDenial::UnsupportedProfile(context.profile()));
    }
    Ok(())
}

fn require_non_ambiguous_fault_scope(
    scenario: &CertifiedPhysicalScenario,
) -> Result<(), SimulationPlanDenial> {
    if scenario.definition().fault().kind() == PhysicalScenarioFaultKind::FutureExtensionSlot {
        return Err(SimulationPlanDenial::AmbiguousFaultScope);
    }
    Ok(())
}

fn require_capabilities(
    context: &SimulationPlanningContext,
    required_shape: &RequiredSimulationPlanShape,
) -> Result<(), SimulationPlanDenial> {
    for capability in required_shape.capabilities.iter() {
        if !context.capabilities().contains(capability) {
            return Err(SimulationPlanDenial::MissingCapability(capability));
        }
    }
    Ok(())
}

fn require_supported_plan_requirements(
    context: &SimulationPlanningContext,
    required_shape: &RequiredSimulationPlanShape,
    scenario: &CertifiedPhysicalScenario,
) -> Result<(YieldpointScheduleBinding, AdmittedDriverContractSet), SimulationPlanDenial> {
    for driver in required_shape.drivers.iter() {
        if !context.driver_contracts().contains_driver(driver) {
            return Err(SimulationPlanDenial::MissingPhysicalDriver(driver));
        }
    }
    let driver_contracts = context
        .driver_contracts()
        .select_required_drivers(required_shape.drivers.iter());
    let scheduled_yieldpoint = scenario
        .definition()
        .schedule()
        .production_boundary_yieldpoint();
    let yieldpoint_binding = driver_contracts
        .bind_required_schedule_yieldpoint(scheduled_yieldpoint, required_shape.drivers.iter())
        .ok_or_else(|| {
            SimulationPlanDenial::UnboundYieldpointSchedule(scheduled_yieldpoint.to_owned())
        })?;
    for observer in required_shape.observers.iter() {
        if !context.supported_observers().contains(observer) {
            return Err(SimulationPlanDenial::MissingObserver(observer));
        }
    }
    for oracle_family in required_shape.oracle_families.iter() {
        if !context.supported_oracle_families().contains(oracle_family) {
            return Err(SimulationPlanDenial::MissingOracleFamily(oracle_family));
        }
    }
    Ok((yieldpoint_binding, driver_contracts))
}

fn require_certification_forbidden_shortcuts(
    forbidden_shortcuts: &ForbiddenShortcutSet,
) -> Result<(), SimulationPlanDenial> {
    for shortcut in ROADMAP_2_BASELINE_SHORTCUTS {
        if !forbidden_shortcuts.contains(shortcut) {
            return Err(SimulationPlanDenial::MissingForbiddenShortcut(shortcut));
        }
    }
    Ok(())
}
