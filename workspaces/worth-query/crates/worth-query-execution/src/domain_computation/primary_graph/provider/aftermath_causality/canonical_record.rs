//! Descriptive canonical facts survive optional source availability.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
};
use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCanonicalCorrectionFact {
    pub role: WorthQueryAftermathCausalRole,
    pub parent: RelationalCommitReceipt,
    pub child: RelationalCommitReceipt,
    pub record: RecordRef,
    pub source: WorthQueryCorrectionSourceCoordinates,
    pub root: WorthQueryCorrectionSourceCoordinates,
    pub selected: Option<WorthQueryCorrectionSourceCoordinates>,
    pub active: Option<WorthQueryCorrectionSourceCoordinates>,
    pub prior: Option<WorthQueryCorrectionSourceCoordinates>,
}

pub(super) fn read(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryAftermathCausalityLayout,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: worth_relational::facade::identity::EntityId,
) -> Result<WorthQueryCanonicalCorrectionFact, WorthQueryAftermathCausalityReadDenial> {
    let fields = required_fields(layout)?;
    let aspect = layout.key_locator.aspect().aspect_key().clone();
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        aspect.clone(),
        fields.clone(),
    )]);
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or(WorthQueryAftermathCausalityReadDenial::Unavailable)?;
    let row = view
        .entity_record_with_projection_scope(execution, entity, scope, |record| {
            (record.kind_id() == layout.entity_kind
                && record.lifecycle() == RecordLifecycleState::Live)
                .then(|| {
                    (
                        record.created_at_version(),
                        fields
                            .iter()
                            .map(|field| record.aspect_field_value(&aspect, field).cloned())
                            .collect::<Vec<_>>(),
                    )
                })
        })
        .map_err(aftermath_read_stop)?
        .ok_or(WorthQueryAftermathCausalityReadDenial::Unavailable)?;
    decode(runtime, entity, row)
}

fn decode(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity: worth_relational::facade::identity::EntityId,
    (version, values): AftermathCausalityRecord,
) -> Result<WorthQueryCanonicalCorrectionFact, WorthQueryAftermathCausalityReadDenial> {
    use WorthQueryAftermathCausalityReadDenial::Unavailable;
    let role = match number(&values, 1)? {
        3 => WorthQueryAftermathCausalRole::RecordedReverse,
        4 => WorthQueryAftermathCausalRole::RecordedReapply,
        _ => return Err(Unavailable),
    };
    let child = runtime
        .history()
        .historical_committed_version(version)
        .ok_or(Unavailable)?
        .commit()
        .clone();
    let parent = runtime
        .history()
        .immutable_commit_receipt(CommitId(number(&values, 3)?))
        .ok_or(Unavailable)?;
    if parent.branch_id.0 != string(&values, 2)? || child.parents.as_slice() != [parent.commit_id] {
        return Err(Unavailable);
    }
    let source = coordinates(runtime, &values, 5)?.ok_or(Unavailable)?;
    let root = coordinates(runtime, &values, 9)?.ok_or(Unavailable)?;
    let selected = coordinates(runtime, &values, 13)?;
    let active = coordinates(runtime, &values, 17)?;
    let prior = coordinates(runtime, &values, 21)?;
    let shape = match role {
        WorthQueryAftermathCausalRole::RecordedReverse => {
            selected.is_none()
                && active.is_none()
                && values[25].as_ref() == Some(&lineage_fields::selector(&root))
                && values[26].as_ref() == Some(&lineage_fields::selector(&source))
                && values[27].is_none()
        }
        WorthQueryAftermathCausalRole::RecordedReapply => {
            selected.is_some()
                && active.is_some()
                && prior == active
                && values[25].is_none()
                && values[26].is_none()
                && values[27].as_ref()
                    == active
                        .as_ref()
                        .map(|active| lineage_fields::consumption_selector(active.relational()))
                        .as_ref()
        }
        _ => false,
    };
    if !shape {
        return Err(Unavailable);
    }
    Ok(WorthQueryCanonicalCorrectionFact {
        role,
        parent,
        child,
        record: RecordRef::Entity(entity),
        source,
        root,
        selected,
        active,
        prior,
    })
}

fn coordinates(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    values: &AftermathCausalityValues,
    start: usize,
) -> Result<Option<WorthQueryCorrectionSourceCoordinates>, WorthQueryAftermathCausalityReadDenial> {
    use WorthQueryAftermathCausalityReadDenial::Unavailable;
    if values[start..start + 4].iter().all(Option::is_none) {
        return Ok(None);
    }
    let relational = runtime
        .history()
        .immutable_commit_receipt(CommitId(number(values, start + 3)?))
        .ok_or(Unavailable)?;
    if relational.branch_id.0 != string(values, start + 2)? {
        return Err(Unavailable);
    }
    Ok(Some(
        WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
            number(values, start)?,
            number(values, start + 1)?,
            relational,
        ),
    ))
}
fn number(
    values: &AftermathCausalityValues,
    index: usize,
) -> Result<u64, WorthQueryAftermathCausalityReadDenial> {
    values
        .get(index)
        .and_then(Option::as_ref)
        .and_then(as_u64)
        .ok_or(WorthQueryAftermathCausalityReadDenial::Unavailable)
}
fn string(
    values: &AftermathCausalityValues,
    index: usize,
) -> Result<String, WorthQueryAftermathCausalityReadDenial> {
    match values.get(index).and_then(Option::as_ref) {
        Some(AspectValue::String(InternedString::Raw(value))) => Ok(value.clone()),
        _ => Err(WorthQueryAftermathCausalityReadDenial::Unavailable),
    }
}
