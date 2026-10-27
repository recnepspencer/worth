//! Derived source syntax and actual declared collection identities, never record IDs.
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_value::{decode_bool, write_bool};
use crate::record::sequence::{decode_sequence, write_sequence};
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationExpression, ApplicationDerivedRelationNode as Node,
    ApplicationDerivedRelationSource, ApplicationDerivedRelationSourceKind as SourceKind,
    ApplicationRelationGraphLaws, ApplicationRuleReadableCollectionAssociation,
    ApplicationSchemaMember as Member, ErasedApplicationDerivedRelationDefinition,
    WorthQueryPortableDerivedRelationParts,
};

use super::super::wire_vocabulary::{decode_optional, write_optional};

pub(super) fn require_shape(member: &Member) -> Result<(), Denial> {
    if let Member::DerivedRelation { definition } = member {
        definition
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    if let Member::RuleReadableCollection { association } = member {
        association
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    Ok(())
}

pub(super) fn write(output: &mut dyn BinaryEncodingSink, member: &Member) -> Result<(), Denial> {
    match member {
        Member::DerivedRelation { definition } => {
            let parts = definition.parts();
            for text in [&parts.relation, &parts.from, &parts.to] {
                output.text(text)?;
            }
            write_sequence(output, &parts.expression.nodes, write_node)?;
            super::relation_graph::write_prefix(
                output,
                parts.laws.endpoints,
                parts.laws.cardinality,
            )?;
            write_bool(output, parts.laws.acyclic)?;
            write_bool(output, parts.laws.symmetric)?;
            write_bool(output, parts.laws.orbit)?;
            write_bool(output, parts.laws.same_partition_endpoints)?;
            write_optional(
                output,
                parts.laws.reaches_at_least.as_ref(),
                super::relation_graph::write_minimum,
            )
        }
        Member::RuleReadableCollection { association } => {
            for text in [&association.relation, &association.feature] {
                output.text(text)?;
            }
            output.u16(association.feature_major)?;
            output.u16(association.feature_minor)?;
            for text in [
                &association.collection,
                &association.computation,
                &association.output_artifact,
                &association.completeness,
            ] {
                output.text(text)?;
            }
            Ok(())
        }
        Member::OperationDerivedRead {
            operation,
            relation,
        } => {
            output.text(operation)?;
            output.text(relation)
        }
        _ => unreachable!("derived member codec router"),
    }
}

pub(super) fn decode(
    tag: u16,
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Member, Denial> {
    let member = match tag {
        41 => {
            budget.require_nesting_depth(2)?;
            let relation = input.text()?.to_owned();
            let from = input.text()?.to_owned();
            let to = input.text()?.to_owned();
            let nodes = decode_sequence(input, budget, 10, |input, _| decode_node(input))?;
            let (endpoints, cardinality) = super::relation_graph::decode_prefix(input)?;
            let laws = ApplicationRelationGraphLaws {
                endpoints,
                cardinality,
                acyclic: decode_bool(input)?,
                symmetric: decode_bool(input)?,
                orbit: decode_bool(input)?,
                same_partition_endpoints: decode_bool(input)?,
                reaches_at_least: decode_optional(input, |input| {
                    super::relation_graph::decode_minimum(input, budget)
                })?,
            };
            Member::DerivedRelation {
                definition: ErasedApplicationDerivedRelationDefinition::from_untrusted_parts(
                    WorthQueryPortableDerivedRelationParts {
                        relation,
                        from,
                        to,
                        expression: ApplicationDerivedRelationExpression { nodes },
                        laws,
                    },
                ),
            }
        }
        42 => Member::RuleReadableCollection {
            association: ApplicationRuleReadableCollectionAssociation {
                relation: input.text()?.to_owned(),
                feature: input.text()?.to_owned(),
                feature_major: input.u16()?,
                feature_minor: input.u16()?,
                collection: input.text()?.to_owned(),
                computation: input.text()?.to_owned(),
                output_artifact: input.text()?.to_owned(),
                completeness: input.text()?.to_owned(),
            },
        },
        43 => Member::OperationDerivedRead {
            operation: input.text()?.to_owned(),
            relation: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    require_shape(&member)?;
    Ok(member)
}

fn write_node(output: &mut dyn BinaryEncodingSink, node: &Node) -> Result<(), Denial> {
    match node {
        Node::Source(source) => {
            output.u16(1)?;
            output.u16(match source.kind {
                SourceKind::Stored => 1,
                SourceKind::Derived => 2,
            })?;
            for text in [&source.relation, &source.from, &source.to] {
                output.text(text)?;
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
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        }),
        2 => Node::Compose {
            left: decode_index(input)?,
            right: decode_index(input)?,
        },
        3 => Node::Reverse {
            input: decode_index(input)?,
        },
        4 => Node::WithoutSelfEdges {
            input: decode_index(input)?,
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn decode_index(input: &mut BinaryInput<'_>) -> Result<usize, Denial> {
    usize::try_from(input.u64()?).map_err(|_| Denial::new(Kind::InvalidRecordShape))
}
