use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationFieldComparison,
    RelationalAuthorizationObservationCounters, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPredicate,
    RelationalAuthorizationRelatedEntityConstraint,
};
use worth_relational::facade::indexes::{
    BoundedIndexParityMode, BoundedRelationJoinLookupRequest, MAX_BOUNDED_INDEX_CANDIDATES,
};

pub(super) struct SelectedCapabilityGrant {
    grant: worth_relational::facade::identity::EntityId,
    work: RelationalAuthorizationObservationCounters,
}

impl SelectedCapabilityGrant {
    pub(super) const fn into_parts(
        self,
    ) -> (
        worth_relational::facade::identity::EntityId,
        RelationalAuthorizationObservationCounters,
    ) {
        (self.grant, self.work)
    }
}

use super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRuntimeTimeSample,
};

pub(super) fn select_exact_grant(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
) -> Result<SelectedCapabilityGrant, WorthQueryOperationAuthorizationDenial> {
    let lookup =
        lookup_candidate_grants(execution, relational, snapshot.clone(), installed, request)?;
    let paths = prepare_candidate_paths(installed, request, sample, lookup.candidate_entity_ids())?;
    let evidence =
        observe_candidate_paths(execution, relational, snapshot, installed, request, paths)?;
    let grant = selected_grant(installed, &evidence)?;
    let work = selection_work(&lookup, evidence.counters());
    Ok(SelectedCapabilityGrant { grant, work })
}

fn lookup_candidate_grants(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
) -> Result<
    worth_relational::facade::indexes::BoundedRelationJoinLookupOutcome,
    WorthQueryOperationAuthorizationDenial,
> {
    let lookup_request = BoundedRelationJoinLookupRequest::new(
        snapshot,
        installed.grant_join_index_id(),
        request.principal(),
        request.resource(),
        MAX_BOUNDED_INDEX_CANDIDATES,
    )
    .map_err(|_| invalid_policy(installed.contract().name()))?;
    let lookup = relational
        .index_access()
        .execute_bounded_relation_join_lookup(
            lookup_request,
            BoundedIndexParityMode::Production,
            execution,
        )
        .map_err(|denied| {
            if let worth_relational::facade::indexes::BoundedRelationJoinLookupDenialKind::Execution(cause)=denied.kind() {
                return super::super::denial::relational_read_denial(cause,installed.contract().name());
            }
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::RelationalObservationRejected),
                installed.contract().name(),
            )
        })?;
    if lookup.overflowed() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::GrantSelectionLimitExceeded,
            ),
            installed.contract().name(),
        ));
    }
    let candidates = lookup.candidate_entity_ids();
    if candidates.is_empty() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityGrantMissing,
            ),
            installed.contract().name(),
        ));
    }
    Ok(lookup)
}

fn prepare_candidate_paths(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    candidates: &[worth_relational::facade::identity::EntityId],
) -> Result<Vec<RelationalAuthorizationPathPlan>, WorthQueryOperationAuthorizationDenial> {
    candidates
        .iter()
        .map(|grant| {
            prepare_grant_path(installed, request, sample).map(|path| {
                path.with_entity_anchors([RelationalAuthorizationEntityAnchor::new(
                    installed.grant_witness().entity_ordinal(),
                    installed.grant_kind(),
                    *grant,
                )])
            })
        })
        .collect()
}

fn observe_candidate_paths(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    paths: Vec<RelationalAuthorizationPathPlan>,
) -> Result<
    worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    WorthQueryOperationAuthorizationDenial,
> {
    let plan = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        request.principal(),
        request.resource(),
        installed.principal_kind(),
        installed.scope_kind(),
        paths,
        [],
    )
    .map_err(|_| invalid_policy(installed.contract().name()))?;
    relational
        .observe_authorization(plan, execution)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::RelationalObservationRejected,
                ),
                installed.contract().name(),
            )
        })
}

fn selected_grant(
    installed: &WorthQueryInstalledCapabilityPlan,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
) -> Result<worth_relational::facade::identity::EntityId, WorthQueryOperationAuthorizationDenial> {
    evidence
        .paths()
        .iter()
        .find_map(|path| path.witness())
        .and_then(|witness| witness.entity_at(installed.grant_witness().entity_ordinal()))
        .ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::CapabilityAuthorizationMissing,
                ),
                installed.contract().name(),
            )
        })
}

fn selection_work(
    lookup: &worth_relational::facade::indexes::BoundedRelationJoinLookupOutcome,
    mut work: RelationalAuthorizationObservationCounters,
) -> RelationalAuthorizationObservationCounters {
    work.relation_join_index_lookups += 1;
    work.relation_join_candidates_inspected += lookup.examined_entry_count();
    work.entity_records_inspected += lookup.verified_entity_record_count();
    work.relation_records_inspected += lookup.verified_relation_record_count();
    work.maximum_frontier_width = work
        .maximum_frontier_width
        .max(lookup.examined_entry_count());
    work
}

pub(super) fn prepare_grant_path(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
) -> Result<RelationalAuthorizationPathPlan, WorthQueryOperationAuthorizationDenial> {
    let template = grant_template(installed)?;
    let mut predicates = template.plan.predicates().to_vec();
    append_grant_predicates(installed, request, sample, &mut predicates);
    let mut plan = template.plan.clone().with_predicates(predicates);
    if let (Some(traversal), Some(entity)) =
        (&installed.request().related_relation, request.related())
    {
        plan = plan.with_related_entities([RelationalAuthorizationRelatedEntityConstraint::new(
            installed.grant_witness().entity_ordinal(),
            traversal.clone(),
            entity,
        )]);
    }
    Ok(plan)
}

fn grant_template(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Result<
    &super::super::capability_registry::WorthQueryCapabilityPathTemplate,
    WorthQueryOperationAuthorizationDenial,
> {
    let template = installed
        .paths()
        .get(installed.grant_witness().path_index())
        .ok_or_else(|| invalid_policy(installed.contract().name()))?;
    if !template.context_anchors.is_empty()
        || template.grant_ordinal != Some(installed.grant_witness().entity_ordinal())
    {
        return Err(invalid_policy(installed.contract().name()));
    }
    Ok(template)
}

fn append_grant_predicates(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    predicates: &mut Vec<RelationalAuthorizationPredicate>,
) {
    let ordinal = installed.grant_witness().entity_ordinal();
    predicates.push(RelationalAuthorizationPredicate::compare(
        ordinal,
        installed.grant_kind(),
        installed.request().not_before.clone(),
        RelationalAuthorizationFieldComparison::AtMost,
        sample.value().clone(),
    ));
    predicates.push(RelationalAuthorizationPredicate::compare(
        ordinal,
        installed.grant_kind(),
        installed.request().not_after.clone(),
        RelationalAuthorizationFieldComparison::AtLeast,
        sample.value().clone(),
    ));
    if let (Some(field), Some(value)) = (&installed.request().field, projection.field()) {
        predicates.push(RelationalAuthorizationPredicate::new(
            ordinal,
            installed.grant_kind(),
            field.clone(),
            value.clone(),
        ));
    }
    if let (Some(field), Some(value)) = (&installed.request().magnitude, projection.magnitude()) {
        predicates.push(RelationalAuthorizationPredicate::compare(
            ordinal,
            installed.grant_kind(),
            field.clone(),
            RelationalAuthorizationFieldComparison::AtLeast,
            value.clone(),
        ));
    }
}

fn invalid_policy(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidInstalledPolicy,
        ),
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
