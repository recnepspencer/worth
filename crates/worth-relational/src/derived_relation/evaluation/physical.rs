//! Actual carrier/lift and physical adjacency acquisition for the one evaluator.
use super::memo::Key;
use super::{insert_member, Adjacency, DefinitionDenial, Denial, Evaluation, Member, SourceRead};
use crate::graph_member::{NativeMemberDomain as Domain, ProjectedMemberCoordinate};
use crate::identity::data::{EntityId, KindId};
use crate::projected_member_domain::ProjectedMemberDomainIdentity;
use crate::validation::data::StructuralRelationRecord;
use std::collections::BTreeSet;
use std::sync::Arc;

impl<Port: SourceRead> Evaluation<'_, Port> {
    pub(super) fn member(&mut self, member: &Member, expected: &Domain) -> Result<(), Denial> {
        super::member_work(self.read, member, 1)?;
        match (member, expected) {
            (Member::Entity(entity), Domain::Entity(kind)) => self.entity(*entity, *kind),
            (Member::Projected(member), Domain::Projected { projected })
                if member.domain == *projected =>
            {
                let domain = self.domain(projected)?;
                self.case_work(&member.case, domain.parts().cases.len())?;
                if domain.case(&member.case).is_none() {
                    return Err(DefinitionDenial::InvalidMemberCase.into());
                }
                self.entity(member.carrier, domain.parts().carrier)
            }
            _ => Err(DefinitionDenial::DiscontinuousEndpoints.into()),
        }
    }
    fn domain(
        &mut self,
        identity: &ProjectedMemberDomainIdentity,
    ) -> Result<crate::projected_member_domain::ProjectedMemberDomainHandle, Denial> {
        let names = identity
            .name_bytes()
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        self.semantic_work(names, self.registry.projected_member_domains.len())?;
        self.registry
            .projected_member_domain(identity)
            .ok_or(DefinitionDenial::InvalidMemberDomain.into())
    }
    fn semantic_work(&mut self, names: usize, count: usize) -> Result<(), Denial> {
        let levels = (usize::BITS - count.max(1).leading_zeros()) as usize;
        self.read.checkpoint(
            names
                .checked_mul(11)
                .and_then(|n| n.checked_mul(levels))
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )
    }
    fn case_work(&mut self, name: &str, count: usize) -> Result<(), Denial> {
        self.semantic_work(name.len(), count)
    }

    pub(super) fn carrier_lift(
        &mut self,
        key: Key,
        identity: &ProjectedMemberDomainIdentity,
        case: &str,
    ) -> Result<(), Denial> {
        let domain = self.domain(identity)?;
        self.case_work(case, domain.parts().cases.len())?;
        if domain.case(case).is_none() {
            return Err(DefinitionDenial::InvalidMemberCase.into());
        }
        let mut targets = BTreeSet::new();
        if key.reverse {
            super::member_work(self.read, &key.anchor, 1)?;
            let Member::Projected(member) = &key.anchor else {
                return Err(DefinitionDenial::DiscontinuousEndpoints.into());
            };
            if member.domain != *identity {
                return Err(DefinitionDenial::DiscontinuousEndpoints.into());
            }
            self.entity(member.carrier, domain.parts().carrier)?;
            self.case_work(case, 1)?;
            if member.case == case {
                insert_member(self.read, &mut targets, Member::Entity(member.carrier))?;
            }
        } else {
            self.member(&key.anchor, &Domain::Entity(domain.parts().carrier))?;
            let carrier = key.anchor.entity().expect("admitted physical carrier");
            let bytes = identity
                .name_bytes()
                .and_then(|n| n.checked_add(case.len()))
                .and_then(|n| {
                    n.checked_add(size_of::<ProjectedMemberCoordinate>() + 2 * size_of::<usize>())
                })
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
            self.read.retain(bytes)?;
            let member = Member::Projected(Arc::new(ProjectedMemberCoordinate {
                domain: identity.clone(),
                carrier,
                case: case.to_owned(),
            }));
            insert_member(self.read, &mut targets, member)?;
        }
        self.save(key, targets)
    }
    fn entity(&mut self, id: EntityId, expected: KindId) -> Result<(), Denial> {
        self.read.checkpoint(1)?;
        let kind = self.read.entity_kind(id)?;
        if kind != expected {
            return Err(DefinitionDenial::DiscontinuousEndpoints.into());
        }
        if !self.entities.contains_key(&id) {
            self.read
                .retain(size_of::<(EntityId, KindId)>() + 4 * size_of::<usize>())?;
            self.entities.insert(id, kind);
        }
        Ok(())
    }
    pub(super) fn stored(
        &mut self,
        key: Key,
        relation: KindId,
        from: KindId,
        to: KindId,
    ) -> Result<(), Denial> {
        self.member(
            &key.anchor,
            &Domain::Entity(if key.reverse { to } else { from }),
        )?;
        let anchor = key.anchor.entity().expect("admitted physical endpoint");
        let probe = Adjacency {
            anchor,
            relation,
            outgoing: !key.reverse,
        };
        if !self.adjacency.contains(&probe) {
            self.read
                .retain(size_of::<Adjacency>() + 4 * size_of::<usize>())?;
            self.adjacency.insert(probe);
        }
        let mut targets = BTreeSet::new();
        for record in self.read.adjacency(probe)? {
            self.read.checkpoint(1)?;
            let (actual_anchor, target) = if key.reverse {
                (record.target, record.source)
            } else {
                (record.source, record.target)
            };
            if record.kind_id != relation || actual_anchor != anchor {
                return Err(DefinitionDenial::InvalidPhysicalSource(relation).into());
            }
            self.entity(record.source, from)?;
            self.entity(record.target, to)?;
            if !self.relations.contains_key(&record.relation_id) {
                self.read
                    .retain(size_of::<StructuralRelationRecord>() + 4 * size_of::<usize>())?;
                self.relations.insert(record.relation_id, record);
            }
            insert_member(self.read, &mut targets, target.into())?;
        }
        self.save(key, targets)
    }
}
