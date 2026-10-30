//! Every named/reversed/Boolean occurrence contributes its inverse prefix.
use super::super::evaluation::{evaluate_position, insert_member, SourceRead};
use super::super::{
    DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle,
    DerivedRelationNode as Node, DerivedRelationReadDenial as Denial,
};
use super::lifecycle::{self, Lifecycle};
use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind};
use crate::graph_member::{NativeMemberDomain as Domain, NativeMemberKey as Member};
use crate::schema::data::RelationalSchemaRegistry;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct Position {
    definition: Handle,
    node: usize,
    reverse: bool,
}
impl Position {
    fn order(
        &self,
    ) -> (
        &crate::derived_relation::DerivedRelationIdentity,
        usize,
        bool,
    ) {
        (&self.definition.parts().identity, self.node, self.reverse)
    }
}
impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.order() == other.order()
    }
}
impl Eq for Position {}
impl PartialOrd for Position {
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Position {
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}
enum Frame {
    Enter(Position),
    Alias(Position, Position),
    Boolean(Position, Position, Position),
    Compose(Position, Position, Position),
}

pub(super) fn collect(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    reverse: bool,
    adjacency: &[AdjacencyDelta],
    events: &[Lifecycle],
    before: &mut impl SourceRead,
    after: &mut impl SourceRead,
) -> Result<(BTreeSet<Member>, Vec<super::super::DerivedRelationData>), Denial> {
    let root = Position {
        definition: definition.clone(),
        node: definition.parts().nodes.len() - 1,
        reverse,
    };
    let mut frames = Vec::new();
    push(&mut frames, Frame::Enter(root.clone()), after)?;
    let mut values = BTreeMap::<Position, BTreeSet<Member>>::new();
    let mut support = Vec::new();
    while let Some(frame) = frames.pop() {
        after.checkpoint(1)?;
        let (key, targets) = match frame {
            Frame::Enter(key) => {
                key_work(&key, values.len(), after)?;
                if values.contains_key(&key) {
                    continue;
                }
                match &key.definition.parts().nodes[key.node] {
                    Node::Stored { relation, .. } => {
                        let mut roots = BTreeSet::new();
                        for delta in adjacency {
                            after.checkpoint(1)?;
                            if delta.kind_id != *relation {
                                continue;
                            }
                            let (source, target) = match delta.kind {
                                AdjacencyDeltaKind::Created { source, target }
                                | AdjacencyDeltaKind::Deleted { source, target } => {
                                    (source, target)
                                }
                            };
                            insert_member(
                                after,
                                &mut roots,
                                if key.reverse {
                                    target.into()
                                } else {
                                    source.into()
                                },
                            )?;
                        }
                        (key, roots)
                    }
                    Node::CarrierLift { domain, case } => {
                        let mut roots = BTreeSet::new();
                        let pair = &key.definition.entry.endpoints[key.node];
                        lifecycle::seed_domain(
                            registry,
                            if key.reverse { &pair.1 } else { &pair.0 },
                            events,
                            &mut roots,
                            after,
                        )?;
                        if key.reverse {
                            let mut selected = BTreeSet::new();
                            for root in roots {
                                after.checkpoint(
                                    domain
                                        .name_bytes()
                                        .and_then(|n| n.checked_add(case.len()))
                                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                                )?;
                                if matches!(&root, Member::Projected(member) if member.domain == *domain && member.case == *case)
                                {
                                    insert_member(after, &mut selected, root)?;
                                }
                            }
                            roots = selected;
                        }
                        (key, roots)
                    }
                    Node::Derived(identity) => {
                        after.checkpoint(
                            identity
                                .name
                                .len()
                                .checked_add(identity.schema.0.len())
                                .and_then(|n| n.checked_mul(11))
                                .and_then(|n| {
                                    n.checked_mul(
                                        (usize::BITS
                                            - registry
                                                .derived_relations
                                                .len()
                                                .max(1)
                                                .leading_zeros())
                                            as usize,
                                    )
                                })
                                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                        )?;
                        let definition = registry
                            .derived_relation(identity)
                            .ok_or(DefinitionDenial::ForeignDefinition)?;
                        let source = Position {
                            node: definition.parts().nodes.len() - 1,
                            definition,
                            reverse: key.reverse,
                        };
                        push(&mut frames, Frame::Alias(key, source.clone()), after)?;
                        push(&mut frames, Frame::Enter(source), after)?;
                        continue;
                    }
                    Node::Reverse { input } | Node::WithoutSelfEdges { input } => {
                        let source = Position {
                            node: *input,
                            reverse: if matches!(
                                &key.definition.parts().nodes[key.node],
                                Node::Reverse { .. }
                            ) {
                                !key.reverse
                            } else {
                                key.reverse
                            },
                            ..key.clone()
                        };
                        push(&mut frames, Frame::Alias(key, source.clone()), after)?;
                        push(&mut frames, Frame::Enter(source), after)?;
                        continue;
                    }
                    Node::Compose { left, right }
                    | Node::Union { left, right }
                    | Node::Intersection { left, right } => {
                        let compose = matches!(
                            &key.definition.parts().nodes[key.node],
                            Node::Compose { .. }
                        );
                        let (first, second) = if compose && key.reverse {
                            (*right, *left)
                        } else {
                            (*left, *right)
                        };
                        let first = Position {
                            node: first,
                            ..key.clone()
                        };
                        let second = Position {
                            node: second,
                            ..key.clone()
                        };
                        let frame = if compose {
                            Frame::Compose(key, first.clone(), second.clone())
                        } else {
                            Frame::Boolean(key, first.clone(), second.clone())
                        };
                        push(&mut frames, frame, after)?;
                        push(&mut frames, Frame::Enter(second), after)?;
                        push(&mut frames, Frame::Enter(first), after)?;
                        continue;
                    }
                }
            }
            Frame::Alias(key, source) => {
                key_work(&source, values.len(), after)?;
                let mut roots = BTreeSet::new();
                for member in &values[&source] {
                    insert_member(after, &mut roots, member.clone())?;
                }
                (key, roots)
            }
            Frame::Boolean(key, left, right) | Frame::Compose(key, left, right) => {
                let compose = matches!(
                    &key.definition.parts().nodes[key.node],
                    Node::Compose { .. }
                );
                let mut roots = BTreeSet::new();
                key_work(&left, values.len(), after)?;
                for member in &values[&left] {
                    insert_member(after, &mut roots, member.clone())?;
                }
                key_work(&right, values.len(), after)?;
                for anchor in &values[&right] {
                    if compose {
                        for data in [
                            pullback(registry, &left, anchor, before)?,
                            pullback(registry, &left, anchor, after)?,
                        ]
                        .into_iter()
                        .flatten()
                        {
                            for member in data.endpoints() {
                                insert_member(after, &mut roots, member.clone())?;
                            }
                            after.retain(size_of::<super::super::DerivedRelationData>())?;
                            support.push(data);
                        }
                    } else {
                        insert_member(after, &mut roots, anchor.clone())?;
                    }
                }
                (key, roots)
            }
        };
        key_work(&key, values.len(), after)?;
        after.retain(size_of::<(Position, BTreeSet<Member>)>() + 4 * size_of::<usize>())?;
        values.insert(key, targets);
    }
    key_work(&root, values.len(), after)?;
    Ok((
        values.remove(&root).expect("complete affected root"),
        support,
    ))
}

fn push(frames: &mut Vec<Frame>, frame: Frame, read: &mut impl SourceRead) -> Result<(), Denial> {
    read.retain(size_of::<Frame>())?;
    frames.push(frame);
    Ok(())
}
fn key_work(key: &Position, count: usize, read: &mut impl SourceRead) -> Result<(), Denial> {
    let identity = &key.definition.parts().identity;
    let bytes = identity
        .schema
        .0
        .len()
        .checked_add(identity.name.len())
        .and_then(|n| n.checked_add(size_of::<Position>()))
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    let levels = (usize::BITS - count.max(1).leading_zeros()) as usize;
    read.checkpoint(
        bytes
            .checked_mul(11)
            .and_then(|n| n.checked_mul(levels))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
    )
}
fn pullback(
    registry: &RelationalSchemaRegistry,
    first: &Position,
    anchor: &Member,
    read: &mut impl SourceRead,
) -> Result<Option<super::super::DerivedRelationData>, Denial> {
    let pair = &first.definition.entry.endpoints[first.node];
    let input = if first.reverse { &pair.0 } else { &pair.1 };
    let expected = match input {
        Domain::Entity(kind) => *kind,
        Domain::Projected { projected } => {
            read.checkpoint(
                projected
                    .name_bytes()
                    .and_then(|n| n.checked_mul(11))
                    .and_then(|n| {
                        n.checked_mul(
                            (usize::BITS
                                - registry
                                    .projected_member_domains
                                    .len()
                                    .max(1)
                                    .leading_zeros()) as usize,
                        )
                    })
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
            )?;
            registry
                .projected_member_domain(projected)
                .ok_or(DefinitionDenial::InvalidMemberDomain)?
                .parts()
                .carrier
        }
    };
    if lifecycle::kind(read, anchor.carrier())? != Some(expected) {
        return Ok(None);
    }
    evaluate_position(
        registry,
        &first.definition,
        first.node,
        !first.reverse,
        anchor.clone(),
        read,
    )
    .map(Some)
}
