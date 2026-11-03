//! One ownership algorithm shared by candidate and exact-basis observations.
use super::*;

pub(super) trait Read {
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
        read: &mut impl Read,
        subject: EntityId,
    ) -> Result<Option<EntityId>, RelationalModelOwnershipDenial> {
        if read.kind(subject)?.is_none() {
            return Err(RelationalModelOwnershipDenial::EntityUnavailable(subject));
        }
        let mut owner = None;
        for domain in &self.domains {
            for entity in walk(read, subject, false, &domain.relations)? {
                if read.kind(entity)? == Some(domain.root) {
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
        Ok(owner)
    }
}

pub(super) fn walk(
    read: &mut impl Read,
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
