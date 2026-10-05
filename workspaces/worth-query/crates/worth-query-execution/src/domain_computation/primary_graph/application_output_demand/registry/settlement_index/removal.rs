//! Exact posting removal is funded before its terminal queue head is unlinked.

use super::*;

impl SettlementIndex {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn admit_selected_removal_work<
        'a,
    >(
        &self,
        identities: impl Iterator<Item = &'a Arc<RecordedSettlementIdentity>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        // Fund header reads and the bounded level calculation before measuring
        // either tree. The denial subject is empty even on this first refusal.
        charge(admission, 4 + usize::BITS as u64)?;
        let count = self.sources.len();
        let outer = comparison_work::<SemanticSource>(count, 16)?;
        let outer_moves = movement_work::<SemanticSource, SourcePostings>(count)?;
        for identity in identities {
            navigate(admission, outer)?;
            charge(admission, 4)?;
            let source = self
                .sources
                .get(identity.source())
                .expect("queued settlement source remains retained");
            charge(admission, 2 + usize::BITS as u64)?;
            let count = source.len();
            let inner = comparison_work::<Address>(count, 4)?;
            let inner_moves = movement_work::<Address, Posting>(count)?;
            // remove() gets the outer bucket again, removes its selected inner
            // address, checks the full recorded identity, then may remove the
            // empty bucket. Quote that last removal for every identity: an
            // earlier removal in this batch can make the bucket's final row.
            let identity_work = size_of::<RecordedSettlementIdentity>()
                .checked_mul(2)
                .and_then(|bytes| bytes.checked_add(size_of::<Address>()))
                .and_then(|bytes| bytes.checked_add(48))
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(empty_denial)?;
            let navigation = outer
                .checked_mul(2)
                .and_then(|work| work.checked_add(inner))
                .and_then(|work| work.checked_add(inner_moves))
                .and_then(|work| work.checked_add(outer_moves))
                .ok_or_else(empty_denial)?;
            navigate(admission, navigation)?;
            charge(admission, identity_work)?;
        }
        Ok(())
    }
}

fn comparison_work<K>(
    entries: usize,
    getter_work: u64,
) -> Result<u64, WorthQueryOutputDemandDenial> {
    // Both operands are compared. SemanticSource::cmp additionally initializes
    // its two fixed ordering tuples; four widths conservatively cover both.
    let width = u64::try_from(size_of::<K>()).map_err(|_| empty_denial())?;
    tree_lookup_work::<K>(entries)
        .and_then(|visits| visits.checked_mul(width.checked_mul(4)?.checked_add(getter_work)?))
        .ok_or_else(empty_denial)
}

fn movement_work<K, V>(entries: usize) -> Result<u64, WorthQueryOutputDemandDenial> {
    let levels = tree_level_bound(entries).ok_or_else(empty_denial)?;
    // Rust 1.94 deletion shifts a leaf and can merge/steal from two children
    // and their parent per level. Charge read/write bounds for three initialized
    // node regions, including edge moves, parent-link repair and local handles.
    // An internal removed pair also travels through a leaf and back to its slot.
    // See alloc/collections/btree/{remove,node}.rs in the Rust 1.94 source.
    let slots = entries.min(11);
    let pairs = size_of::<K>()
        .checked_add(size_of::<V>())
        .and_then(|width| width.checked_mul(slots))
        .ok_or_else(empty_denial)?;
    let links = slots
        .checked_add(1)
        .and_then(|slots| slots.checked_mul(size_of::<usize>() * 3))
        .ok_or_else(empty_denial)?;
    let per_level = pairs
        .checked_add(links)
        .and_then(|bytes| bytes.checked_add(size_of::<usize>() * 16))
        .and_then(|bytes| bytes.checked_mul(6))
        .ok_or_else(empty_denial)?;
    per_level
        .checked_mul(levels)
        .and_then(|bytes| bytes.checked_add(size_of::<(K, V)>().checked_mul(4)?))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(empty_denial)
}

fn charge(
    admission: &mut InvalidationEditAdmission,
    work: u64,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(work)
        .map_err(|_| empty_denial())
}

/// Shared-index searches and rebalancing are physical navigation, reported
/// apart from the request's declared work.
fn navigate(
    admission: &mut InvalidationEditAdmission,
    work: u64,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_ordered_operations(1, work)
        .map_err(|_| empty_denial())
}

fn empty_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(Kind::WorkBudgetExceeded, "")
}
