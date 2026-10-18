//! Closed canonical preparation progress, paired to original native custody.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::NativeComputationReads;
use std::{ops::Range, sync::Arc};

pub(super) struct CallRangeProgress {
    binding: CheckedCallShape,
    next: usize,
}
pub(in crate::domain_computation::primary_graph) struct NativeCallCarriage {
    native: NativeComputationReads,
    binding: CheckedCallShape,
    fresh: BTreeSet<ComputationRead>,
    _memory:
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
}
impl CallRangeProgress {
    pub(super) fn fresh(&mut self, read: ComputationRead) {
        assert_eq!(
            self.binding.shape.ordinal(read),
            Some(self.next),
            "fresh call occupies the next canonical position"
        );
        self.next += 1;
    }
    pub(super) fn carried<'a>(
        &'a self,
        fresh: &'a BTreeSet<ComputationRead>,
    ) -> impl Iterator<Item = ComputationRead> + 'a {
        self.binding
            .shape
            .calls()
            .take(self.next)
            .filter(|read| !fresh.contains(read))
    }
}
impl NativeCallCarriage {
    pub(in crate::domain_computation::primary_graph) fn carried(
        &self,
        read: ComputationRead,
    ) -> bool {
        self.binding.shape.ordinal(read).is_some() && !self.fresh.contains(&read)
    }
    pub(in crate::domain_computation::primary_graph) fn same_native(
        &self,
        native: &NativeComputationReads,
    ) -> bool {
        self.native.same_native(native)
    }
    pub(super) fn closes(
        &self,
        native: &NativeComputationReads,
        fresh: &BTreeSet<ComputationRead>,
    ) -> bool {
        self.same_native(native)
            && &self.fresh == fresh
            && native.facts().has_call_shape(&self.binding)
    }
}
impl ComputationFactAttribution {
    pub(in crate::domain_computation::primary_graph) fn begin_ranges(
        &mut self,
        binding: CheckedCallShape,
    ) {
        assert!(
            self.calls.is_empty() && self.carried.is_empty() && self.fresh_calls.is_empty(),
            "range preparation starts before its calls"
        );
        assert!(
            self.native
                .as_ref()
                .is_some_and(|native| native.facts().has_call_shape(&binding)),
            "checked call shape belongs to this native prior"
        );
        self.range_progress = Some(CallRangeProgress { binding, next: 0 });
    }
    pub(in crate::domain_computation::primary_graph) fn carry_range(
        &mut self,
        range: Range<usize>,
    ) {
        let progress = self
            .range_progress
            .as_mut()
            .expect("native range preparation began");
        assert_eq!(
            progress.next, range.start,
            "carried ranges neither omit nor duplicate calls"
        );
        assert!(range.start < range.end && range.end <= progress.binding.shape.len());
        progress.next = range.end;
    }
    pub(in crate::domain_computation::primary_graph) fn range_close_bytes(&self) -> Option<u64> {
        (size_of::<NativeCallCarriage>() as u64)
            .checked_add(2 * size_of::<usize>() as u64)?
            .checked_add(
                u64::try_from(
                    self.fresh_calls
                        .len()
                        .checked_mul(11 * size_of::<ComputationRead>() + 16 * size_of::<usize>())?,
                )
                .ok()?,
            )
    }
    pub(in crate::domain_computation::primary_graph) fn close_ranges(
        &mut self,
        memory:crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    ) -> Arc<NativeCallCarriage> {
        let progress = self
            .range_progress
            .as_ref()
            .expect("native range preparation began");
        assert_eq!(
            progress.next,
            progress.binding.shape.len(),
            "every actual call settled preparation"
        );
        let closed = Arc::new(NativeCallCarriage {
            native: self.native.as_ref().unwrap().clone(),
            binding: progress.binding.clone(),
            fresh: self.fresh_calls.clone(),
            _memory: memory,
        });
        self.range_closed = Some(Arc::clone(&closed));
        closed
    }
    pub(super) fn is_carried(&self, read: ComputationRead) -> bool {
        self.carried.contains(&read)
            || self.range_progress.as_ref().is_some_and(|progress| {
                progress
                    .binding
                    .shape
                    .ordinal(read)
                    .is_some_and(|ordinal| ordinal < progress.next)
                    && !self.fresh_calls.contains(&read)
            })
    }
}
