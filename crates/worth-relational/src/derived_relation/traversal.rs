//! The existing Native frontier driver over genuine derived endpoint sources.
use super::evaluation::{evaluate_position, member_work, SourceRead};
use super::{
    DerivedRelationData as Data, DerivedRelationDenial as DefinitionDenial,
    DerivedRelationHandle as Handle, DerivedRelationReadDenial as Denial,
};
use crate::graph_member::{NativeMemberDomain as Domain, NativeMemberKey as Member};
use crate::schema::data::RelationalSchemaRegistry;
use crate::structural_traversal::{walk as frontier, BreadthFirstRead};
use std::collections::BTreeSet;
mod complete;
pub(in crate::derived_relation) use complete::acquire;

pub(crate) struct DerivedTraversal {
    pub(crate) members: BTreeSet<Member>,
    pub(crate) support: Vec<DerivedTraversalStep>,
    pub(crate) steps: Vec<super::DerivedRelationWalkStep>,
}

pub(crate) struct DerivedTraversalStep {
    pub(crate) anchor: Member,
    pub(crate) reverse: bool,
    pub(crate) data: Data,
}

pub(crate) fn walk(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    seeds: &[Member],
    reverse: bool,
    read: &mut impl SourceRead,
) -> Result<DerivedTraversal, Denial> {
    run(registry, definition, seeds, reverse, false, false, read)
}

pub(crate) fn walk_undirected(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    seeds: &[Member],
    read: &mut impl SourceRead,
) -> Result<DerivedTraversal, Denial> {
    run(registry, definition, seeds, false, true, false, read)
}

pub(in crate::derived_relation) fn observed_walk(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    seed: Member,
    reverse: bool,
    undirected: bool,
    read: &mut impl SourceRead,
) -> Result<DerivedTraversal, Denial> {
    let input = if reverse {
        &definition.parts().to
    } else {
        &definition.parts().from
    };
    if !available(registry, input, &seed, read)? {
        return Err(DefinitionDenial::RecordUnavailable.into());
    }
    run(
        registry,
        definition,
        &[seed],
        reverse,
        undirected,
        true,
        read,
    )
}

fn run(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    seeds: &[Member],
    reverse: bool,
    undirected: bool,
    record_steps: bool,
    read: &mut impl SourceRead,
) -> Result<DerivedTraversal, Denial> {
    let mut policy = DerivedFrontier {
        registry,
        definition,
        reverse,
        undirected,
        read,
        support: Vec::new(),
        steps: Vec::new(),
        record_steps,
    };
    let members = frontier(seeds, &mut policy)?;
    Ok(DerivedTraversal {
        members,
        support: policy.support,
        steps: policy.steps,
    })
}

struct DerivedFrontier<'a, Port> {
    registry: &'a RelationalSchemaRegistry,
    definition: &'a Handle,
    reverse: bool,
    undirected: bool,
    read: &'a mut Port,
    support: Vec<DerivedTraversalStep>,
    steps: Vec<super::DerivedRelationWalkStep>,
    record_steps: bool,
}
impl<Port: SourceRead> BreadthFirstRead<Member> for DerivedFrontier<'_, Port> {
    type Error = Denial;
    fn seed(&mut self, _: Member) -> Result<(), Denial> {
        self.read.checkpoint(1)
    }
    fn retain(&mut self, member: Member, distance: usize) -> Result<(), Denial> {
        self.read
            .retain(3 * size_of::<Member>() + 5 * size_of::<usize>())?;
        if self.record_steps {
            self.read
                .retain(4 * size_of::<super::DerivedRelationWalkStep>())?;
            self.steps
                .push(super::DerivedRelationWalkStep { member, distance });
        }
        Ok(())
    }
    fn membership_lookup(&mut self, member: &Member, count: usize) -> Result<(), Denial> {
        member_work(self.read, member, count)
    }
    fn expand(
        &mut self,
        member: Member,
        _: usize,
        mut enqueue: impl FnMut(&mut Self, Member) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        for reverse in [self.reverse, !self.reverse]
            .into_iter()
            .take(if self.undirected { 2 } else { 1 })
        {
            let input = if reverse {
                &self.definition.parts().to
            } else {
                &self.definition.parts().from
            };
            if !available(self.registry, input, &member, self.read)? {
                continue;
            }
            let data = evaluate_position(
                self.registry,
                self.definition,
                self.definition.parts().nodes.len() - 1,
                reverse,
                member.clone(),
                self.read,
            )?;
            for endpoint in data.endpoints() {
                self.read.checkpoint(1)?;
                enqueue(self, endpoint.clone())?;
            }
            self.read.retain(4 * size_of::<DerivedTraversalStep>())?;
            self.support.push(DerivedTraversalStep {
                anchor: member.clone(),
                reverse,
                data,
            });
        }
        Ok(())
    }
}

/// A heterogeneous relation has no successors outside its declared input
/// domain. Missing retired generations are absent; original stops still win.
pub(crate) fn available(
    registry: &RelationalSchemaRegistry,
    domain: &Domain,
    member: &Member,
    read: &mut impl SourceRead,
) -> Result<bool, Denial> {
    member_work(read, member, 1)?;
    let expected = match (domain, member) {
        (Domain::Entity(kind), Member::Entity(_)) => *kind,
        (Domain::Projected { projected }, Member::Projected(member)) => {
            let names = projected
                .name_bytes()
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
            let levels = (usize::BITS
                - registry
                    .projected_member_domains
                    .len()
                    .max(1)
                    .leading_zeros()) as usize;
            read.checkpoint(
                names
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(levels))
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
            )?;
            if member.domain != *projected {
                return Ok(false);
            }
            let domain = registry
                .projected_member_domain(projected)
                .ok_or(DefinitionDenial::InvalidMemberDomain)?;
            let case_levels =
                (usize::BITS - domain.parts().cases.len().max(1).leading_zeros()) as usize;
            read.checkpoint(
                member
                    .case
                    .len()
                    .checked_mul(case_levels)
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
            )?;
            if domain.case(&member.case).is_none() {
                return Err(DefinitionDenial::InvalidMemberCase.into());
            }
            domain.parts().carrier
        }
        _ => return Ok(false),
    };
    match read.entity_kind(member.carrier()) {
        Ok(observed) => Ok(observed == expected),
        Err(Denial::Definition(DefinitionDenial::RecordUnavailable)) => Ok(false),
        Err(cause) => Err(cause),
    }
}
