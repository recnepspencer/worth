//! One planned definition: its recorded node and full source contract.
use super::{conditional_definition, Denial, Read, Write};
use crate::data::conditional_execution::SignalConditionalContractDefinition as Definition;
use worth_foundational::facade::DurableSignalNodeIdentity;

pub(crate) fn write(
    out: &mut Write<'_>,
    node: DurableSignalNodeIdentity,
    definition: &Definition,
) -> Result<(), Denial> {
    out.node(node)?;
    conditional_definition::write(out, definition)
}

pub(crate) fn read(payload: &[u8]) -> Result<(DurableSignalNodeIdentity, Definition), Denial> {
    let mut input = Read::image(payload)?;
    input.materialize = true;
    let node =
        DurableSignalNodeIdentity::from_recorded(input.branch, input.small_number()?, input.small_number()?);
    let definition = conditional_definition::read(&mut input)?;
    input.finish()?;
    Ok((node, definition))
}
