//! One versioned wire grammar for symbolic expressions and their endpoint namespaces.
use super::*;
use crate::record::sequence::{decode_sequence, write_sequence};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationExpression, ApplicationDerivedRelationNode as Node,
    ApplicationDerivedRelationSource, ApplicationDerivedRelationSourceKind as SourceKind,
    ApplicationGraphMemberMeaning as Domain, ErasedApplicationDerivedRelationDefinition,
    WorthQueryPortableDerivedRelationParts,
};

const EXPRESSION_WIRE_VERSION: u16 = 3;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationDerivedRelationDefinition,
) -> Result<(), Denial> {
    output.u16(EXPRESSION_WIRE_VERSION)?;
    let parts = definition.parts();
    output.text(&parts.relation)?;
    write_member_domain(output, &parts.from)?;
    write_member_domain(output, &parts.to)?;
    write_sequence(output, &parts.expression.nodes, write_node)?;
    super::write_laws(output, &parts.laws)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Member, Denial> {
    budget.require_nesting_depth(2)?;
    let version = input.u16()?;
    if version != EXPRESSION_WIRE_VERSION {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let relation = input.text()?.to_owned();
    let from = decode_member_domain(input)?;
    let to = decode_member_domain(input)?;
    let nodes = decode_sequence(input, budget, 10, |input, _| decode_node(input))?;
    let laws = super::decode_laws(input, budget)?;
    let definition = ErasedApplicationDerivedRelationDefinition::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableDerivedRelationParts {
            relation,
            from,
            to,
            expression: ApplicationDerivedRelationExpression { nodes },
            laws,
        },
    );
    let member = Member::DerivedRelation { definition };
    super::require_shape(&member)?;
    Ok(member)
}

pub(in crate::record::application_schema::member) fn write_member_domain(
    output: &mut dyn BinaryEncodingSink,
    domain: &Domain,
) -> Result<(), Denial> {
    output.u16(match domain {
        Domain::Entity(_) => 1,
        Domain::Projected(_) => 2,
    })?;
    output.text(domain.name())
}
pub(in crate::record::application_schema::member) fn decode_member_domain(
    input: &mut BinaryInput<'_>,
) -> Result<Domain, Denial> {
    match input.u16()? {
        1 => Ok(Domain::Entity(input.text()?.to_owned())),
        2 => Ok(Domain::Projected(input.text()?.to_owned())),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn write_node(output: &mut dyn BinaryEncodingSink, node: &Node) -> Result<(), Denial> {
    match node {
        Node::Source(source) => {
            output.u16(1)?;
            output.u16(match source.kind {
                SourceKind::Stored => 1,
                SourceKind::Derived => 2,
            })?;
            output.text(&source.relation)?;
            write_member_domain(output, &source.from)?;
            write_member_domain(output, &source.to)
        }
        Node::CarrierLift {
            domain,
            carrier,
            case,
        } => {
            output.u16(5)?;
            for name in [domain, carrier, case] {
                output.text(name)?;
            }
            Ok(())
        }
        Node::Union { left, right } | Node::Intersection { left, right } => {
            output.u16(if matches!(node, Node::Union { .. }) {
                6
            } else {
                7
            })?;
            output.u64(*left as u64)?;
            output.u64(*right as u64)
        }
        Node::OrderedEvents {
            predecessor,
            introduced,
            retired,
        } => {
            output.u16(8)?;
            for index in [predecessor, introduced, retired] {
                output.u64(*index as u64)?;
            }
            Ok(())
        }
        Node::Compose { left, right } => {
            output.u16(2)?;
            output.u64(*left as u64)?;
            output.u64(*right as u64)
        }
        Node::Reverse { input } | Node::WithoutSelfEdges { input } => {
            output.u16(if matches!(node, Node::Reverse { .. }) {
                3
            } else {
                4
            })?;
            output.u64(*input as u64)
        }
    }
}
fn decode_node(input: &mut BinaryInput<'_>) -> Result<Node, Denial> {
    Ok(match input.u16()? {
        1 => Node::Source(ApplicationDerivedRelationSource {
            kind: match input.u16()? {
                1 => SourceKind::Stored,
                2 => SourceKind::Derived,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
            relation: input.text()?.to_owned(),
            from: decode_member_domain(input)?,
            to: decode_member_domain(input)?,
        }),
        2 => Node::Compose {
            left: super::decode_index(input)?,
            right: super::decode_index(input)?,
        },
        3 => Node::Reverse {
            input: super::decode_index(input)?,
        },
        4 => Node::WithoutSelfEdges {
            input: super::decode_index(input)?,
        },
        5 => Node::CarrierLift {
            domain: input.text()?.to_owned(),
            carrier: input.text()?.to_owned(),
            case: input.text()?.to_owned(),
        },
        6 => Node::Union {
            left: super::decode_index(input)?,
            right: super::decode_index(input)?,
        },
        7 => Node::Intersection {
            left: super::decode_index(input)?,
            right: super::decode_index(input)?,
        },
        8 => Node::OrderedEvents {
            predecessor: super::decode_index(input)?,
            introduced: super::decode_index(input)?,
            retired: super::decode_index(input)?,
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}
