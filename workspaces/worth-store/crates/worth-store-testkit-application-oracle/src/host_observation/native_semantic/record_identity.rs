use super::*;
pub(crate) fn correspond_receipt(
    map: &mut DurableIdentityCorrespondence,
    h: &mut RelationalCommitReceipt,
    r: &RelationalCommitReceipt,
) -> Result<(), HostDivergence> {
    if h.parents.len() != r.parents.len() {
        return Err(bad());
    }
    map.admit(Id::NativeCommit(h.commit_id), Id::NativeCommit(r.commit_id))?;
    map.admit(
        Id::NativeVersion(h.version_id),
        Id::NativeVersion(r.version_id),
    )?;
    map.admit(
        Id::NativeBranch(h.branch_id.clone()),
        Id::NativeBranch(r.branch_id.clone()),
    )?;
    h.commit_id = r.commit_id;
    h.version_id = r.version_id;
    h.branch_id = r.branch_id.clone();
    for (h, r) in h.parents.iter_mut().zip(&r.parents) {
        map.admit(Id::NativeCommit(*h), Id::NativeCommit(*r))?;
        *h = *r;
    }
    Ok(())
}
pub(crate) fn correspond_record(
    map: &mut DurableIdentityCorrespondence,
    h: &mut RecordRef,
    r: &RecordRef,
) -> Result<(), HostDivergence> {
    match (&*h, r) {
        (RecordRef::Entity(h), RecordRef::Entity(r)) => {
            map.admit(Id::Entity(*h), Id::Entity(*r))?
        }
        (RecordRef::Relation(h), RecordRef::Relation(r)) => {
            map.admit(Id::Relation(*h), Id::Relation(*r))?
        }
        (RecordRef::Entity(_), RecordRef::Relation(_))
        | (RecordRef::Relation(_), RecordRef::Entity(_)) => return Err(bad()),
    }
    *h = r.clone();
    Ok(())
}
pub(super) fn correspond_endpoint(
    map: &mut DurableIdentityCorrespondence,
    h: &mut RelationalCreatedEndpoint,
    r: &RelationalCreatedEndpoint,
) -> Result<(), HostDivergence> {
    match (&*h, r) {
        (RelationalCreatedEndpoint::Existing(h), RelationalCreatedEndpoint::Existing(r)) => {
            map.admit(Id::Entity(*h), Id::Entity(*r))?
        }
        (RelationalCreatedEndpoint::Created(h), RelationalCreatedEndpoint::Created(r)) => {
            map.admit(
                Id::NativePartition(h.partition),
                Id::NativePartition(r.partition),
            )?;
            if h.kind != r.kind || h.client_key != r.client_key {
                return Err(bad());
            }
        }
        (RelationalCreatedEndpoint::Existing(_), RelationalCreatedEndpoint::Created(_))
        | (RelationalCreatedEndpoint::Created(_), RelationalCreatedEndpoint::Existing(_)) => {
            return Err(bad())
        }
    }
    *h = r.clone();
    Ok(())
}
