//! Closed canonical preparation progress, paired to original native custody.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::NativeComputationReads;
use std::{ops::Range, sync::Arc};

pub(super) struct CallRangeProgress {
    binding: CheckedCallShape,
    next: usize,
    transition: Option<
        Arc<crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition>,
    >,
}
pub(in crate::domain_computation::primary_graph) struct NativeCallCarriage {
    native: NativeComputationReads,
    binding: CheckedCallShape,
    fresh: BTreeSet<ComputationRead>,
    transition: Option<
        Arc<crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition>,
    >,
    _memory: Option<
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    >,
}
impl CallRangeProgress {
    fn shape(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape> {
        self.transition
            .as_ref()
            .map_or(&self.binding.shape, |edit| edit.target())
    }

    pub(super) fn fresh(&mut self, read: ComputationRead) {
        assert_eq!(
            self.shape().ordinal(read),
            Some(self.next),
            "fresh call occupies the next canonical position"
        );
        self.next += 1;
    }
    pub(super) fn carried<'a>(
        &'a self,
        fresh: &'a BTreeSet<ComputationRead>,
    ) -> impl Iterator<Item = ComputationRead> + 'a {
        self.shape()
            .calls()
            .take(self.next)
            .filter(|read| !fresh.contains(read))
    }
}
impl NativeCallCarriage {
    pub(in crate::domain_computation::primary_graph) fn target(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape> {
        self.transition
            .as_ref()
            .map_or(&self.binding.shape, |edit| edit.target())
    }
    pub(super) fn transition(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition>
    {
        self.transition.as_deref()
    }
    pub(super) fn changes_shape(&self) -> bool {
        self.transition.is_some() && !Arc::ptr_eq(&self.binding.shape, self.target())
    }
    pub(in crate::domain_computation::primary_graph) fn matches_target(
        &self,
        shape: &Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape>,
    ) -> bool {
        Arc::ptr_eq(self.target(), shape)
    }

    pub(in crate::domain_computation::primary_graph) fn carried(
        &self,
        read: ComputationRead,
    ) -> bool {
        self.target().ordinal(read).is_some()
            && self.binding.shape.ordinal(read).is_some()
            && !self.fresh.contains(&read)
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
            && self.transition.as_ref().is_none_or(|edit| {
                edit.matches_prior(&self.binding.shape) && edit.closes_fresh(fresh)
            })
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
        self.range_progress = Some(CallRangeProgress {
            binding,
            next: 0,
            transition: None,
        });
    }
    /// Routing may change only the not-yet-executed suffix. The completed
    /// prefix retains its exact original correspondence and native binding.
    pub(in crate::domain_computation::primary_graph) fn select_shape_transition(
        &mut self,
        transition: Arc<
            crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition,
        >,
    ) -> bool {
        let Some(progress) = self.range_progress.as_mut() else {
            return false;
        };
        if progress.transition.is_some()
            || !transition.matches_prior(&progress.binding.shape)
            || (progress.next > 0 && !transition.maps(0..progress.next, 0..progress.next))
        {
            return false;
        }
        progress.transition = Some(transition);
        true
    }
    pub(in crate::domain_computation::primary_graph) fn can_carry_mapped_range(
        &self,
        source: &Range<usize>,
        target: &Range<usize>,
    ) -> bool {
        let Some(progress) = self.range_progress.as_ref() else {
            return false;
        };
        if progress.next != target.start {
            return false;
        }
        match &progress.transition {
            Some(edit) => edit.maps(source.clone(), target.clone()),
            None => {
                source == target
                    && target.start < target.end
                    && target.end <= progress.shape().len()
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn carry_mapped_range(
        &mut self,
        source: Range<usize>,
        target: Range<usize>,
    ) -> bool {
        if !self.can_carry_mapped_range(&source, &target) {
            return false;
        }
        self.range_progress
            .as_mut()
            .expect("checked range progress")
            .next = target.end;
        true
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
        memory: Option<crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation>,
    ) -> Arc<NativeCallCarriage> {
        let progress = self
            .range_progress
            .as_ref()
            .expect("native range preparation began");
        assert_eq!(
            progress.next,
            progress.shape().len(),
            "every actual call settled preparation"
        );
        let closed = Arc::new(NativeCallCarriage {
            native: self.native.as_ref().unwrap().clone(),
            binding: progress.binding.clone(),
            fresh: self.fresh_calls.clone(),
            transition: progress.transition.clone(),
            _memory: memory,
        });
        self.range_closed = Some(Arc::clone(&closed));
        closed
    }
    pub(super) fn is_carried(&self, read: ComputationRead) -> bool {
        self.carried.contains(&read)
            || self.range_progress.as_ref().is_some_and(|progress| {
                progress
                    .shape()
                    .ordinal(read)
                    .is_some_and(|ordinal| ordinal < progress.next)
                    && !self.fresh_calls.contains(&read)
            })
    }
}
