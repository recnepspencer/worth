use super::{Denial, Read, Write};
use crate::data::node::NodeEvaluationConfig;
use crate::schema::data::{
    SignalSchemaBinding, SignalSchemaId, SignalSchemaName, SignalSchemaVersion,
};

pub(super) fn write(out: &mut Write<'_>, value: &NodeEvaluationConfig) -> Result<(), Denial> {
    let NodeEvaluationConfig {
        schema_binding,
        contract,
        condition,
        comparator,
        output_equivalence,
        partitioned_output,
    } = value;
    out.boolean(schema_binding.is_some())?;
    if let Some(binding) = schema_binding {
        out.small_number(binding.schema_id().0)?;
        out.text(binding.semantic_name().as_str())?;
        out.small_number(binding.version().major as u32)?;
        out.small_number(binding.version().minor as u32)?;
        out.text(binding.descriptor_digest())?;
    }
    // The six retired optional policy names and the empty binding list
    // preserve the ordinary authored-image wire layout.
    for _ in 0..6 {
        out.boolean(false)?;
    }
    out.count(0)?;
    super::contracts::write(out, contract)?;
    super::policies::write_condition(out, condition)?;
    out.boolean(comparator.is_some())?;
    if let Some(comparator) = comparator {
        super::policies::write_comparator(out, comparator)?;
    }
    super::policies::write_output(out, output_equivalence)?;
    out.boolean(*partitioned_output)
}

pub(super) fn read(read: &mut Read<'_>) -> Result<NodeEvaluationConfig, Denial> {
    let schema_binding = if read.boolean()? {
        Some(SignalSchemaBinding::new(
            SignalSchemaId(read.small_number()?),
            SignalSchemaName::new(read.text()?),
            SignalSchemaVersion::new(
                read.small_number()?
                    .try_into()
                    .map_err(|_| Denial::Malformed)?,
                read.small_number()?
                    .try_into()
                    .map_err(|_| Denial::Malformed)?,
            ),
            read.text()?,
        ))
    } else {
        None
    };
    for _ in 0..6 {
        if read.boolean()? {
            return Err(Denial::LegacyMergeArtifactRetired);
        }
    }
    if read.count()? != 0 {
        return Err(Denial::LegacyMergeArtifactRetired);
    }
    let contract = super::contracts::read(read)?;
    let condition = super::policies::read_condition(read)?;
    let comparator = if read.boolean()? {
        Some(super::policies::read_comparator(read)?)
    } else {
        None
    };
    let output_equivalence = super::policies::read_output(read)?;
    let partitioned_output = read.boolean()?;
    Ok(NodeEvaluationConfig {
        schema_binding,
        contract,
        condition,
        comparator,
        output_equivalence,
        partitioned_output,
    })
}
