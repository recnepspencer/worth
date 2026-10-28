//! Translate only closed syntax to the one Native registered evaluator.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationNode as Node, ApplicationDerivedRelationSourceKind as SourceKind,
    WorthQueryPortableDerivedRelationParts,
};
use worth_relational::facade::schema::DerivedRelationNode as Native;

pub(super) fn lower(
    parts: &WorthQueryPortableDerivedRelationParts,
    identity: &impl Fn(&str) -> DerivedRelationIdentity,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    domains: &BTreeMap<String, super::super::projected_member_domain::ProjectedMemberDomainLayout>,
) -> Result<Vec<Native>, Denial> {
    parts
        .expression
        .nodes
        .iter()
        .map(|node| {
            Ok(match node {
                Node::Source(source) => match source.kind {
                    SourceKind::Stored => Native::Stored {
                        relation: required_kind(relations, &source.relation)?,
                        from: required_kind(
                            entities,
                            source
                                .from
                                .entity()
                                .ok_or_else(|| invalid_member(&source.relation))?,
                        )?,
                        to: required_kind(
                            entities,
                            source
                                .to
                                .entity()
                                .ok_or_else(|| invalid_member(&source.relation))?,
                        )?,
                    },
                    SourceKind::Derived => Native::Derived(identity(&source.relation)),
                },
                Node::Compose { left, right } => Native::Compose {
                    left: *left,
                    right: *right,
                },
                Node::Reverse { input } => Native::Reverse { input: *input },
                Node::WithoutSelfEdges { input } => Native::WithoutSelfEdges { input: *input },
                Node::CarrierLift {
                    domain,
                    carrier,
                    case,
                } => {
                    let layout = domains
                        .get(domain)
                        .filter(|layout| layout.carrier.as_ref() == carrier)
                        .ok_or_else(|| invalid_member(domain))?;
                    if layout.handle.case(case).is_none() {
                        return Err(invalid_member(case));
                    }
                    Native::CarrierLift {
                        domain: layout.handle.parts().identity.clone(),
                        case: case.clone(),
                    }
                }
                Node::Union { left, right } => Native::Union {
                    left: *left,
                    right: *right,
                },
                Node::Intersection { left, right } => Native::Intersection {
                    left: *left,
                    right: *right,
                },
            })
        })
        .collect()
}
