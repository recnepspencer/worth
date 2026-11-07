use crate::basis_lifecycle::ScopedSubscriptionDeclarationBasis;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::budget::QuerySubscriptionWorkBudget;
use super::counters::QuerySubscriptionDeclarationCounters;
use super::diagnostic::QuerySubscriptionDiagnosticStage;
use super::equivalence::QuerySubscriptionEquivalenceBasis;
use super::error::{
    QuerySubscriptionFamilySelectionError, QuerySubscriptionFamilySelectionFailureClass,
};
use super::family::QuerySubscriptionFamily;
use super::future_selection::QuerySubscriptionFutureSelection;
use super::input::LiveQueryAdmissionArtifact;
use super::posture::{
    QuerySubscriptionAllocationPosture, QuerySubscriptionBasisPosture,
    QuerySubscriptionBridgePosture, QuerySubscriptionCostPosture,
};
use super::selection_future::validate_future_selection;
use super::selection_live_graph_access::{
    live_graph_access_posture_for_family, QuerySubscriptionLiveGraphAccessPosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionFamilySelection {
    family: QuerySubscriptionFamily,
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    future_selection: QuerySubscriptionFutureSelection,
    cost_posture: QuerySubscriptionCostPosture,
    live_graph_access_posture: QuerySubscriptionLiveGraphAccessPosture,
    basis_posture: QuerySubscriptionBasisPosture,
    scoped_declaration_basis: Option<ScopedSubscriptionDeclarationBasis>,
    bridge_posture: QuerySubscriptionBridgePosture,
    equivalence_basis: QuerySubscriptionEquivalenceBasis,
    work_budget: QuerySubscriptionWorkBudget,
    required_slice_count: usize,
    authorized_projection_width: usize,
    ordering_width: usize,
    grouping_width: usize,
    relation_scope_width: usize,
    view_shape_metadata_width: usize,
    counters: QuerySubscriptionDeclarationCounters,
}

impl QuerySubscriptionFamilySelection {
    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn live_family(&self) -> &LiveQueryFamily {
        &self.live_family
    }

    pub fn view_family(&self) -> Option<LiveViewShapeFamily> {
        self.view_family
    }

    pub fn cost_posture(&self) -> &QuerySubscriptionCostPosture {
        &self.cost_posture
    }

    pub fn live_graph_access_posture(&self) -> &QuerySubscriptionLiveGraphAccessPosture {
        &self.live_graph_access_posture
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_posture(&self) -> &QuerySubscriptionBasisPosture {
        &self.basis_posture
    }

    pub fn scoped_declaration_basis(&self) -> Option<&ScopedSubscriptionDeclarationBasis> {
        self.scoped_declaration_basis.as_ref()
    }

    pub fn bridge_posture(&self) -> &QuerySubscriptionBridgePosture {
        &self.bridge_posture
    }

    pub fn equivalence_basis(&self) -> &QuerySubscriptionEquivalenceBasis {
        &self.equivalence_basis
    }

    pub fn work_budget(&self) -> &QuerySubscriptionWorkBudget {
        &self.work_budget
    }

    pub fn required_slice_count(&self) -> usize {
        self.required_slice_count
    }

    pub fn authorized_projection_width(&self) -> usize {
        self.authorized_projection_width
    }

    pub fn ordering_width(&self) -> usize {
        self.ordering_width
    }

    pub fn grouping_width(&self) -> usize {
        self.grouping_width
    }

    pub fn relation_scope_width(&self) -> usize {
        self.relation_scope_width
    }

    pub fn view_shape_metadata_width(&self) -> usize {
        self.view_shape_metadata_width
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }
}

pub(super) struct FamilyClassification {
    pub(super) family: QuerySubscriptionFamily,
    pub(super) cost_posture: QuerySubscriptionCostPosture,
    pub(super) bridge_posture: QuerySubscriptionBridgePosture,
}

pub fn select_query_subscription_family(
    live: LiveQueryAdmissionArtifact,
    budget: QuerySubscriptionWorkBudget,
) -> Result<QuerySubscriptionFamilySelection, QuerySubscriptionFamilySelectionError> {
    let source_identity = live.diagnostic_source_identity();
    let mut counters = QuerySubscriptionDeclarationCounters::default();

    if !live.relationship_proof_posture.admits_subscription() {
        counters.family_denial_count = 1;
        counters.relationship_proof_drift_denial_count = 1;
        return Err(QuerySubscriptionFamilySelectionError::new(
            QuerySubscriptionFamilySelectionFailureClass::RelationshipProofAdmissionDrift,
            "subscription relationship proof posture drifted after live admission",
            QuerySubscriptionDiagnosticStage::RelationshipProofDrift,
            source_identity,
            counters,
        ));
    }

    if budget.bridge_family_map_lookup_limit == 0 {
        counters.family_denial_count = 1;
        counters.work_budget_denial_count = 1;
        return Err(QuerySubscriptionFamilySelectionError::new(
            QuerySubscriptionFamilySelectionFailureClass::WorkBudgetExceeded,
            "subscription family selection requires one bridge-family registry lookup",
            QuerySubscriptionDiagnosticStage::FamilySelection,
            source_identity,
            counters,
        ));
    }

    counters.family_registry_lookup_count = 1;
    counters.view_family_registry_lookup_count = u64::from(live.view_family.is_some());

    let classification = classify_subscription_family(&live, source_identity, &mut counters)?;
    validate_future_selection(
        &live,
        &classification.family,
        source_identity,
        &mut counters,
    )?;
    validate_admission_dimensions(
        &live,
        &classification.family,
        source_identity,
        &mut counters,
    )?;
    let required_slice_count = required_slice_count(&live, &classification.family);
    let required_policy_width = live.policy_context_width() + live.tenant_context_width();

    if live.authorized_projection_width > budget.authorized_projection_width_limit
        || live.view_shape_metadata_width > budget.view_shape_metadata_width_limit
        || required_policy_width > budget.policy_tenant_digest_width_limit
        || required_slice_count > budget.max_admitted_slice_count
    {
        counters.family_denial_count = 1;
        counters.work_budget_denial_count = 1;
        return Err(QuerySubscriptionFamilySelectionError::new(
            QuerySubscriptionFamilySelectionFailureClass::WorkBudgetExceeded,
            format!(
                "subscription family {:?} needs {} slices within an explicit work budget",
                classification.family, required_slice_count
            ),
            QuerySubscriptionDiagnosticStage::FamilySelection,
            source_identity,
            counters,
        ));
    }

    if budget.allocation_posture == QuerySubscriptionAllocationPosture::NoAllocation {
        counters.family_denial_count = 1;
        counters.work_budget_denial_count = 1;
        counters.forbidden_heap_allocation_denial_count = 1;
        return Err(QuerySubscriptionFamilySelectionError::new(
            QuerySubscriptionFamilySelectionFailureClass::AllocationBudgetExceeded,
            "subscription family selection currently requires a scratch buffer for meaning digest construction",
            QuerySubscriptionDiagnosticStage::FamilySelection,
            source_identity,
            counters,
        ));
    }

    counters.family_selection_count = 1;
    counters.scratch_allocation_count = 1;
    let equivalence_basis = QuerySubscriptionEquivalenceBasis::new(&live, &classification)
        .map_err(|cause| {
            QuerySubscriptionFamilySelectionError::identity_refused(cause, counters.clone())
        })?;
    counters.equivalence_digest_part_count = equivalence_basis.digest_part_count() as u64;
    let live_graph_access_posture = live_graph_access_posture_for_family(&classification.family);

    Ok(QuerySubscriptionFamilySelection {
        family: classification.family,
        live_family: live.live_family,
        view_family: live.view_family,
        future_selection: live.future_selection,
        cost_posture: classification.cost_posture,
        live_graph_access_posture,
        basis_posture: live.basis_posture,
        scoped_declaration_basis: live.scoped_declaration_basis,
        bridge_posture: classification.bridge_posture,
        equivalence_basis,
        work_budget: budget,
        required_slice_count,
        authorized_projection_width: live.authorized_projection_width,
        ordering_width: live.ordering_width,
        grouping_width: live.grouping_width,
        relation_scope_width: live.relation_scope_width,
        view_shape_metadata_width: live.view_shape_metadata_width,
        counters,
    })
}

fn classify_subscription_family(
    live: &LiveQueryAdmissionArtifact,
    source_identity: &WorthQueryEvidenceIdentity,
    counters: &mut QuerySubscriptionDeclarationCounters,
) -> Result<FamilyClassification, QuerySubscriptionFamilySelectionError> {
    let family = match live.view_family {
        Some(view_family) => {
            let expected_live_family = view_family.underlying_live_family();
            if expected_live_family != live.live_family {
                counters.family_denial_count = 1;
                return Err(QuerySubscriptionFamilySelectionError::new(
                    QuerySubscriptionFamilySelectionFailureClass::ViewFamilyLiveFamilyMismatch,
                    format!(
                        "view family {} requires live family {}, not {}",
                        view_family.as_str(),
                        expected_live_family.as_str(),
                        live.live_family.as_str()
                    ),
                    QuerySubscriptionDiagnosticStage::ViewMismatch,
                    source_identity,
                    counters.clone(),
                ));
            }
            match view_family {
                LiveViewShapeFamily::Table => QuerySubscriptionFamily::CollectionMembership,
                LiveViewShapeFamily::Detail => QuerySubscriptionFamily::DetailExact,
                LiveViewShapeFamily::InspectorDetailObserved
                | LiveViewShapeFamily::InspectorDetailFocused => {
                    QuerySubscriptionFamily::InspectorDetailExact
                }
                LiveViewShapeFamily::KanbanGrouped => {
                    QuerySubscriptionFamily::GroupedCollectionMembership
                }
            }
        }
        None => match &live.live_family {
            LiveQueryFamily::Detail => QuerySubscriptionFamily::DetailExact,
            LiveQueryFamily::OrderedCollection => QuerySubscriptionFamily::CollectionMembership,
            LiveQueryFamily::BoundedMaterialization => {
                QuerySubscriptionFamily::BoundedMaterialization
            }
        },
    };

    let cost_posture = match family {
        QuerySubscriptionFamily::DetailExact | QuerySubscriptionFamily::InspectorDetailExact => {
            QuerySubscriptionCostPosture::BoundedExact
        }
        QuerySubscriptionFamily::CollectionMembership
        | QuerySubscriptionFamily::BoundedMaterialization => {
            QuerySubscriptionCostPosture::BoundedMembership
        }
        QuerySubscriptionFamily::GroupedCollectionMembership => {
            QuerySubscriptionCostPosture::BoundedWithViewGrouping
        }
    };

    Ok(FamilyClassification {
        family,
        cost_posture,
        bridge_posture: QuerySubscriptionBridgePosture::BridgeDeclarationAdmitted,
    })
}

fn validate_admission_dimensions(
    live: &LiveQueryAdmissionArtifact,
    family: &QuerySubscriptionFamily,
    source_identity: &WorthQueryEvidenceIdentity,
    counters: &mut QuerySubscriptionDeclarationCounters,
) -> Result<(), QuerySubscriptionFamilySelectionError> {
    let valid = match family {
        QuerySubscriptionFamily::DetailExact => {
            live.authorized_projection_width > 0
                && live.ordering_width == 0
                && live.grouping_width == 0
                && live.relation_scope_width == 0
                && live.view_shape_metadata_width == 0
        }
        QuerySubscriptionFamily::InspectorDetailExact => {
            live.authorized_projection_width > 0
                && live.ordering_width == 0
                && live.grouping_width == 0
                && live.relation_scope_width == 0
                && live.view_shape_metadata_width > 0
        }
        QuerySubscriptionFamily::CollectionMembership => {
            live.authorized_projection_width > 0
                && live.ordering_width > 0
                && live.grouping_width == 0
                && live.relation_scope_width == 0
                && live.view_shape_metadata_width == 0
        }
        QuerySubscriptionFamily::GroupedCollectionMembership => {
            live.authorized_projection_width > 0
                && live.grouping_width > 0
                && live.relation_scope_width == 0
                && live.view_shape_metadata_width > 0
        }
        QuerySubscriptionFamily::BoundedMaterialization => {
            live.authorized_projection_width > 0
                && live.ordering_width > 0
                && live.grouping_width == 0
                && live.relation_scope_width > 0
                && live.view_shape_metadata_width == 0
        }
    };

    if valid {
        return Ok(());
    }

    counters.family_denial_count = 1;
    counters.admission_dimension_denial_count = 1;
    Err(QuerySubscriptionFamilySelectionError::new(
        QuerySubscriptionFamilySelectionFailureClass::InvalidAdmissionDimensions,
        format!(
            "subscription family {:?} received dimensions outside its admitted phase-1 width contract",
            family
        ),
        QuerySubscriptionDiagnosticStage::FamilySelection,
        source_identity,
        counters.clone(),
    ))
}

fn required_slice_count(
    live: &LiveQueryAdmissionArtifact,
    family: &QuerySubscriptionFamily,
) -> usize {
    match family {
        QuerySubscriptionFamily::DetailExact => live.authorized_projection_width,
        QuerySubscriptionFamily::InspectorDetailExact => {
            live.authorized_projection_width + live.view_shape_metadata_width
        }
        QuerySubscriptionFamily::CollectionMembership => {
            live.authorized_projection_width + live.ordering_width + 1
        }
        QuerySubscriptionFamily::GroupedCollectionMembership => {
            live.authorized_projection_width
                + live.ordering_width
                + live.grouping_width
                + live.view_shape_metadata_width
                + 1
        }
        QuerySubscriptionFamily::BoundedMaterialization => {
            live.authorized_projection_width + live.ordering_width + live.relation_scope_width + 1
        }
    }
}
