//! One stable-slot tree, maintained only by genuine Native occurrence changes.
use super::{storage::Storage, AccountIndexWork};
use bank_domain::{queries::CurrentAccountsIndex, schema::BankSchema};
use std::{
    fmt,
    sync::{atomic::Ordering, Arc},
};
use worth_query_host::facade::{
    primary_graph::*,
    runtime::{
        build_sorted_ordered_tree, insert_ordered_tree, remove_ordered_tree,
        ExecutionOrderedTreeNode,
    },
};
pub(super) type Node =
    ExecutionOrderedTreeNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub(super) type Stop = WorthQueryManagedIndexProviderStop<Denial>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Denial {
    InvalidTree,
}
impl fmt::Display for Denial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Denial {}
pub struct Provider(pub Arc<AccountIndexWork>);
impl WorthQueryManagedIndexProvider<BankSchema, CurrentAccountsIndex> for Provider {
    type Node = Node;
    type Summary = ();
    type Denial = Denial;
    const IDENTITY: &'static str = "worth.bank.account-roster.v1";
    const VERSION: u32 = 1;
    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(source_count)
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, BankSchema, CurrentAccountsIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.builds.fetch_add(1, Ordering::Relaxed);
        let slots = sources.slots().map(|slot| {
            let slot = slot?;
            sources.get(slot)?;
            Ok(slot)
        });
        build_sorted_ordered_tree(sources.len(), slots, &mut Storage { nodes, control })
    }
    fn update(
        &self,
        before: &WorthQueryManagedIndexSourceRows<'_, BankSchema, CurrentAccountsIndex>,
        after: &WorthQueryManagedIndexSourceRows<'_, BankSchema, CurrentAccountsIndex>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::Relaxed);
        self.0.changes.fetch_add(changes.len(), Ordering::Relaxed);
        for change in changes {
            control.checkpoint(1)?;
            if let Some(slot) = change.before() {
                before.get(slot)?;
                root = remove_ordered_tree(root, slot, &mut Storage { nodes, control })?;
            }
        }
        for change in changes {
            control.checkpoint(1)?;
            if let Some(slot) = change.after() {
                after.get(slot)?;
                root = Some(insert_ordered_tree(
                    root,
                    slot,
                    &mut Storage { nodes, control },
                )?);
            }
        }
        Ok(root)
    }
    fn validate_probe(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<BankSchema, CurrentAccountsIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(1)?;
        let _ = probe;
        Ok(())
    }
    fn matches_source(
        &self,
        _row: WorthQueryManagedIndexSourceRow<'_, BankSchema, CurrentAccountsIndex>,
        probe: &WorthQueryManagedIndexProbeValue<BankSchema, CurrentAccountsIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.validate_probe(probe, control)?;
        Ok(true)
    }
    fn decompose(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        control.checkpoint(1)?;
        Ok(WorthQueryManagedIndexNodeRegion::new(
            Some(node.source),
            [node.left, node.right],
        ))
    }
}
