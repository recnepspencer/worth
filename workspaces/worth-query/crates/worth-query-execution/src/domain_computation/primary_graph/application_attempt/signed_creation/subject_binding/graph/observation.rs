//! A missing value is certified absence only when the native field observation says so.
use super::{identity, BodyCandidate, CaptureSpec, Denial, Presence};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationObservedFact as Fact, WorthQueryApplicationRealizedEffect as Effect,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationSchema, ApplicationSigningSubjectObservedFieldVersion as Version,
    },
};
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalFieldPresence,
    storage::AuthoritativeFieldComparisonKey as Comparison,
};

pub(super) fn capture<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    candidate: &BodyCandidate<Schema, Binding>,
    source: EntityId,
    spec: &CaptureSpec<'_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Version, Denial> {
    let layout = &candidate.read_set.lease.layout;
    let locator = layout
        .field_locator(
            spec.source.entity(),
            spec.source.aspect(),
            spec.source.field(),
        )
        .ok_or(Denial::CandidateMismatch)?;
    let mut value = None;
    let mut revision = None;
    for fact in candidate.read_set.facts.iter() {
        identity::admit_assembly(context, 0)?;
        match fact {
            Fact::Field {
                entity_id,
                locator: observed,
                value: raw,
                ..
            } if *entity_id == source && same_field(observed, locator) => value = Some(Some(raw)),
            Fact::AbsentField {
                entity_id,
                locator: observed,
                ..
            } if *entity_id == source && same_field(observed, locator) => value = Some(None),
            Fact::SourceFieldRevision {
                entity_id,
                locator: observed,
                native_revision,
            } if *entity_id == source && same_field(observed, locator) => {
                revision = Some(*native_revision)
            }
            _ => {}
        }
    }
    let value = value.ok_or(Denial::CandidateMismatch)?;
    let revision = revision.ok_or(Denial::CandidateMismatch)?;
    let observed = match (value, revision) {
        (Some(_), Some(revision)) if revision.presence() == RelationalFieldPresence::Present => {
            Version::Present {
                native_revision: revision.version().0,
            }
        }
        (None, revision)
            if spec.source_presence == Presence::Optional
                && spec.target_presence == Presence::Optional
                && revision.is_none_or(|r| r.presence() == RelationalFieldPresence::Absent) =>
        {
            Version::Absent {
                native_revision: revision.map(|r| r.version().0),
            }
        }
        _ => return Err(Denial::CandidateMismatch),
    };
    let slot = candidate
        .read_set
        .signed_creation
        .as_ref()
        .ok_or(Denial::CandidateMismatch)?
        .slot
        .effect;
    let Some(Effect::CreateEntity { fields, .. }) = candidate.effects.get(slot) else {
        return Err(Denial::CandidateMismatch);
    };
    let target = layout
        .field_locator(
            spec.target.entity(),
            spec.target.aspect(),
            spec.target.field(),
        )
        .ok_or(Denial::CandidateMismatch)?;
    let captured = fields.get(target);
    match (value, captured) {
        (Some(value), Some(captured)) => {
            let bytes = Comparison::required_encoded_capacity_bytes(value)
                .and_then(|bytes| {
                    bytes.checked_add(Comparison::required_encoded_capacity_bytes(captured)?)
                })
                .ok_or(Denial::RepresentationCapacity)?;
            identity::admit_assembly(
                context,
                usize::try_from(bytes).map_err(|_| Denial::RepresentationCapacity)?,
            )?;
            if Comparison::from_aspect_value(value) != Comparison::from_aspect_value(captured) {
                return Err(Denial::CandidateMismatch);
            }
        }
        (None, None) => {}
        _ => return Err(Denial::CandidateMismatch),
    }
    // Paths may return to the same native identity. Reject an in-candidate subject
    // write here even when static entity kinds alone could not prove aliasing.
    for effect in &candidate.effects {
        identity::admit_assembly(context, 0)?;
        let touched = match effect {
            Effect::UpdateEntity {
                entity_id, fields, ..
            } => *entity_id == source && fields.keys().any(|f| same_field(f, locator)),
            Effect::PatchOptionalEntityFields {
                entity_id, fields, ..
            } => *entity_id == source && fields.keys().any(|f| same_field(f, locator)),
            Effect::DeleteEntity { entity_id } => *entity_id == source,
            _ => false,
        };
        if touched {
            return Err(Denial::CandidateMismatch);
        }
    }
    Ok(observed)
}
fn same_field(
    left: &worth_foundational::facade::AspectFieldLocator,
    right: &worth_foundational::facade::AspectFieldLocator,
) -> bool {
    left.aspect().aspect_key() == right.aspect().aspect_key()
        && left.field_path() == right.field_path()
}
