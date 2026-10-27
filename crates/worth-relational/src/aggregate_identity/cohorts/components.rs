//! Complete affected prior components; proposed components use checked delta maintenance.
use super::*;
mod partition;
pub(super) use partition::maintain;

pub(super) fn groups(
    seeds: &BTreeSet<EntityId>,
    view: &InvariantStateView<'_>,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<BTreeSet<EntityId>>> {
    let mut visited = BTreeSet::new();
    let mut groups = Vec::new();
    for &seed in seeds {
        work.checkpoint(1)?;
        if visited.contains(&seed) {
            continue;
        }
        let Some(metadata) = view.entity_metadata(seed) else {
            continue;
        };
        work.acquired(Acquisition::Entity)?;
        if metadata.kind_id != contract.member_kind {
            continue;
        }
        let members = walk_native_breadth_first(
            &[seed],
            &mut CouplingRead {
                view,
                contract,
                work,
            },
        )?;
        for &member in &members {
            work.retain(extent::seed())?;
            visited.insert(member);
        }
        work.retain(extent::group())?;
        groups.push(members);
    }
    Ok(groups)
}

struct CouplingRead<'a, 'state, 'work, 'scope, 'authority, 'memory> {
    view: &'a InvariantStateView<'state>,
    contract: &'a AggregateIdentityContractDeclaration,
    work: &'a mut AggregateWork<'work, 'scope, 'authority, 'memory>,
}
impl BreadthFirstRead for CouplingRead<'_, '_, '_, '_, '_, '_> {
    type Error = worth_execution::MapKernelFailure<PlanDenial>;
    fn seed(&mut self, _: EntityId) -> BuildResult<()> {
        self.work.checkpoint(1)
    }
    fn retain(&mut self, _: EntityId, _: usize) -> BuildResult<()> {
        self.work.retain(extent::frontier_vertex())
    }
    fn expand(
        &mut self,
        entity: EntityId,
        _: usize,
        mut enqueue: impl FnMut(&mut Self, EntityId) -> BuildResult<()>,
    ) -> BuildResult<()> {
        for outgoing in [true, false] {
            for id in adjacency(self.view, entity, outgoing, self.work)? {
                self.work.checkpoint(1)?;
                let Some(edge) = self.view.relation_metadata(id) else {
                    continue;
                };
                self.work.acquired(Acquisition::Relation)?;
                if !self.contract.source.contains_relation(edge.kind_id) {
                    continue;
                }
                let neighbor = if outgoing { edge.target } else { edge.source };
                let Some(metadata) = self.view.entity_metadata(neighbor) else {
                    continue;
                };
                self.work.acquired(Acquisition::Entity)?;
                if metadata.kind_id != self.contract.member_kind {
                    return self.work.invalid(PlanDenial::InvalidComponentGraph);
                }
                enqueue(self, neighbor)?;
            }
        }
        Ok(())
    }
}
