//! One authored encoding of Bridge-owned async declarations and source nodes.
use super::{
    fields::{Reader, Writer},
    rows::Rows,
    BridgeAuthoredSectionDenial as D,
};
use crate::conditional_execution::owned_async::{
    definition_source::OwnedAsyncDefinitionSource, BridgeOwnedAsyncDeclarationDescription,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::{DurableSignalBranchIdentity, DurableSignalNodeIdentity};
pub(super) fn write(
    out: &mut Writer<'_>,
    branch: DurableSignalBranchIdentity,
    rows: &[&OwnedAsyncDefinitionSource],
) -> Result<(), D> {
    out.number(rows.len() as u64)?;
    for row in rows {
        out.record(&mut |out| {
            out.node(DurableSignalNodeIdentity::from_recorded(
                branch,
                row.node.index(),
                row.node.generation(),
            ))?;
            out.json(&row.description)
        })?;
    }
    Ok(())
}
pub(super) fn read(
    bytes: &[u8],
    branch: DurableSignalBranchIdentity,
) -> Result<BTreeMap<Arc<str>, OwnedAsyncDefinitionSource>, D> {
    let mut rows = Rows::new(bytes)?;
    let mut definitions = BTreeMap::new();
    let mut nodes = std::collections::BTreeSet::new();
    while let Some(bytes) = rows.next()? {
        let mut input = Reader::new(bytes);
        let node = input.node()?;
        if node.branch() != branch {
            return Err(D::ForeignBranch);
        }
        let description: BridgeOwnedAsyncDeclarationDescription =
            serde_json::from_slice(input.blob()?).map_err(|_| D::Malformed)?;
        input.finish()?;
        let source = OwnedAsyncDefinitionSource {
            node: worth_signal::facade::NodeId::new(node.index(), node.generation()),
            description,
        };
        if source.description.identity.is_empty()
            || !nodes.insert(node)
            || definitions
                .insert(Arc::from(source.description.identity.as_str()), source)
                .is_some()
        {
            return Err(D::Malformed);
        }
    }
    rows.finish()?;
    Ok(definitions)
}
