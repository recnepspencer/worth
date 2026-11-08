//! One ownership algorithm shared by candidate and exact-basis observations.
use super::*;

pub(super) trait Read {
    fn charge_work(&mut self, units: u64) -> Result<(), RelationalModelOwnershipDenial>;
    fn kind(&mut self, entity: EntityId) -> Result<Option<KindId>, RelationalModelOwnershipDenial>;
    fn adjacent(
        &mut self,
        entity: EntityId,
        outgoing: bool,
        relations: &[KindId],
    ) -> Result<Vec<EntityId>, RelationalModelOwnershipDenial>;
    fn retain(&mut self) -> Result<(), RelationalModelOwnershipDenial>;
}

impl RelationalModelOwnership {
    /// O(D * (V+A) log V) over declared ancestors, including raw adjacency A.
    /// Several paths to one root are one owner. A missing subject is not unowned.
    pub(super) fn owner(
        &self,
        read: &mut (impl Read + ?Sized),
        subject: EntityId,
    ) -> Result<Option<EntityId>, RelationalModelOwnershipDenial> {
        self.owner_with_boundary(read, subject, None)
            .map(|(owner, _)| owner)
    }
    pub(super) fn owner_with_boundary(
        &self,
        read: &mut (impl Read + ?Sized),
        subject: EntityId,
        mut boundary: Option<(u64, &mut dyn FnMut(EntityId, KindId) -> bool)>,
    ) -> Result<(Option<EntityId>, bool), RelationalModelOwnershipDenial> {
        if read.kind(subject)?.is_none() {
            return Err(RelationalModelOwnershipDenial::EntityUnavailable(subject));
        }
        let mut owner = None;
        let mut within = false;
        for domain in &self.domains {
            for entity in walk(read, subject, false, &domain.relations)? {
                let kind = read.kind(entity)?;
                if let (Some((work, contains)), Some(kind)) = (boundary.as_mut(), kind) {
                    read.charge_work(*work)?;
                    within |= contains(entity, kind);
                }
                if kind == Some(domain.root) {
                    if let Some(first_owner) = owner {
                        if first_owner != entity {
                            return Err(RelationalModelOwnershipDenial::Conflict(
                                RelationalModelOwnershipConflict { subject },
                            ));
                        }
                    } else {
                        owner = Some(entity);
                    }
                }
            }
        }
        Ok((owner, within))
    }
}

pub(super) fn walk(
    read: &mut (impl Read + ?Sized),
    subject: EntityId,
    outgoing: bool,
    relations: &[KindId],
) -> Result<BTreeSet<EntityId>, RelationalModelOwnershipDenial> {
    let mut visited = BTreeSet::new();
    let mut queue = Vec::new();
    read.retain()?;
    visited.insert(subject);
    queue.push(subject);
    let mut next = 0;
    while next < queue.len() {
        let entity = queue[next];
        next += 1;
        for neighbor in read.adjacent(entity, outgoing, relations)? {
            if !visited.contains(&neighbor) && read.kind(neighbor)?.is_some() {
                read.retain()?;
                visited.insert(neighbor);
                queue.push(neighbor);
            }
        }
    }
    Ok(visited)
}
