//! Derived source syntax and actual declared collection identities, never record IDs.
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_value::{decode_bool, write_bool};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationGraphLaws, ApplicationRuleReadableCollectionAssociation,
    ApplicationSchemaMember as Member,
};

mod expression_wire;
pub(super) use expression_wire::{decode_member_domain, write_member_domain};

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
        Member::DerivedRelation { definition } => expression_wire::write(output, definition),
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
        47 => return expression_wire::decode(input, budget),
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

fn decode_index(input: &mut BinaryInput<'_>) -> Result<usize, Denial> {
    usize::try_from(input.u64()?).map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

fn write_laws(
    output: &mut dyn BinaryEncodingSink,
    laws: &ApplicationRelationGraphLaws,
) -> Result<(), Denial> {
    super::relation_graph::write_prefix(output, laws.endpoints, laws.cardinality)?;
    write_bool(output, laws.acyclic)?;
    write_bool(output, laws.symmetric)?;
    write_bool(output, laws.orbit)?;
    write_bool(output, laws.same_partition_endpoints)?;
    write_optional(
        output,
        laws.reaches_at_least.as_ref(),
        super::relation_graph::write_minimum,
    )
}

fn decode_laws(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationRelationGraphLaws, Denial> {
    let (endpoints, cardinality) = super::relation_graph::decode_prefix(input)?;
    Ok(ApplicationRelationGraphLaws {
        endpoints,
        cardinality,
        acyclic: decode_bool(input)?,
        symmetric: decode_bool(input)?,
        orbit: decode_bool(input)?,
        same_partition_endpoints: decode_bool(input)?,
        reaches_at_least: decode_optional(input, |input| {
            super::relation_graph::decode_minimum(input, budget)
        })?,
    })
}
