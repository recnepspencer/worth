//! Iterative transport validation keeps finite recipe syntax acyclic and closed.
use super::denial::ApplicationDerivedRelationDefinitionDenial as Denial;
use super::expression::{ApplicationDerivedRelationExpression, ApplicationDerivedRelationNode};
use crate::application_schema::ApplicationGraphMemberSelector as Member;

impl ApplicationDerivedRelationExpression {
    /// Checks finite postorder syntax and returns the final endpoint kinds.
    pub fn validate(&self) -> Result<(Member<'_>, Member<'_>), Denial> {
        if self.nodes.is_empty() {
            return Err(Denial::EmptyExpression);
        }
        let mut endpoints: Vec<(Member<'_>, Member<'_>)> = Vec::with_capacity(self.nodes.len());
        for (index, node) in self.nodes.iter().enumerate() {
            let before = |input: usize| {
                (input < index)
                    .then(|| endpoints[input])
                    .ok_or(Denial::InvalidNodeReference)
            };
            let pair = match node {
                ApplicationDerivedRelationNode::Source(source) => {
                    (source.from.borrowed(), source.to.borrowed())
                }
                ApplicationDerivedRelationNode::Compose { left, right } => {
                    let left = before(*left)?;
                    let right = before(*right)?;
                    if left.1 != right.0 {
                        return Err(Denial::DiscontinuousEndpoints);
                    }
                    (left.0, right.1)
                }
                ApplicationDerivedRelationNode::Reverse { input } => {
                    let (from, to) = before(*input)?;
                    (to, from)
                }
                ApplicationDerivedRelationNode::WithoutSelfEdges { input } => {
                    let pair = before(*input)?;
                    if pair.0 != pair.1 {
                        return Err(Denial::HeterogeneousSelfExclusion);
                    }
                    pair
                }
                ApplicationDerivedRelationNode::CarrierLift {
                    domain, carrier, ..
                } => (Member::Entity(carrier), Member::Projected(domain)),
                ApplicationDerivedRelationNode::Union { left, right }
                | ApplicationDerivedRelationNode::Intersection { left, right } => {
                    let pair = before(*left)?;
                    if pair != before(*right)? {
                        return Err(Denial::DiscontinuousEndpoints);
                    }
                    pair
                }
            };
            endpoints.push(pair);
        }
        let mut reached = vec![false; self.nodes.len()];
        *reached.last_mut().expect("nonempty expression") = true;
        for (index, node) in self.nodes.iter().enumerate().rev() {
            if !reached[index] {
                return Err(Denial::UnusedExpressionNode);
            }
            match node {
                ApplicationDerivedRelationNode::Source(_)
                | ApplicationDerivedRelationNode::CarrierLift { .. } => {}
                ApplicationDerivedRelationNode::Compose { left, right }
                | ApplicationDerivedRelationNode::Union { left, right }
                | ApplicationDerivedRelationNode::Intersection { left, right } => {
                    reached[*left] = true;
                    reached[*right] = true;
                }
                ApplicationDerivedRelationNode::Reverse { input }
                | ApplicationDerivedRelationNode::WithoutSelfEdges { input } => {
                    reached[*input] = true;
                }
            }
        }
        Ok(*endpoints.last().expect("nonempty expression"))
    }
}
