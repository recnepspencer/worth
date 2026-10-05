use std::sync::Arc;

use crate::domain_installation::{
    WorthQueryAdmittedInvalidationImpact, WorthQueryLiveProjectionRefresh,
};
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::projection_consumption::ConsumedFieldValueFact;

use super::indexed_effect::WorthQueryPerformedIndexedLivePatch;
use super::{WorthQueryCoalescedMaintenancePlan, WorthQueryMaintenanceStrategy};

/// Query-owned material produced by one performed granular maintenance pass.
///
/// This is deliberately downstream of the settled projection refresh. An
/// admitted role or maintenance strategy cannot mint it by itself.
pub enum WorthQueryPerformedMaintenanceEffect {
    ProjectionPatch(WorthQueryPerformedProjectionPatch),
    IndexedLivePatch(WorthQueryPerformedIndexedLivePatch),
}

pub(crate) struct WorthQueryDerivedMaintenanceEffect {
    pub(crate) effect: Arc<WorthQueryPerformedMaintenanceEffect>,
    pub(crate) projection_commit: crate::live::WorthQueryPendingProjectionMaintenanceState,
    pub(crate) collection_commit:
        Option<crate::domain_installation::WorthQueryPendingCollectionStateMutation>,
}

pub(crate) struct WorthQueryProjectionMaintenanceRequest<'a, D, O, F, L>
where
    L: crate::basis_lifecycle::BasisOperationLane,
{
    pub(crate) owner: &'a str,
    pub(crate) plan: &'a WorthQueryCoalescedMaintenancePlan,
    pub(crate) impacts: &'a [WorthQueryAdmittedInvalidationImpact],
    pub(crate) current:
        &'a crate::domain_installation::WorthQuerySettledDomainProjection<D, O, F, L>,
    pub(crate) refresh: &'a WorthQueryLiveProjectionRefresh,
}

pub(crate) struct WorthQueryPreparedProjectionMaintenance {
    pub(super) preview: crate::live::WorthQueryProjectionMaintenancePreview,
    pub(super) broad_collection_change: bool,
    pub(super) changed_native_targets:
        Vec<crate::domain_installation::WorthQueryCollectionChangedNativeTarget>,
    /// Every source entity of a full-scope refresh; empty for exact impacts.
    pub(super) full_scope_sources: Vec<WorthQueryEntityIdentity>,
}

impl WorthQueryPerformedMaintenanceEffect {
    pub const fn projection_patch(&self) -> Option<&WorthQueryPerformedProjectionPatch> {
        match self {
            Self::ProjectionPatch(patch) => Some(patch),
            Self::IndexedLivePatch(_) => None,
        }
    }

    pub const fn indexed_live_patch(&self) -> Option<&WorthQueryPerformedIndexedLivePatch> {
        match self {
            Self::ProjectionPatch(_) => None,
            Self::IndexedLivePatch(patch) => Some(patch),
        }
    }

    pub fn identity(&self) -> &str {
        match self {
            Self::ProjectionPatch(patch) => patch.identity(),
            Self::IndexedLivePatch(patch) => patch.identity(),
        }
    }
}

pub struct WorthQueryPerformedProjectionPatch {
    identity: String,
    fact_set_digest: String,
    affected_entities: Vec<WorthQueryEntityIdentity>,
    fields: Vec<ConsumedFieldValueFact>,
    prior_field_comparisons: usize,
}

impl WorthQueryPerformedProjectionPatch {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn fact_set_digest(&self) -> &str {
        &self.fact_set_digest
    }

    pub fn affected_entities(&self) -> &[WorthQueryEntityIdentity] {
        &self.affected_entities
    }

    pub fn fields(&self) -> &[ConsumedFieldValueFact] {
        &self.fields
    }

    pub const fn prior_field_comparisons(&self) -> usize {
        self.prior_field_comparisons
    }
}

pub(crate) fn derive_performed_maintenance_effect<
    D,
    O,
    F,
    L: crate::basis_lifecycle::BasisOperationLane,
>(
    plan: &WorthQueryCoalescedMaintenancePlan,
    impacts: &[WorthQueryAdmittedInvalidationImpact],
    current: &crate::domain_installation::WorthQuerySettledDomainProjection<D, O, F, L>,
    refresh: &WorthQueryLiveProjectionRefresh,
    collection: Option<&crate::domain_installation::WorthQueryCollectionConsumerWindow>,
    projection: WorthQueryPreparedProjectionMaintenance,
) -> Result<Option<WorthQueryDerivedMaintenanceEffect>, super::WorthQueryMaintenanceDenial> {
    let facts = refresh.authority().facts();
    let indexed_maintenance =
        plan.strategies() != [WorthQueryMaintenanceStrategy::LocalProjectionPatch];
    let (affected_entities, source_affected_entities) = affected_entity_sets(
        facts,
        impacts,
        &projection.full_scope_sources,
        indexed_maintenance,
    );
    if affected_entities.is_empty() {
        return Ok(None);
    }
    let (fields, prior_field_comparisons, projection_commit) = projection.preview.into_parts();
    if fields.is_empty() && !indexed_maintenance {
        return Ok(None);
    }
    let collection_maintenance = match collection {
        Some(collection) => {
            let affected = source_affected_entities.iter().cloned().collect();
            let keys = collection.keys_for_granular_change(
                projection.broad_collection_change,
                &projection.changed_native_targets,
            );
            let replacement_targets = collection.replacement_targets_for_granular_change(
                projection.broad_collection_change,
                &projection.changed_native_targets,
            );
            Some(
                collection
                    .prepare_granular_maintenance(
                        current,
                        refresh.source_rows(),
                        &affected,
                        &keys,
                        &replacement_targets,
                        collection_impact(plan),
                    )
                    .map_err(|_| super::WorthQueryMaintenanceDenial::PerformedEffectUnavailable)?,
            )
        }
        None if indexed_maintenance => {
            return Err(super::WorthQueryMaintenanceDenial::PerformedEffectUnavailable)
        }
        None => None,
    };
    if indexed_maintenance {
        let (mutation, collection_commit) = collection_maintenance
            .expect("indexed maintenance validated retained collection state above");
        return Ok(super::indexed_effect::derive(
            plan,
            refresh,
            affected_entities,
            fields,
            prior_field_comparisons,
            mutation,
        )
        .map(|effect| WorthQueryDerivedMaintenanceEffect {
            effect,
            projection_commit,
            collection_commit: Some(collection_commit),
        }));
    }
    Ok(Some(projection_effect(
        facts,
        affected_entities,
        fields,
        prior_field_comparisons,
        projection_commit,
        collection_maintenance.map(|(_, pending)| pending),
    )))
}

pub(crate) fn prepare_projection_maintenance<D, O, F, L>(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    request: WorthQueryProjectionMaintenanceRequest<'_, D, O, F, L>,
) -> WorthQueryPreparedProjectionMaintenance
where
    L: crate::basis_lifecycle::BasisOperationLane,
{
    let changed = changed_fields(request.impacts);
    let select_all_fields = request
        .plan
        .strategies()
        .contains(&WorthQueryMaintenanceStrategy::BoundedReexecution)
        || request.impacts.iter().any(|impact| {
            impact.roles().iter().any(|role| {
                matches!(
                    role,
                    crate::domain_installation::WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness
                        | crate::domain_installation::WorthQuerySemanticDependencyRole::InstalledDomainInvariant
                )
            })
        });
    let affected_sources = request
        .impacts
        .iter()
        .flat_map(|impact| impact.truth().change_set().changes())
        .filter_map(|change| change.relational_record_identity())
        .map(|identity| {
            WorthQueryEntityIdentity::from_relational_record(identity)
                .terminal_projection_for_reporting()
        })
        .collect();
    let preview = workspace.preview_projection_maintenance(
        request.owner,
        request.current.authority().facts(),
        request.refresh.authority().facts(),
        affected_sources,
        select_all_fields,
        changed.broad_projection_change,
        &changed.projection_targets,
    );
    WorthQueryPreparedProjectionMaintenance {
        preview,
        broad_collection_change: changed.broad_collection_change,
        changed_native_targets: changed.native_targets,
        full_scope_sources: Vec::new(),
    }
}

fn affected_entity_sets(
    facts: &crate::projection_consumption::ConsumedProjectionFactSet,
    impacts: &[WorthQueryAdmittedInvalidationImpact],
    full_scope_sources: &[WorthQueryEntityIdentity],
    indexed_maintenance: bool,
) -> (Vec<WorthQueryEntityIdentity>, Vec<WorthQueryEntityIdentity>) {
    let mut affected = facts
        .entity_identities()
        .iter()
        .map(|fact| fact.entity_identity().clone())
        .collect::<Vec<_>>();
    let mut source_affected = impacts
        .iter()
        .flat_map(|impact| impact.truth().change_set().changes())
        .filter_map(|change| change.relational_record_identity())
        .map(WorthQueryEntityIdentity::from_relational_record)
        .chain(full_scope_sources.iter().cloned())
        .collect::<Vec<_>>();
    source_affected.sort();
    source_affected.dedup();
    if indexed_maintenance {
        affected.extend(source_affected.iter().cloned());
        affected.sort();
        affected.dedup();
    }
    (affected, source_affected)
}

fn projection_effect(
    facts: &crate::projection_consumption::ConsumedProjectionFactSet,
    affected_entities: Vec<WorthQueryEntityIdentity>,
    fields: Vec<ConsumedFieldValueFact>,
    prior_field_comparisons: usize,
    projection_commit: crate::live::WorthQueryPendingProjectionMaintenanceState,
    collection_commit: Option<crate::domain_installation::WorthQueryPendingCollectionStateMutation>,
) -> WorthQueryDerivedMaintenanceEffect {
    let mut identity_parts = vec![
        "worth_query_performed_projection_patch_v1".to_owned(),
        format!("facts:{}", facts.fact_set_digest()),
    ];
    identity_parts.extend(
        affected_entities
            .iter()
            .map(|entity| format!("entity:{}", entity.evidence_identity().as_str())),
    );
    identity_parts.extend(fields.iter().map(|field| {
        format!(
            "field:{}:{}",
            field.source_row_identity(),
            field.field_path().terminal_projection_for_boundary()
        )
    }));
    WorthQueryDerivedMaintenanceEffect {
        effect: Arc::new(WorthQueryPerformedMaintenanceEffect::ProjectionPatch(
            WorthQueryPerformedProjectionPatch {
                identity: crate::identity::hash_parts(&identity_parts),
                fact_set_digest: facts.fact_set_digest().to_owned(),
                affected_entities,
                fields,
                prior_field_comparisons,
            },
        )),
        projection_commit,
        collection_commit,
    }
}

fn collection_impact(
    plan: &WorthQueryCoalescedMaintenancePlan,
) -> crate::domain_installation::WorthQueryImpactClass {
    use crate::domain_installation::WorthQueryImpactClass as Impact;
    if plan
        .strategies()
        .contains(&WorthQueryMaintenanceStrategy::WindowRefill)
    {
        Impact::WindowShift
    } else if plan
        .strategies()
        .contains(&WorthQueryMaintenanceStrategy::StableReorderOrRegroup)
    {
        Impact::ReorderOrRegroup
    } else if plan
        .strategies()
        .contains(&WorthQueryMaintenanceStrategy::MembershipSplice)
    {
        Impact::MembershipSplice
    } else {
        Impact::ValuePatch
    }
}

struct WorthQueryChangedFields {
    broad_projection_change: bool,
    broad_collection_change: bool,
    projection_targets: Vec<crate::live::WorthQueryProjectionChangeTarget>,
    native_targets: Vec<crate::domain_installation::WorthQueryCollectionChangedNativeTarget>,
}

fn changed_fields(impacts: &[WorthQueryAdmittedInvalidationImpact]) -> WorthQueryChangedFields {
    let mut broad_projection_change = false;
    let mut broad_collection_change = false;
    let mut projection_targets = Vec::new();
    let mut native_targets = Vec::new();
    for impact in impacts {
        for change in impact.truth().change_set().changes() {
            let Some(change) = change.semantic_change() else {
                broad_projection_change = true;
                broad_collection_change = true;
                continue;
            };
            use worth_runtime_bridge::facade::BridgeSemanticAspectChangeBreadth as Breadth;
            match change.effective_breadth() {
                Breadth::ExactField => {
                    let path = change
                        .effective_field_path()
                        .expect("exact field breadth retains its authoritative field path");
                    native_targets.push(
                        crate::domain_installation::WorthQueryCollectionChangedNativeTarget::new(
                            change.aspect_key().clone(),
                            Some(path.clone()),
                        ),
                    );
                    projection_targets.push(crate::live::WorthQueryProjectionChangeTarget::new(
                        change.aspect_key().clone(),
                        Some(path.clone()),
                    ));
                }
                Breadth::WholeAspect => {
                    native_targets.push(
                        crate::domain_installation::WorthQueryCollectionChangedNativeTarget::new(
                            change.aspect_key().clone(),
                            None,
                        ),
                    );
                    projection_targets.push(crate::live::WorthQueryProjectionChangeTarget::new(
                        change.aspect_key().clone(),
                        None,
                    ));
                }
                Breadth::Entity | Breadth::Surface => {
                    broad_projection_change = true;
                    broad_collection_change = true;
                }
            }
        }
    }
    projection_targets.sort();
    projection_targets.dedup();
    native_targets.sort();
    native_targets.dedup();
    WorthQueryChangedFields {
        broad_projection_change,
        broad_collection_change,
        projection_targets,
        native_targets,
    }
}
