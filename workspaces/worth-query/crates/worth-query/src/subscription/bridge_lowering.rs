use crate::basis_lifecycle::ScopedSubscriptionDeclarationBasis;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::identity_authority::{
    project_query_subscription_evidence, QueryProjectionIdentity, QuerySubscriptionIdentityKind,
};

use super::basis_request::QuerySubscriptionBasisBindingRequest;
use super::bridge_family::{
    BridgeSubscriptionDeclarationFamilyKind, QueryToBridgeSubscriptionFamilyMap,
};
use super::bridge_lowering_budget::{
    QuerySubscriptionBridgeFallbackPosture, QuerySubscriptionBridgeLoweringBudget,
};
use super::bridge_lowering_error::{
    QuerySubscriptionBridgeLoweringDenialKind, QuerySubscriptionBridgeLoweringError,
};
use super::bridge_slice::{BridgeSubscriptionSliceKind, QueryToBridgeSliceMap};
use super::counters::QuerySubscriptionDeclarationCounters;
use super::declaration::QuerySubscriptionDeclarationArtifact;
use super::diagnostic::QuerySubscriptionDiagnosticStage;
use super::evidence_identities::bridge_lowering_plan_identity;
use super::future_selection::QuerySubscriptionFutureSelection;
use super::posture::QuerySubscriptionBasisPosture;
use super::posture::QuerySubscriptionBridgePosture;
use super::signal_strategy::QuerySubscriptionSignalStrategyRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeSubscriptionLoweringPlan {
    query_declaration_identity: WorthQueryEvidenceIdentity,
    bridge_declaration_identity: WorthQueryEvidenceIdentity,
    family_map: QueryToBridgeSubscriptionFamilyMap,
    slice_map: QueryToBridgeSliceMap,
    future_selection: QuerySubscriptionFutureSelection,
    basis_request: QuerySubscriptionBasisBindingRequest,
    scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
    signal_strategy_request: QuerySubscriptionSignalStrategyRequest,
    lowering_budget: QuerySubscriptionBridgeLoweringBudget,
    counters: QuerySubscriptionDeclarationCounters,
}

impl BridgeSubscriptionLoweringPlan {
    pub fn query_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.query_declaration_identity)
    }

    pub fn query_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_declaration_identity
    }

    pub fn bridge_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.bridge_declaration_identity)
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn bridge_family(&self) -> &BridgeSubscriptionDeclarationFamilyKind {
        self.family_map.bridge_family()
    }

    pub fn bridge_slices(&self) -> &[BridgeSubscriptionSliceKind] {
        self.slice_map.bridge_slices()
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_request(&self) -> &QuerySubscriptionBasisBindingRequest {
        &self.basis_request
    }

    pub fn scoped_declaration_basis(&self) -> &ScopedSubscriptionDeclarationBasis {
        &self.scoped_declaration_basis
    }

    pub fn signal_strategy_request(&self) -> &QuerySubscriptionSignalStrategyRequest {
        &self.signal_strategy_request
    }

    pub fn lowering_budget(&self) -> &QuerySubscriptionBridgeLoweringBudget {
        &self.lowering_budget
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }
}

pub fn lower_query_subscription_to_bridge(
    declaration: QuerySubscriptionDeclarationArtifact,
    lowering_budget: QuerySubscriptionBridgeLoweringBudget,
) -> Result<BridgeSubscriptionLoweringPlan, QuerySubscriptionBridgeLoweringError> {
    let mut counters = declaration.counters().clone();
    let source_identity = declaration.declaration_identity();
    counters.bridge_family_registry_lookup_count = 1;

    let family_map = QueryToBridgeSubscriptionFamilyMap::for_query_family(declaration.family());
    if !lowering_budget.admits_bridge_family(family_map.bridge_family()) {
        counters.bridge_family_denial_count = 1;
        return Err(QuerySubscriptionBridgeLoweringError::new(
            QuerySubscriptionBridgeLoweringDenialKind::BridgeFamilyUnsupported,
            "bridge family registry does not admit this query subscription family",
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            source_identity,
            counters,
        ));
    }

    let slice_map = QueryToBridgeSliceMap::from_slice_intent(declaration.slice_intent());
    counters.bridge_slice_registry_lookup_count = slice_map.bridge_slices().len() as u64;

    if !lowering_budget.admits_bridge_slices(slice_map.bridge_slices()) {
        counters.bridge_slice_denial_count = 1;
        return Err(QuerySubscriptionBridgeLoweringError::new(
            QuerySubscriptionBridgeLoweringDenialKind::BridgeSliceUnsupported,
            "bridge slice registry does not admit every query subscription slice",
            QuerySubscriptionDiagnosticStage::BridgeSliceLowering,
            source_identity,
            counters,
        ));
    }

    if declaration.bridge_posture() == &QuerySubscriptionBridgePosture::BridgeLoweringDeferred
        && lowering_budget.bridge_fallback_posture()
            != &QuerySubscriptionBridgeFallbackPosture::CertifiedFallbackAdmitted
    {
        counters.bridge_fallback_denial_count = 1;
        return Err(QuerySubscriptionBridgeLoweringError::new(
            QuerySubscriptionBridgeLoweringDenialKind::BridgeFallbackUnsupported,
            "bridge fallback lowering is explicit debt and is not admitted by this lowering budget",
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            source_identity,
            counters,
        ));
    }

    if basis_denied(declaration.basis_posture(), &lowering_budget) {
        counters.basis_binding_denial_count = 1;
        return Err(QuerySubscriptionBridgeLoweringError::new(
            QuerySubscriptionBridgeLoweringDenialKind::BasisBindingUnsupported,
            "bridge basis binding cannot honestly bind this query subscription basis",
            QuerySubscriptionDiagnosticStage::BasisBinding,
            source_identity,
            counters,
        ));
    }

    let scoped_declaration_basis =
        declaration
            .scoped_declaration_basis()
            .cloned()
            .ok_or_else(|| {
                counters.basis_binding_denial_count = 1;
                QuerySubscriptionBridgeLoweringError::new(
                    QuerySubscriptionBridgeLoweringDenialKind::BasisBindingUnsupported,
                    "subscription lowering requires Query-minted scoped declaration basis proof",
                    QuerySubscriptionDiagnosticStage::BasisBinding,
                    source_identity,
                    counters.clone(),
                )
            })?;

    if exceeds_lowering_budget(&slice_map, &lowering_budget) {
        counters.work_budget_denial_count = 1;
        return Err(QuerySubscriptionBridgeLoweringError::new(
            QuerySubscriptionBridgeLoweringDenialKind::LoweringBudgetExceeded,
            "bridge lowering exceeds its explicit bridge lowering budget",
            QuerySubscriptionDiagnosticStage::BridgeSliceLowering,
            source_identity,
            counters,
        ));
    }

    let basis_request = QuerySubscriptionBasisBindingRequest::from_declaration(&declaration)
        .map_err(|cause| {
            QuerySubscriptionBridgeLoweringError::identity_refused(cause, counters.clone())
        })?;
    let signal_strategy_request =
        QuerySubscriptionSignalStrategyRequest::for_bridge_family(family_map.bridge_family());
    counters.bridge_lowering_count = 1;
    counters.bridge_slice_count = slice_map.bridge_slices().len() as u64;
    counters.basis_binding_request_count = 1;
    counters.signal_strategy_request_count = 1;

    let query_declaration_identity = declaration.declaration_identity().clone();
    let bridge_declaration_identity = bridge_lowering_plan_identity(
        &query_declaration_identity,
        family_map.bridge_family(),
        basis_request.evidence_identity(),
        signal_strategy_request.evidence_identity(),
        slice_map.bridge_slices(),
    )
    .map_err(|cause| {
        QuerySubscriptionBridgeLoweringError::identity_refused(cause, counters.clone())
    })?;

    Ok(BridgeSubscriptionLoweringPlan {
        query_declaration_identity,
        bridge_declaration_identity,
        family_map,
        slice_map,
        future_selection: declaration.future_selection().clone(),
        basis_request,
        scoped_declaration_basis,
        signal_strategy_request,
        lowering_budget,
        counters,
    })
}

fn basis_denied(
    basis: &QuerySubscriptionBasisPosture,
    budget: &QuerySubscriptionBridgeLoweringBudget,
) -> bool {
    match basis {
        QuerySubscriptionBasisPosture::CurrentHead | QuerySubscriptionBasisPosture::BranchHead => {
            false
        }
        QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot => {
            !budget.historical_basis_support()
        }
        QuerySubscriptionBasisPosture::PreviewScoped => !budget.preview_basis_support(),
        QuerySubscriptionBasisPosture::DeniedUnsupportedBasis => true,
    }
}

fn exceeds_lowering_budget(
    slice_map: &QueryToBridgeSliceMap,
    budget: &QuerySubscriptionBridgeLoweringBudget,
) -> bool {
    budget.bridge_family_registry_lookup_limit() < 1
        || slice_map.bridge_slices().len() > budget.bridge_slice_registry_lookup_limit()
        || slice_map.bridge_slices().len() > budget.bridge_declaration_input_width_limit()
        || budget.basis_request_width_limit() < 1
        || budget.signal_strategy_request_width_limit() < 1
}
