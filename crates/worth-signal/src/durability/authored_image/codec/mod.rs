//! Concrete authored-declaration codec. It never visits node evaluation storage.
mod configuration;
mod contracts;
mod execution_tags;
mod policies;
mod primitives;
mod reuse_tags;
mod temporal;
#[cfg(test)]
mod tests;
use super::SignalAuthoredImageDenial as Denial;
use crate::data::handle::NodeId;
use crate::data::node::NodeDefinitionData;
use primitives::Read;
pub(crate) use primitives::Write;

pub(crate) fn write_definition(
    out: &mut Write<'_>,
    value: &NodeDefinitionData,
) -> Result<(), Denial> {
    let NodeDefinitionData {
        declared_dependencies,
        tombstoned,
        conditional_contract_generation,
        conditional_contract_occurrence,
        eval_config,
    } = value;
    out.boolean(*tombstoned)?;
    out.number(*conditional_contract_generation)?;
    out.number(*conditional_contract_occurrence)?;
    configuration::write(out, eval_config)?;
    let dependencies = declared_dependencies
        .as_ref()
        .map_or(&[][..], |value| value.as_slice());
    out.count(dependencies.len())?;
    for edge in dependencies {
        out.node(edge.source())?;
        out.byte(edge.aspect().index() as u8)?;
        out.scope(edge.scope_ref())?;
    }
    Ok(())
}

pub(super) fn validate(payload: &[u8]) -> Result<(), Denial> {
    let mut read = Read::new(payload);
    let slots = read.count()?;
    for index in 0..slots {
        let generation = read.small_number()?;
        let retired = read.boolean()?;
        if read.boolean()? {
            if retired {
                return Err(Denial::Malformed);
            }
            read.node = NodeId::new(index as u32, generation);
            read.boolean()?;
            read.number()?;
            read.number()?;
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
    read.finish()
}
