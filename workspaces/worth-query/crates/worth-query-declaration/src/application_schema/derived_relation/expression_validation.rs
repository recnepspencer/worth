//! Iterative transport validation keeps finite recipe syntax acyclic and closed.
use super::denial::ApplicationDerivedRelationDefinitionDenial as Denial;
use super::expression::{ApplicationDerivedRelationExpression, ApplicationDerivedRelationNode};

impl ApplicationDerivedRelationExpression {
    /// Checks finite postorder syntax and returns the final endpoint kinds.
    pub fn validate(&self) -> Result<(&str, &str), Denial> {
        if self.nodes.is_empty() {
            return Err(Denial::EmptyExpression);
        }
        let mut endpoints: Vec<(&str, &str)> = Vec::with_capacity(self.nodes.len());
        for (index, node) in self.nodes.iter().enumerate() {
            let before = |input: usize| {
                (input < index)
                    .then(|| endpoints[input])
                    .ok_or(Denial::InvalidNodeReference)
            };
            let pair = match node {
                ApplicationDerivedRelationNode::Source(source) => {
                    (source.from.as_str(), source.to.as_str())
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
                ApplicationDerivedRelationNode::Source(_) => {}
                ApplicationDerivedRelationNode::Compose { left, right } => {
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
