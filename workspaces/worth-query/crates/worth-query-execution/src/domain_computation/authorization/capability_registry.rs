use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationSchema,
};
use worth_runtime_bridge::facade::BridgeAuthorizationInstallationBatch;

use super::capability_lowering::compile_capability_plan;
pub(super) use super::capability_lowering::WorthQueryInstalledCapabilityPlan;
use super::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

mod delegation;
pub(super) use delegation::{
    WorthQueryCapabilityDelegationActivationBindings, WorthQueryCapabilityDelegationBindings,
    WorthQueryCapabilityRevocationBindings,
};
mod elevation;
pub(super) use elevation::{
    WorthQueryCapabilityElevationBindings, WorthQueryCapabilityElevationLifecycleBindings,
    WorthQueryCapabilityElevationTemporalBindings, WorthQueryCapabilityUpperBoundBindings,
};
mod elevation_lifecycle;
pub(super) use elevation_lifecycle::WorthQueryElevationLifecycleOperationRole;
use elevation_lifecycle::WorthQueryInstalledElevationLifecycleRegistry;
mod bindings;
pub(super) use bindings::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryCapabilityPlanCompilationEvidence {
    capability_count: usize,
    path_count: usize,
    rule_count: usize,
    clause_count: usize,
    guard_count: usize,
    context_anchor_count: usize,
    elevation_lifecycle_operation_count: usize,
    canonical_basis_preparations: usize,
    digest_derivations: usize,
    digest_text_materializations: usize,
}

impl WorthQueryCapabilityPlanCompilationEvidence {
    pub const fn capability_count(self) -> usize {
        self.capability_count
    }

    pub const fn path_count(self) -> usize {
        self.path_count
    }

    pub const fn rule_count(self) -> usize {
        self.rule_count
    }

    pub const fn clause_count(self) -> usize {
        self.clause_count
    }

    pub const fn guard_count(self) -> usize {
        self.guard_count
    }

    pub const fn context_anchor_count(self) -> usize {
        self.context_anchor_count
    }

    pub const fn elevation_lifecycle_operation_count(self) -> usize {
        self.elevation_lifecycle_operation_count
    }

    pub const fn canonical_basis_preparations(self) -> usize {
        self.canonical_basis_preparations
    }

    pub const fn digest_derivations(self) -> usize {
        self.digest_derivations
    }

    pub const fn digest_text_materializations(self) -> usize {
        self.digest_text_materializations
    }

    pub(super) fn record(&mut self, plan: &WorthQueryInstalledCapabilityPlan) {
        self.capability_count += 1;
        self.path_count += plan.paths().len();
        self.rule_count += plan.rules().len();
        self.clause_count += plan.paths().len();
        self.guard_count += plan
            .paths()
            .iter()
            .filter(|path| !matches!(path.guard, WorthQueryCapabilityRequestGuard::Unconditional))
            .count();
        self.context_anchor_count += plan
            .paths()
            .iter()
            .map(|path| path.context_anchors.len())
            .sum::<usize>();
    }
}

pub(super) struct WorthQueryInstalledCapabilityRegistry {
    plans: BTreeMap<[u8; 32], WorthQueryInstalledCapabilityPlan>,
    elevation_lifecycles: WorthQueryInstalledElevationLifecycleRegistry,
    compilation: WorthQueryCapabilityPlanCompilationEvidence,
}

impl WorthQueryInstalledCapabilityRegistry {
    pub(super) fn compile<Schema>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        layout: &WorthQueryPrimaryGraphLayout,
        bridge_installation: &mut BridgeAuthorizationInstallationBatch,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
    {
        let mut plans = BTreeMap::new();
        let mut elevation_lifecycles = WorthQueryInstalledElevationLifecycleRegistry::default();
        let mut compilation = WorthQueryCapabilityPlanCompilationEvidence::default();
        for source in schema.capability_plan_sources() {
            let plan = compile_capability_plan(schema, source, layout, bridge_installation)?;
            compilation.record(&plan);
            if plans.insert(*source.identity().bytes(), plan).is_some() {
                return Err(super::authorization_denial(
                    source.contract().name(),
                    "duplicate installed capability plan",
                ));
            }
        }
        for (capability_identity, plan) in &plans {
            elevation_lifecycles
                .install(*capability_identity, &plans, plan.contract())
                .map_err(|()| {
                    super::authorization_denial(
                        plan.contract().name(),
                        "invalid or competing installed elevation lifecycle transition",
                    )
                })?;
        }
        compilation.elevation_lifecycle_operation_count = elevation_lifecycles.len();
        Ok(Self {
            plans,
            elevation_lifecycles,
            compilation,
        })
    }

    pub(super) fn plan<Schema, Capability, Operation, Input>(
        &self,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    ) -> Option<&WorthQueryInstalledCapabilityPlan> {
        self.plans.get(capability.identity().bytes())
    }

    pub(super) fn plan_by_identity(
        &self,
        capability_identity: &[u8; 32],
    ) -> Option<&WorthQueryInstalledCapabilityPlan> {
        self.plans.get(capability_identity)
    }

    pub(super) fn elevation_lifecycle_operation<Schema, Operation>(
        &self,
        operation: &str,
        input_type: &str,
    ) -> Result<
        Option<(
            [u8; 32],
            [u8; 32],
            WorthQueryElevationLifecycleOperationRole,
        )>,
        (),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        self.elevation_lifecycles
            .operation::<Schema, Operation>(operation, input_type)
    }

    pub(super) const fn compilation(&self) -> WorthQueryCapabilityPlanCompilationEvidence {
        self.compilation
    }
}
