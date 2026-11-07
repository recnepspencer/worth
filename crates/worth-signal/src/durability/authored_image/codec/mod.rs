//! Concrete authored-declaration codec. It never visits node evaluation storage.
mod conditional_definition;
mod configuration;
mod contracts;
mod execution_tags;
mod graph_rows;
mod threshold_value;
pub(crate) use graph_rows::{read as read_graph_rows, AuthoredGraphRows};
mod policies;
mod primitives;
mod reuse_tags;
mod runtime_rows;
mod runtime_tiers;
pub(crate) use runtime_rows::{read as read_runtime_rows, write as write_runtime_rows};
pub(crate) use runtime_rows::{
    ComputationDefinitionView, KeyedNodeDefinitionView, RuntimeDefinitionRows,
    RuntimeDefinitionView,
};
mod temporal;
#[cfg(test)]
mod tests;
use super::SignalAuthoredImageDenial as Denial;
use crate::data::node::NodeDefinitionData;
use primitives::Read;
pub(crate) use primitives::Write;

pub(crate) fn write_definition(
    out: &mut Write<'_>,
    value: &NodeDefinitionData,
) -> Result<(), Denial> {
    graph_rows::validate_definition_source(value)?;
    let NodeDefinitionData {
        conditional_definition,
        declared_dependencies,
        tombstoned,
        conditional_contract_generation: _,
        conditional_contract_occurrence: _,
        eval_config,
    } = value;
    out.boolean(*tombstoned)?;
    out.boolean(conditional_definition.is_some())?;
    if let Some(definition) = conditional_definition {
        conditional_definition::write(out, definition)?;
    }
    configuration::write(out, eval_config)?;
    let dependencies = declared_dependencies
        .as_ref()
        .map_or(&[][..], |value| value.as_slice());
    out.count(dependencies.len())?;
    for edge in dependencies {
        out.node(out.scope_node(edge.source()))?;
        out.byte(edge.aspect().index() as u8)?;
        out.scope(edge.scope_ref())?;
    }
    Ok(())
}

pub(super) fn validate(
    payload: &[u8],
) -> Result<worth_foundational::facade::DurableSignalBranchIdentity, Denial> {
    let mut read = Read::image(payload)?;
    let branch = read.branch;
    let slots = read.count()?;
    for _ in 0..slots {
        read.small_number()?;
        let retired = read.boolean()?;
        if read.boolean()? {
            if retired {
                return Err(Denial::Malformed);
            }
            read.boolean()?;
            if read.boolean()? {
                conditional_definition::read(&mut read)?;
            }
            configuration::read(&mut read)?;
            let edges = read.count()?;
            for _ in 0..edges {
                read.node_id()?;
                read.aspect()?;
                read.scope()?;
            }
        }
    }
    let free = read.count()?;
    for _ in 0..free {
        if read.small_number()? as usize >= slots {
            return Err(Denial::Malformed);
        }
    }
    read.finish()?;
    Ok(branch)
}
