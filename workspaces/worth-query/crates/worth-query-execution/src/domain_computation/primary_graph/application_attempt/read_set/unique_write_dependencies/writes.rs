//! Present normalized assignments only; optional clears consume no unique value.
use super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationRealizedEffect as Effect,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::KindId;
type Write<'a> = (KindId, &'a AspectFieldLocator, &'a AspectValue);
pub(super) fn values(
    effects: &[Effect],
    kind_of: impl Fn(&str) -> Option<KindId>,
) -> Result<Vec<Write<'_>>, WorthQueryApplicationAttemptDenial> {
    let mut writes = Vec::new();
    for effect in effects {
        match effect {
            Effect::CreateEntity { kind, fields, .. } => {
                writes.extend(fields.iter().map(|(field, value)| (*kind, field, value)))
            }
            Effect::UpdateEntity { entity, fields, .. } => {
                let kind = kind_of(entity).ok_or_else(|| missing_kind(entity))?;
                writes.extend(fields.iter().map(|(field, value)| (kind, field, value)));
            }
            Effect::PatchOptionalEntityFields { entity, fields, .. } => {
                let kind = kind_of(entity).ok_or_else(|| missing_kind(entity))?;
                writes.extend(fields.iter().filter_map(|(field, write)| {
                    write.value.as_ref().map(|value| (kind, field, value))
                }));
            }
            Effect::DeleteEntity { .. }
            | Effect::CreateRelation { .. }
            | Effect::DeleteRelation { .. }
            | Effect::Emit(_) => {}
        }
    }
    Ok(writes)
}
fn missing_kind(entity: &str) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
        entity,
    )
}
