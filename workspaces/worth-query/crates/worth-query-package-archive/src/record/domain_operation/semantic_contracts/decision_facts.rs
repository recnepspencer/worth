use super::vocabulary::{decision_kind, decision_kind_tag};
use super::{read_usize, unsupported, write_usize};
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};
use worth_query_installation::facade::*;

pub(in crate::record::domain_operation) fn write_decision_facts(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOperationDecisionFactContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryOperationDecisionFactContract::NotRequired => output.u16(1),
        WorthQueryOperationDecisionFactContract::Declared { required_families } => {
            output.u16(2)?;
            write_sequence(output, required_families, |output, family| {
                output.text(family.identity())?;
                output.u16(decision_kind_tag(family.kind()))?;
                match family.cardinality() {
                    WorthQueryDecisionFactCardinality::Variable => output.u16(3),
                    WorthQueryDecisionFactCardinality::Exact(count) => {
                        output.u16(1)?;
                        write_usize(output, count)
                    }
                    WorthQueryDecisionFactCardinality::Bounded { maximum } => {
                        output.u16(2)?;
                        write_usize(output, maximum)
                    }
                }
            })
        }
    }
}

pub(in crate::record::domain_operation) fn decode_decision_facts(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryOperationDecisionFactContract, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOperationDecisionFactContract::NotRequired),
        2 => {
            let families = decode_sequence(input, budget, 8, |input, _| {
                let identity = input.text()?.to_owned();
                let kind = decision_kind(input.u16()?)?;
                let tag = input.u16()?;
                let family = WorthQueryDecisionFactFamily::new(identity, kind)
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
                match tag {
                    1 => family.with_exact_fact_count(read_usize(input)?),
                    2 => family.with_bounded_fact_count(read_usize(input)?),
                    3 => Ok(family.with_variable_fact_count()),
                    _ => return unsupported(),
                }
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))
            })?;
            require_canonical_sequence(&families)?;
            WorthQueryOperationDecisionFactContract::declared(families)
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))
        }
        _ => unsupported(),
    }
}
