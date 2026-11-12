//! Versioned shared contents and references remain untrusted descriptive meaning.
use super::*;
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};
use worth_query_declaration::facade::application_schema::ApplicationDecisionReadScopeMeaning;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    output.u16(1)?;
    match member {
        ApplicationSchemaMember::DecisionReadScope { definition } => {
            write_sequence(
                output,
                definition.targets(),
                operation::write_decision_read_target,
            )?;
            write_sequence(output, definition.derived_relations(), |output, name| {
                output.text(name)
            })
        }
        ApplicationSchemaMember::OperationDecisionReadScope { operation, scope } => {
            output.text(operation)?;
            output.text(scope)
        }
        _ => unreachable!("only shared decision-read members select this codec"),
    }
}

pub(super) fn decode(
    tag: u16,
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    if input.u16()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    if tag == 56 {
        return Ok(ApplicationSchemaMember::OperationDecisionReadScope {
            operation: input.text()?.to_owned(),
            scope: input.text()?.to_owned(),
        });
    }
    budget.require_nesting_depth(2)?;
    let targets = decode_sequence(input, budget, 2, |input, _| {
        operation::decode_decision_read_target(input)
    })?;
    let derived = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
    // The SDK normalizes authored contents; wire input must already be canonical.
    require_canonical_sequence(&targets)?;
    require_canonical_sequence(&derived)?;
    let definition = ApplicationDecisionReadScopeMeaning::from_untrusted_parts(targets, derived)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(ApplicationSchemaMember::DecisionReadScope { definition })
}
