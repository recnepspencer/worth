//! Validation versions and touched identities retain the same meaning as graph IDs.
use super::*;
use worth_relational::facade::identity::{EntityId, RelationId, VersionId};
use worth_relational::facade::{runtime::InvariantVerdict, transactions::InvariantViolationFields};
mod violation;

pub(super) fn version(
    map: &mut DurableIdentityCorrespondence,
    h: &mut VersionId,
    r: VersionId,
) -> Result<(), HostDivergence> {
    map.admit(Id::NativeVersion(*h), Id::NativeVersion(r))?;
    *h = r;
    Ok(())
}
pub(super) fn entity(
    map: &mut DurableIdentityCorrespondence,
    h: &mut EntityId,
    r: EntityId,
) -> Result<(), HostDivergence> {
    map.admit(Id::Entity(*h), Id::Entity(r))?;
    *h = r;
    Ok(())
}
pub(super) fn relation(
    map: &mut DurableIdentityCorrespondence,
    h: &mut RelationId,
    r: RelationId,
) -> Result<(), HostDivergence> {
    map.admit(Id::Relation(*h), Id::Relation(r))?;
    *h = r;
    Ok(())
}
fn fields(v: &mut InvariantVerdict) -> Option<&mut InvariantViolationFields> {
    if let InvariantVerdict::Violation(v) | InvariantVerdict::Advisory { violation: v, .. } = v {
        Some(&mut v.fields)
    } else {
        None
    }
}
pub(super) fn correspond(
    map: &mut DurableIdentityCorrespondence,
    home: &mut [RelationalInvariantExecutionMeaning],
    reference: &[RelationalInvariantExecutionMeaning],
) -> Result<(), HostDivergence> {
    if home.len() != reference.len() {
        return Err(bad());
    }
    for (h, r) in home.iter_mut().zip(reference) {
        version(map, &mut h.metadata.version, r.metadata.version)?;
        version(
            map,
            &mut h.metadata.current_version,
            r.metadata.current_version,
        )?;
        if h.checks.len() != r.checks.len() || h.decisions.len() != r.decisions.len() {
            return Err(bad());
        }
        for (h, r) in h.checks.iter_mut().zip(&r.checks) {
            match (&mut h.custom_provenance, &r.custom_provenance) {
                (None, None) => (),
                (Some(h), Some(r)) => {
                    version(map, &mut h.version, r.version)?;
                    version(map, &mut h.current_version, r.current_version)?;
                    if h.touched.visible_entity_ids.len() != r.touched.visible_entity_ids.len()
                        || h.touched.visible_relation_ids.len()
                            != r.touched.visible_relation_ids.len()
                    {
                        return Err(bad());
                    }
                    for (h, r) in h
                        .touched
                        .visible_entity_ids
                        .iter()
                        .zip(r.touched.visible_entity_ids.iter())
                    {
                        map.admit(Id::Entity(*h), Id::Entity(*r))?;
                    }
                    for (h, r) in h
                        .touched
                        .visible_relation_ids
                        .iter()
                        .zip(r.touched.visible_relation_ids.iter())
                    {
                        map.admit(Id::Relation(*h), Id::Relation(*r))?;
                    }
                    if h.touched.touched_partition_ids.len()
                        != r.touched.touched_partition_ids.len()
                    {
                        return Err(bad());
                    }
                    for (h, r) in h
                        .touched
                        .touched_partition_ids
                        .iter()
                        .zip(r.touched.touched_partition_ids.iter())
                    {
                        map.admit(Id::NativePartition(*h), Id::NativePartition(*r))?;
                    }
                    h.touched.visible_entity_ids = r.touched.visible_entity_ids.clone();
                    h.touched.visible_relation_ids = r.touched.visible_relation_ids.clone();
                    h.touched.touched_partition_ids = r.touched.touched_partition_ids.clone();
                }
                (None, Some(_)) | (Some(_), None) => return Err(bad()),
            }
            let mut r_verdict = r.verdict.clone();
            match (fields(&mut h.verdict), fields(&mut r_verdict)) {
                (None, None) => (),
                (Some(h), Some(r)) => violation::correspond(map, h, r)?,
                (None, Some(_)) | (Some(_), None) => return Err(bad()),
            }
        }
    }
    Ok(())
}
