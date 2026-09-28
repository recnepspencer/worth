//! The condition node record. Version 2 carries the expression's encoded
//! draft and its operands in name order; version 1 named one Bool query and
//! decodes as that query read under the migrated operand.

use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowCondition, MIGRATED_WORKFLOW_CONDITION_OPERAND,
};

use super::{nonempty_text, text};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::workflow_definition::{DraftCondition, DraftConditionOperand};

/// Four empty texts: the smallest operand record.
const MINIMUM_OPERAND_BYTES: usize = 4 * 4;

pub(super) fn encode_condition(
    output: &mut BinaryOutput,
    condition: &ApplicationWorkflowCondition,
) -> Result<(), Denial> {
    let draft = condition.draft();
    output.u32(u32::try_from(draft.len()).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?);
    output.raw_bytes(draft);
    let operands = condition.operands();
    output.u16(
        u16::try_from(operands.len()).map_err(|_| Denial::new(Kind::NestedEntryBudgetExceeded))?,
    );
    for operand in operands {
        let query = operand.query();
        for field in [
            operand.name(),
            query.identifier(),
            query.parameter_type().as_str(),
            query.result_type().as_str(),
        ] {
            text(output, field)?;
        }
    }
    Ok(())
}

pub(super) fn decode_condition(input: &mut BinaryInput<'_>) -> Result<DraftCondition, Denial> {
    let length = usize::try_from(input.u32()?).map_err(|_| Denial::new(Kind::Truncated))?;
    let draft = input.take(length)?;
    if draft.is_empty() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let count = usize::from(input.u16()?);
    if count == 0 {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    if count.saturating_mul(MINIMUM_OPERAND_BYTES) > input.remaining_len() {
        return Err(Denial::new(Kind::Truncated));
    }
    let mut operands: Vec<DraftConditionOperand> = Vec::with_capacity(count);
    for _ in 0..count {
        let name = nonempty_text(input)?;
        let operand = decode_operand(input, name)?;
        if operands
            .last()
            .is_some_and(|previous| previous.name >= operand.name)
        {
            return Err(Denial::new(Kind::NonCanonicalRecordSequence));
        }
        operands.push(operand);
    }
    Ok(DraftCondition::Expression {
        draft: draft.into(),
        operands: operands.into_boxed_slice(),
    })
}

/// A version-1 record: the query alone, read as the whole decision.
pub(super) fn decode_version_1_condition(
    input: &mut BinaryInput<'_>,
) -> Result<DraftCondition, Denial> {
    decode_operand(input, MIGRATED_WORKFLOW_CONDITION_OPERAND.to_owned())
        .map(DraftCondition::Migrated)
}

fn decode_operand(
    input: &mut BinaryInput<'_>,
    name: String,
) -> Result<DraftConditionOperand, Denial> {
    Ok(DraftConditionOperand {
        name,
        identifier: nonempty_text(input)?,
        parameter_type: nonempty_text(input)?,
        result_type: nonempty_text(input)?,
    })
}
