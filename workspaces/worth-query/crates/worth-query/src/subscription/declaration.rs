use super::counters::QuerySubscriptionDeclarationCounters;
use super::declaration_digest::QuerySubscriptionDeclarationDigest;
use super::declaration_error::{
    QuerySubscriptionDeclarationDenial, QuerySubscriptionDeclarationDenialKind,
};
use super::delivery::QuerySubscriptionDeliveryIntent;
use super::diagnostic::QuerySubscriptionDiagnosticStage;
use super::evidence_identities::query_subscription_declaration_identity;
use super::family::QuerySubscriptionFamily;
use super::future_selection::QuerySubscriptionFutureSelection;
use super::posture::{
    QuerySubscriptionAllocationPosture, QuerySubscriptionBasisPosture,
    QuerySubscriptionBridgePosture, QuerySubscriptionCostPosture,
};
use super::selection::QuerySubscriptionFamilySelection;
use super::selection_live_graph_access::QuerySubscriptionLiveGraphAccessPosture;
use super::slice::{
    QuerySubscriptionSliceIntent, QuerySubscriptionSliceKind, QuerySubscriptionSlicePart,
};
use super::slice_budget::QuerySubscriptionSliceBudget;
use crate::basis_lifecycle::ScopedSubscriptionDeclarationBasis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDeclarationArtifact {
    family: QuerySubscriptionFamily,
    future_selection: QuerySubscriptionFutureSelection,
    cost_posture: QuerySubscriptionCostPosture,
    basis_posture: QuerySubscriptionBasisPosture,
    scoped_declaration_basis: Option<ScopedSubscriptionDeclarationBasis>,
    bridge_posture: QuerySubscriptionBridgePosture,
    live_graph_access_posture: QuerySubscriptionLiveGraphAccessPosture,
    equivalence_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    slice_intent: QuerySubscriptionSliceIntent,
    delivery_intent: QuerySubscriptionDeliveryIntent,
    declaration_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    declaration_digest: QuerySubscriptionDeclarationDigest,
    slice_budget: QuerySubscriptionSliceBudget,
    counters: QuerySubscriptionDeclarationCounters,
}

impl QuerySubscriptionDeclarationArtifact {
    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn cost_posture(&self) -> &QuerySubscriptionCostPosture {
        &self.cost_posture
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

    pub fn live_graph_access_posture(&self) -> &QuerySubscriptionLiveGraphAccessPosture {
        &self.live_graph_access_posture
    }

    pub fn equivalence_identity(&self) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.equivalence_identity
    }

    pub fn slice_intent(&self) -> &QuerySubscriptionSliceIntent {
        &self.slice_intent
    }

    pub fn delivery_intent(&self) -> &QuerySubscriptionDeliveryIntent {
        &self.delivery_intent
    }

    pub fn declaration_identity(&self) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.declaration_identity
    }
    pub fn slice_budget(&self) -> &QuerySubscriptionSliceBudget {
        &self.slice_budget
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }

    #[cfg(test)]
    pub(super) fn with_bridge_posture(
        mut self,
        bridge_posture: QuerySubscriptionBridgePosture,
    ) -> Self {
        self.bridge_posture = bridge_posture;
        self
    }
}

pub fn declare_query_subscription(
    selection: QuerySubscriptionFamilySelection,
    slice_budget: QuerySubscriptionSliceBudget,
) -> Result<QuerySubscriptionDeclarationArtifact, QuerySubscriptionDeclarationDenial> {
    let mut counters = selection.counters().clone();
    let source_identity = selection.equivalence_basis().evidence_identity();
    let raw_parts = raw_slice_parts(&selection);
    let input_count = raw_parts.len();
    let sort_comparison_count = input_count.saturating_sub(1);

    if slice_budget.masked_slice_request_detected() {
        counters.declaration_denial_count = 1;
        counters.masked_slice_denial_count = 1;
        return Err(QuerySubscriptionDeclarationDenial::new(
            QuerySubscriptionDeclarationDenialKind::UnsupportedMaskedSlice,
            "masked subscription slices require purpose-specific non-disclosing evidence",
            QuerySubscriptionDiagnosticStage::Declaration,
            source_identity,
            counters,
        ));
    }

    if selection.family() == &QuerySubscriptionFamily::GroupedCollectionMembership
        && !slice_budget.grouping_slice_support()
    {
        counters.declaration_denial_count = 1;
        return Err(QuerySubscriptionDeclarationDenial::new(
            QuerySubscriptionDeclarationDenialKind::UnsupportedGroupingSlice,
            "grouped subscription declaration requires admitted grouping slice support",
            QuerySubscriptionDiagnosticStage::Declaration,
            source_identity,
            counters,
        ));
    }

    if selection.family() == &QuerySubscriptionFamily::BoundedMaterialization
        && !slice_budget.bounded_materialization_slice_support()
    {
        counters.declaration_denial_count = 1;
        return Err(QuerySubscriptionDeclarationDenial::new(
            QuerySubscriptionDeclarationDenialKind::UnsupportedBoundedMaterializationSlice,
            "bounded materialization declaration requires admitted relation-scope slice support",
            QuerySubscriptionDiagnosticStage::Declaration,
            source_identity,
            counters,
        ));
    }

    if !slice_budget.delivery_intent_support() {
        counters.declaration_denial_count = 1;
        counters.delivery_intent_denial_count = 1;
        return Err(QuerySubscriptionDeclarationDenial::new(
            QuerySubscriptionDeclarationDenialKind::DeliveryIntentUnsupported,
            "subscription declaration requires admitted delivery intent support",
            QuerySubscriptionDiagnosticStage::DeliveryIntent,
            source_identity,
            counters,
        ));
    }

    if slice_budget.allocation_posture() == &QuerySubscriptionAllocationPosture::NoAllocation {
        counters.declaration_denial_count = 1;
        counters.work_budget_denial_count = 1;
        counters.forbidden_heap_allocation_denial_count = 1;
        return Err(QuerySubscriptionDeclarationDenial::new(
            QuerySubscriptionDeclarationDenialKind::AllocationBudgetExceeded,
            "subscription declaration requires scratch sorting and deduplication",
            QuerySubscriptionDiagnosticStage::Declaration,
            source_identity,
            counters,
        ));
    }

    if exceeds_slice_budget(
        &selection,
        input_count,
        sort_comparison_count,
        &slice_budget,
    ) {
        counters.declaration_denial_count = 1;
        counters.work_budget_denial_count = 1;
        return Err(QuerySubscriptionDeclarationDenial::new(
            QuerySubscriptionDeclarationDenialKind::SliceBudgetExceeded,
            "subscription declaration slice intent exceeds its explicit slice budget",
            QuerySubscriptionDiagnosticStage::Declaration,
            source_identity,
            counters,
        ));
    }

    counters.scratch_allocation_count += 1;
    let slice_intent =
        QuerySubscriptionSliceIntent::from_canonical_parts(raw_parts).map_err(|cause| {
            QuerySubscriptionDeclarationDenial::identity_refused(cause, counters.clone())
        })?;
    let delivery_intent = QuerySubscriptionDeliveryIntent::for_family(selection.family());

    counters.declaration_count = 1;
    counters.declared_slice_count = input_count as u64;
    counters.deduplicated_slice_count = slice_intent.len() as u64;
    counters.slice_deduplication_input_count = input_count as u64;
    counters.slice_sort_comparison_count = sort_comparison_count as u64;

    let equivalence_identity = selection.equivalence_basis().evidence_identity().clone();
    let declaration_identity = query_subscription_declaration_identity(
        selection.family(),
        selection.live_family(),
        selection.view_family(),
        selection.cost_posture(),
        selection.basis_posture(),
        selection.bridge_posture(),
        selection.live_graph_access_posture(),
        selection.future_selection(),
        &equivalence_identity,
        &slice_intent,
        &delivery_intent,
        selection.work_budget().max_admitted_slice_count(),
        &slice_budget,
    )
    .map_err(|cause| {
        QuerySubscriptionDeclarationDenial::identity_refused(cause, counters.clone())
    })?;
    counters.declaration_digest_part_count = 18;
    let declaration_digest =
        QuerySubscriptionDeclarationDigest::from_evidence_identity(&declaration_identity);

    Ok(QuerySubscriptionDeclarationArtifact {
        family: selection.family().clone(),
        future_selection: selection.future_selection().clone(),
        cost_posture: selection.cost_posture().clone(),
        basis_posture: selection.basis_posture().clone(),
        scoped_declaration_basis: selection.scoped_declaration_basis().cloned(),
        bridge_posture: selection.bridge_posture().clone(),
        live_graph_access_posture: *selection.live_graph_access_posture(),
        equivalence_identity,
        slice_intent,
        delivery_intent,
        declaration_identity,
        declaration_digest,
        slice_budget,
        counters,
    })
}

fn raw_slice_parts(
    selection: &QuerySubscriptionFamilySelection,
) -> Vec<QuerySubscriptionSlicePart> {
    let mut parts = projected_parts(selection.authorized_projection_width());
    match selection.family() {
        QuerySubscriptionFamily::DetailExact => {}
        QuerySubscriptionFamily::CollectionMembership => {
            parts.push(QuerySubscriptionSlicePart::new(
                QuerySubscriptionSliceKind::Membership,
                0,
            ));
            parts.extend(ordering_parts(selection.ordering_width()));
        }
        QuerySubscriptionFamily::GroupedCollectionMembership => {
            parts.push(QuerySubscriptionSlicePart::new(
                QuerySubscriptionSliceKind::Membership,
                0,
            ));
            parts.extend(ordering_parts(selection.ordering_width()));
            parts.extend(grouping_parts(selection.grouping_width()));
            parts.extend(metadata_parts(selection.view_shape_metadata_width()));
        }
        QuerySubscriptionFamily::InspectorDetailExact => {
            parts.extend(metadata_parts(selection.view_shape_metadata_width()));
        }
        QuerySubscriptionFamily::BoundedMaterialization => {
            parts.push(QuerySubscriptionSlicePart::new(
                QuerySubscriptionSliceKind::Membership,
                0,
            ));
            parts.extend(ordering_parts(selection.ordering_width()));
            parts.extend(relation_scope_parts(selection.relation_scope_width()));
        }
    }
    parts
}

fn projected_parts(width: usize) -> Vec<QuerySubscriptionSlicePart> {
    (0..width)
        .map(|ordinal| {
            QuerySubscriptionSlicePart::new(
                QuerySubscriptionSliceKind::AuthorizedProjection,
                ordinal,
            )
        })
        .collect()
}

fn ordering_parts(width: usize) -> Vec<QuerySubscriptionSlicePart> {
    (0..width)
        .map(|ordinal| {
            QuerySubscriptionSlicePart::new(QuerySubscriptionSliceKind::Ordering, ordinal)
        })
        .collect()
}

fn grouping_parts(width: usize) -> Vec<QuerySubscriptionSlicePart> {
    (0..width)
        .map(|ordinal| {
            QuerySubscriptionSlicePart::new(QuerySubscriptionSliceKind::Grouping, ordinal)
        })
        .collect()
}

fn relation_scope_parts(width: usize) -> Vec<QuerySubscriptionSlicePart> {
    (0..width)
        .map(|ordinal| {
            QuerySubscriptionSlicePart::new(QuerySubscriptionSliceKind::RelationScope, ordinal)
        })
        .collect()
}

fn metadata_parts(width: usize) -> Vec<QuerySubscriptionSlicePart> {
    (0..width)
        .map(|ordinal| {
            QuerySubscriptionSlicePart::new(QuerySubscriptionSliceKind::ViewShapeMetadata, ordinal)
        })
        .collect()
}

fn exceeds_slice_budget(
    selection: &QuerySubscriptionFamilySelection,
    input_count: usize,
    sort_comparison_count: usize,
    budget: &QuerySubscriptionSliceBudget,
) -> bool {
    selection.authorized_projection_width() > budget.projected_slice_width_limit()
        || selection.ordering_width() > budget.ordering_slice_width_limit()
        || selection.grouping_width() > budget.grouping_slice_width_limit()
        || selection.relation_scope_width() > budget.relation_scope_slice_width_limit()
        || selection.view_shape_metadata_width() > budget.metadata_slice_width_limit()
        || input_count > budget.deduplication_input_width_limit()
        || input_count > budget.deduplicated_output_width_limit()
        || sort_comparison_count > budget.sort_comparison_limit()
}
