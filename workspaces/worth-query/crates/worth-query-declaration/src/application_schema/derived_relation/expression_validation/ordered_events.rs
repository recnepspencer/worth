//! Literal physical event roles, independent of current graph observations.
use super::{ApplicationDerivedRelationNode as Node, Denial, Member};
use crate::application_schema::ApplicationDerivedRelationSourceKind as SourceKind;

pub(super) fn endpoints<'a>(
    nodes: &'a [Node],
    position: usize,
    roles: [usize; 3],
) -> Result<(Member<'a>, Member<'a>), Denial> {
    let source = |index: usize| {
        if index >= position {
            return Err(Denial::InvalidNodeReference);
        }
        let Node::Source(source) = &nodes[index] else {
            return Err(Denial::InvalidEventOperand);
        };
        if source.kind != SourceKind::Stored
            || source.from.entity().is_none()
            || source.to.entity().is_none()
        {
            return Err(Denial::InvalidEventOperand);
        }
        Ok((source.from.borrowed(), source.to.borrowed()))
    };
    let predecessor = source(roles[0])?;
    let introduced = source(roles[1])?;
    let retired = source(roles[2])?;
    if predecessor.0 != predecessor.1 || predecessor.0 != introduced.0 || introduced != retired {
        return Err(Denial::DiscontinuousEndpoints);
    }
    Ok(introduced)
}
