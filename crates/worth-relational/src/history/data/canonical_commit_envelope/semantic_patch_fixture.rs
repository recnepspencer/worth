//! Keep one hostile patch fixture consistent at the earlier semantic boundary.
use super::CanonicalCommitEnvelope;
use crate::transactions::data::RecordRef;
use std::{alloc::Layout, sync::Arc};
use worth_execution::ExecutionRequest;

impl CanonicalCommitEnvelope {
    #[cfg(test)]
    pub(crate) fn synchronize_patch_semantics_for_test(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) {
        request
            .check_live()
            .expect("the original fixture request is live");
        let patches = &self.body.patch.authoritative_record_patches;
        let semantic = Arc::get_mut(&mut self.semantic_result)
            .expect("the original-request semantic copy is actually unique");
        let allocation_bytes = |count| {
            u64::try_from(
                Layout::array::<RecordRef>(count)
                    .expect("fixture roster layout")
                    .size(),
            )
            .expect("fixture roster bytes fit")
        };
        let new_bytes = allocation_bytes(patches.len());
        let old_bytes = allocation_bytes(semantic.changed_records.capacity());
        let prior = semantic
            .decoded_memory
            .as_ref()
            .map(|memory| memory.bytes());
        let final_bytes = prior.map(|prior| {
            let peak = prior
                .checked_add(new_bytes)
                .expect("fixture roster peak fits");
            let retained = peak
                .checked_sub(old_bytes)
                .expect("original roster was funded");
            semantic
                .decoded_memory
                .as_mut()
                .unwrap()
                .grow_for_allocation(peak)
                .expect("original request admits the new roster before growth");
            retained
        });
        // None follows the original unaccounted fixture policy. This helper is
        // not a positive funding witness; accounted copies keep their same bill.
        let mut changed = Vec::with_capacity(patches.len());
        changed.extend(patches.iter().map(|patch| patch.target.clone()));
        changed.sort_unstable();
        changed.dedup();
        let old = std::mem::replace(&mut semantic.changed_records, changed);
        semantic.change.changed_record_count = semantic.changed_records.len();
        semantic.publication.patch_record_count = patches.len();
        drop(old);
        if let Some(retained) = final_bytes {
            semantic
                .decoded_memory
                .as_mut()
                .unwrap()
                .resize(retained)
                .expect("releasing the replaced roster cannot exhaust memory");
        }
    }
}
