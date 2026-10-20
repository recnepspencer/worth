//! Complete installed capability product and its atomic installation transition.

use std::sync::Arc;

use worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityContract;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapabilityPlanSource,
    WorthQueryInstalledApplicationSchema,
};
use worth_runtime_bridge::facade::{
    BridgeAuthorizationInstallationBatch, BridgeAuthorizationInstallationRequest,
    BridgeAuthorizationRuleEffect,
};

use super::{
    authorization_denial, capability_principal, compile_composition_rules, compile_grant_path,
    elevation, kind, request_bindings, WorthQueryCapabilityRuleLoweringAccumulator,
    WorthQueryCapabilityRuleSet,
};
use crate::domain_computation::authorization::capability_registry::{
    WorthQueryCapabilityDecisionRuleBindings, WorthQueryCapabilityDelegationBindings,
    WorthQueryCapabilityElevationBindings, WorthQueryCapabilityGrantWitnessBinding,
    WorthQueryCapabilityPathTemplate, WorthQueryCapabilityRequestBindings,
    WorthQueryCapabilityRuleBinding, WorthQueryCapabilityUpperBoundBindings,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

pub(in crate::domain_computation::authorization) struct WorthQueryInstalledCapabilityPlan {
    correspondence: worth_runtime_bridge::facade::BridgeAuthorizationCorrespondenceIdentity,
    capability_authority_identity: Arc<str>,
    contract: ErasedApplicationCapabilityContract,
    principal_kind: worth_relational::facade::identity::KindId,
    grant_kind: worth_relational::facade::identity::KindId,
    scope_kind: worth_relational::facade::identity::KindId,
    grant_join_index_id: worth_relational::facade::indexes::DerivedIndexId,
    grant_witness: WorthQueryCapabilityGrantWitnessBinding,
    rule_set: WorthQueryCapabilityRuleSet,
    decision_rules: WorthQueryCapabilityDecisionRuleBindings,
    request: WorthQueryCapabilityRequestBindings,
    delegation: WorthQueryCapabilityDelegationBindings,
    elevation: Option<WorthQueryCapabilityElevationBindings>,
    upper_bound: Option<WorthQueryCapabilityUpperBoundBindings>,
}

impl WorthQueryInstalledCapabilityPlan {
    pub(in crate::domain_computation::authorization) const fn correspondence(
        &self,
    ) -> worth_runtime_bridge::facade::BridgeAuthorizationCorrespondenceIdentity {
        self.correspondence
    }
    pub(in crate::domain_computation::authorization) const fn capability_authority_identity(
        &self,
    ) -> &Arc<str> {
        &self.capability_authority_identity
    }
    pub(in crate::domain_computation::authorization) const fn contract(
        &self,
    ) -> &ErasedApplicationCapabilityContract {
        &self.contract
    }
    pub(in crate::domain_computation::authorization) const fn principal_kind(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.principal_kind
    }
    pub(in crate::domain_computation::authorization) const fn grant_kind(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.grant_kind
    }
    pub(in crate::domain_computation::authorization) const fn scope_kind(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.scope_kind
    }
    pub(in crate::domain_computation::authorization) const fn grant_join_index_id(
        &self,
    ) -> worth_relational::facade::indexes::DerivedIndexId {
        self.grant_join_index_id
    }
    pub(in crate::domain_computation::authorization) const fn grant_witness(
        &self,
    ) -> WorthQueryCapabilityGrantWitnessBinding {
        self.grant_witness
    }
    pub(in crate::domain_computation::authorization) const fn decision_rules(
        &self,
    ) -> &WorthQueryCapabilityDecisionRuleBindings {
        &self.decision_rules
    }
    pub(in crate::domain_computation::authorization) const fn request(
        &self,
    ) -> &WorthQueryCapabilityRequestBindings {
        &self.request
    }
    pub(in crate::domain_computation::authorization) const fn delegation(
        &self,
    ) -> &WorthQueryCapabilityDelegationBindings {
        &self.delegation
    }
    pub(in crate::domain_computation::authorization) const fn elevation(
        &self,
    ) -> &Option<WorthQueryCapabilityElevationBindings> {
        &self.elevation
    }
    pub(in crate::domain_computation::authorization) const fn upper_bound(
        &self,
    ) -> &Option<WorthQueryCapabilityUpperBoundBindings> {
        &self.upper_bound
    }
    pub(in crate::domain_computation::authorization) fn paths(
        &self,
    ) -> &[WorthQueryCapabilityPathTemplate] {
        self.rule_set.paths()
    }
    pub(in crate::domain_computation::authorization) fn rules(
        &self,
    ) -> &[WorthQueryCapabilityRuleBinding] {
        self.rule_set.rules()
    }
}

pub(in crate::domain_computation::authorization) fn compile_capability_plan<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    source: WorthQueryInstalledApplicationCapabilityPlanSource<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    bridge_installation: &mut BridgeAuthorizationInstallationBatch,
) -> Result<WorthQueryInstalledCapabilityPlan, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    let contract = source.contract();
    let principal_kind = kind(layout, capability_principal(contract)?)?;
    let grant_kind = kind(layout, contract.grant_entity())?;
    let scope_entity = contract.target().resource_path().scope_entity();
    let scope_kind = kind(layout, scope_entity)?;
    let grant_join_index_id = layout
        .capability_grant_join_index_id(
            contract.delegation().grantee().relation(),
            contract.target().resource().relation(),
        )
        .ok_or_else(|| {
            authorization_denial(contract.name(), "capability grant join is not installed")
        })?;
    let mut lowering = WorthQueryCapabilityRuleLoweringAccumulator::new();
    let grant_path_index = lowering.path_count();
    let _ = lowering.add_rule(
        BridgeAuthorizationRuleEffect::Required,
        vec![vec![compile_grant_path(
            source.identity().bytes(),
            contract,
            layout,
            principal_kind,
            grant_kind,
            scope_kind,
        )?]],
    );
    let grant_witness = WorthQueryCapabilityGrantWitnessBinding::new(grant_path_index, 1);
    let decision_rules =
        compile_composition_rules(contract, layout, source.identity().bytes(), &mut lowering)?;
    let upper_bound_lowering = lowering.completed_prefix();
    let elevation = elevation::compile_elevation_rules(
        contract,
        layout,
        source.identity().bytes(),
        principal_kind,
        grant_kind,
        scope_kind,
        &mut lowering,
    )?;
    let completed = lowering.finish();
    let upper_bound_lowering = completed.completed_prefix(upper_bound_lowering);
    let request = request_bindings(contract, layout)?;
    let delegation = WorthQueryCapabilityDelegationBindings::compile(contract, layout)?;
    let upper_bound = if elevation.is_some() {
        let upper_bound_path_count = upper_bound_lowering.path_count();
        let identity = source.elevation_upper_bound_identity();
        let correspondence = bridge_installation
            .add(BridgeAuthorizationInstallationRequest::new(
                &identity,
                super::super::bridge_authorization_binding_identity(&schema.binding_identity()),
                format!("{}:elevation-upper-bound", contract.name()),
                scope_entity,
                format!("{}:elevation-upper-bound", contract.operation()),
                upper_bound_lowering
                    .rules()
                    .iter()
                    .map(|rule| rule.bridge().clone()),
            ))
            .map_err(|denial| {
                authorization_denial(denial.subject(), "Bridge rejected elevation upper bound")
            })?;
        Some(WorthQueryCapabilityUpperBoundBindings {
            correspondence,
            path_count: upper_bound_path_count,
            rules: upper_bound_lowering.into_storage(),
            decision_rules: decision_rules.clone(),
        })
    } else {
        None
    };
    let correspondence = bridge_installation
        .add(BridgeAuthorizationInstallationRequest::new(
            source.identity().digest(),
            super::super::bridge_authorization_binding_identity(&schema.binding_identity()),
            contract.name(),
            scope_entity,
            contract.operation(),
            completed.rules().iter().map(|rule| rule.bridge().clone()),
        ))
        .map_err(|denial| authorization_denial(denial.subject(), "Bridge rejected capability"))?;
    Ok(WorthQueryInstalledCapabilityPlan {
        correspondence,
        capability_authority_identity: Arc::from(source.authority_identity()),
        contract: contract.clone(),
        principal_kind,
        grant_kind,
        scope_kind,
        grant_join_index_id,
        grant_witness,
        request,
        delegation,
        elevation,
        upper_bound,
        rule_set: completed.into_storage(),
        decision_rules,
    })
}
