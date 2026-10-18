//! One prepared fixed-shape edit of existing native leaves. No freshness grant.
use super::{Fact, FactPayload, FactStorage, StorageControl};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StoreDenial,
    fact_verification_observation as observation,
};
use std::cell::RefCell;
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};

mod leaf;
mod preparation;
use leaf::PreparedLeaf;
use preparation::{build, count_nodes};

pub(in crate::domain_computation::primary_graph) struct NativeFactPatchBuilder {
    nodes: ExecutionArray<RefCell<Option<PreparedNode>>>,
    root: usize,
    replacements: usize,
    written: usize,
}

enum PreparedNode {
    Keep(FactPayload),
    Leaf(PreparedLeaf),
    Branch {
        left: usize,
        right: usize,
        left_len: usize,
        output: ExecutionArrayBuilder<FactPayload>,
    },
    Published(FactPayload),
}

impl FactPayload {
    /// Pre-admit every touched leaf, its deep-copy funding, paths and scratch.
    /// Selection is storage input only; callers still need native read/source
    /// authority before sharing an unchanged fact into a new completed read.
    pub(in crate::domain_computation::primary_graph) fn prepare_native_patch(
        &self,
        changed: &im::OrdSet<usize>,
        control: StorageControl<'_, '_>,
    ) -> Result<NativeFactPatchBuilder, StoreDenial> {
        control.check_live()?;
        if changed
            .iter()
            .next_back()
            .is_some_and(|ordinal| *ordinal >= self.len())
        {
            return Err(StoreDenial::ConflictingBody);
        }
        let count = count_nodes(self, changed, 0, control)?;
        let mut nodes = ExecutionArrayBuilder::allocate(count, control.policy())?;
        let root = build(self, changed, 0, &mut nodes, control)?;
        let nodes = nodes.seal()?;
        control.check_live()?;
        Ok(NativeFactPatchBuilder {
            nodes,
            root,
            replacements: changed.len(),
            written: 0,
        })
    }
}

impl NativeFactPatchBuilder {
    /// Only a planned ordinal may be filled, exactly once, before effects.
    /// Admit the actual replacement's deep copy before making it. A borrowed
    /// Fact carries no transferable funding or source/currentness authority.
    pub(in crate::domain_computation::primary_graph) fn replace(
        &mut self,
        mut ordinal: usize,
        fact: &Fact,
        control: StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        let mut index = self.root;
        loop {
            control.check_live()?;
            observation::native_patch_path();
            let mut slot = self.nodes[index].borrow_mut();
            match slot.as_mut().expect("prepared patch node") {
                PreparedNode::Branch {
                    left,
                    right,
                    left_len,
                    ..
                } => {
                    if ordinal < *left_len {
                        index = *left;
                    } else {
                        ordinal -= *left_len;
                        index = *right;
                    }
                }
                PreparedNode::Leaf(leaf) => {
                    if ordinal >= leaf.replacements.len() {
                        return Err(StoreDenial::ConflictingBody);
                    }
                    let bit = 1_u32 << ordinal;
                    if leaf.changed & bit == 0 || leaf.filled & bit != 0 {
                        return Err(StoreDenial::ConflictingBody);
                    }
                    let quote = super::chunk::deep_clone_bytes(fact)
                        .ok_or(StoreDenial::Representability)?;
                    let funding = control.policy().reserve_memory(quote);
                    control.check_live()?;
                    let funding = funding?;
                    observation::native_patch_clone();
                    let copied = fact.clone();
                    control.check_live()?;
                    if let Some(funding) = funding {
                        leaf.replacement_heaps[ordinal]
                            .set(funding)
                            .expect("one planned replacement grant");
                    }
                    *leaf.replacements[ordinal].borrow_mut() = Some(copied);
                    leaf.filled |= bit;
                    self.written += 1;
                    return Ok(());
                }
                PreparedNode::Keep(_) | PreparedNode::Published(_) => {
                    return Err(StoreDenial::ConflictingBody)
                }
            }
        }
    }

    /// All storage was prepared before effects. Only changed leaves are
    /// filled; untouched branches keep their exact old Arc and admission.
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        control: StorageControl<'_, '_>,
    ) -> Result<FactPayload, StoreDenial> {
        control.check_live()?;
        if self.written != self.replacements {
            return Err(StoreDenial::ConflictingBody);
        }
        for index in 0..self.nodes.len() {
            control.check_live()?;
            observation::native_patch_path();
            let node = self.nodes[index]
                .borrow_mut()
                .take()
                .expect("one prepared node");
            let value = match node {
                PreparedNode::Keep(value) => value,
                PreparedNode::Leaf(leaf) => leaf.finish(control)?,
                PreparedNode::Branch {
                    left,
                    right,
                    mut output,
                    ..
                } => {
                    output.push(take_published(&self.nodes, left))?;
                    output.push(take_published(&self.nodes, right))?;
                    let value = FactPayload::joined(output.seal()?);
                    observation::native_payload_join();
                    value
                }
                PreparedNode::Published(_) => unreachable!("postorder author writes a node once"),
            };
            *self.nodes[index].borrow_mut() = Some(PreparedNode::Published(value));
        }
        control.check_live()?;
        Ok(take_published(&self.nodes, self.root))
    }
}

fn take_published(
    nodes: &ExecutionArray<RefCell<Option<PreparedNode>>>,
    index: usize,
) -> FactPayload {
    let Some(PreparedNode::Published(value)) = nodes[index].borrow_mut().take() else {
        unreachable!("children seal before parent")
    };
    value
}

#[cfg(test)]
mod tests;
