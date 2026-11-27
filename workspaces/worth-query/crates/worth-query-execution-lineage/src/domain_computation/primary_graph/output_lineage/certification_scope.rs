//! One certification's closure memo across nested `FamilySelection`
//! re-resolutions (public #429, plan §3.4 "Closure memo").
//!
//! A chain of `current_output` readers re-resolves each producer inside the
//! walk of its consumer. Without a memo, every reader's walk compares its
//! producer's closure again, which is exponential in the chain depth. A
//! top-level verification walk opens a frame with [`CertificationScope::open`];
//! each `FamilySelection` fact joins it with [`CertificationScope::enter`],
//! or opens one when no walk did. The frame keeps the completed `Current`
//! walks of terminal rows with an output projection, keyed by the terminal
//! identity the image resolves, with the rows each walk read.
//!
//! Each fact reads the image afresh. The memo is kept while every row a
//! memoized walk read keeps its meaning in the fresh image. Otherwise, between
//! two top-level facts, it is cleared; inside an outer fact's walk, the fact
//! stops with the fence's stop, since a row that walk read has changed.
//! A hit adds its rows to the walk that takes it, so that walk's own fence
//! covers the reused verdict. The armed shadow runs
//! [`CertificationScope::isolated`], so it never takes a verdict the optimized
//! path computed.
//!
//! Only the equal-chain walk is memoized. Without an image the candidate is
//! compared in full each time; that path has no terminal resolution to key on.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::invalidation::{EqualityRecoveryImage, ObservedRows, SettlementVerificationStop};
use super::family_selection::reresolution_observation::{
    self, WorthQueryFamilySelectionCostForTest,
};
use super::{CertificationBasis, ProductCoordinate, RecordedSettlementIdentity};
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::{
    map_verification_stop, ConsumedOutputVerificationStop,
};
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;

struct Frame {
    selected: PositionedRelationalSnapshot,
    coordinate: ProductCoordinate,
    /// The image the current fact reads; `None` before the first fact, or
    /// when no image covers `selected`.
    image: Option<EqualityRecoveryImage>,
    memo: BTreeMap<Arc<RecordedSettlementIdentity>, ObservedRows>,
    /// Every row a memoized walk read.
    observed: ObservedRows,
    members: usize,
    /// The `FamilySelection` facts in progress.
    facts: usize,
    /// The re-resolution cost counted when the frame opened.
    cost_from: Option<WorthQueryFamilySelectionCostForTest>,
}

impl Frame {
    fn answers(&self, coordinate: ProductCoordinate, selected: &PositionedRelationalSnapshot) -> bool {
        self.coordinate == coordinate && self.selected == *selected
    }

    /// The memo serves a walk in `image` when both resolve terminals
    /// through the same equal links.
    fn serves(&self, image: &EqualityRecoveryImage) -> bool {
        self.image.as_ref().is_some_and(|held| held.same_links(image))
    }

    fn clear(&mut self) {
        self.memo.clear();
        self.observed = ObservedRows::new();
    }
}

thread_local! {
    static FRAMES: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) };
}

/// One walk's or one fact's membership in the frame; the frame is dropped
/// when its last member ends.
pub struct CertificationScope {
    fact: bool,
    _thread_bound: PhantomData<*const ()>,
}

impl Drop for CertificationScope {
    fn drop(&mut self) {
        FRAMES.with_borrow_mut(|frames| {
            if let Some(frame) = frames.last_mut() {
                frame.members -= 1;
                if self.fact {
                    frame.facts -= 1;
                }
                if frame.members == 0 {
                    if let Some(frame) = frames.pop() {
                        reresolution_observation::record_frame(frame.cost_from);
                    }
                }
            }
        });
    }
}

fn changed() -> ConsumedOutputVerificationStop {
    map_verification_stop(SettlementVerificationStop::Edit(
        CompanionCellEditStop::TopologyGenerationChanged,
    ))
}

impl CertificationScope {
    fn member(fact: bool) -> Self {
        Self { fact, _thread_bound: PhantomData }
    }

    /// Open the frame of one top-level verification walk at `basis`, or join
    /// the frame it runs in. It reads no image; the first fact does.
    pub fn open(basis: CertificationBasis<'_>, selected: &PositionedRelationalSnapshot) -> Self {
        let coordinate = basis.coordinate();
        FRAMES.with_borrow_mut(|frames| {
            match frames.last_mut().filter(|frame| frame.answers(coordinate, selected)) {
                Some(frame) => frame.members += 1,
                None => frames.push(Frame {
                    selected: selected.clone(),
                    coordinate,
                    image: None,
                    memo: BTreeMap::new(),
                    observed: ObservedRows::new(),
                    members: 1,
                    facts: 0,
                    cost_from: reresolution_observation::frame_start(),
                }),
            }
        });
        Self::member(false)
    }

    /// Enter the frame for one `FamilySelection` fact at `basis` and return
    /// the image its candidates are compared in (`None` when none covers
    /// `selected`).
    pub fn enter(
        owner: &SourceInvalidationOwner,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<(Self, Option<EqualityRecoveryImage>), ConsumedOutputVerificationStop> {
        let fresh = ConsumedOutputEvidence::mark_free_image(owner, selected)?;
        let coordinate = basis.coordinate();
        FRAMES.with_borrow_mut(|frames| {
            match frames.last_mut().filter(|frame| frame.answers(coordinate, selected)) {
                Some(frame) => {
                    if let Some(held) = &frame.image {
                        let kept = fresh
                            .as_ref()
                            .is_some_and(|fresh| held.keeps_meaning(fresh, &frame.observed));
                        if !kept {
                            if frame.facts > 0 {
                                return Err(changed());
                            }
                            frame.clear();
                        }
                    }
                    frame.image.clone_from(&fresh);
                    frame.members += 1;
                    frame.facts += 1;
                }
                None => frames.push(Frame {
                    selected: selected.clone(),
                    coordinate,
                    image: fresh.clone(),
                    memo: BTreeMap::new(),
                    observed: ObservedRows::new(),
                    members: 1,
                    facts: 1,
                    cost_from: reresolution_observation::frame_start(),
                }),
            }
            Ok(())
        })?;
        Ok((Self::member(true), fresh))
    }

    /// The rows a completed `Current` walk of `terminal` read, when the
    /// current frame serves a walk in `image`.
    pub fn memo_hit(
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        image: &EqualityRecoveryImage,
        terminal: &Arc<RecordedSettlementIdentity>,
    ) -> Option<ObservedRows> {
        FRAMES.with_borrow(|frames| {
            let frame = frames.last()?;
            (frame.answers(basis.coordinate(), selected) && frame.serves(image))
                .then(|| frame.memo.get(terminal).cloned())
                .flatten()
        })
    }

    /// Record that the walk of `terminal`, a row with an output projection,
    /// completed `Current` after reading `observed`.
    pub fn memo_record(
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        image: &EqualityRecoveryImage,
        terminal: Arc<RecordedSettlementIdentity>,
        observed: &ObservedRows,
    ) {
        FRAMES.with_borrow_mut(|frames| {
            if let Some(frame) = frames
                .last_mut()
                .filter(|frame| frame.answers(basis.coordinate(), selected) && frame.serves(image))
            {
                frame.observed = std::mem::take(&mut frame.observed).union(observed.clone());
                frame.memo.insert(terminal, observed.clone());
            }
        });
    }

    /// Run `walk` with no frame visible, as the shadow's own scope.
    pub fn isolated<R>(walk: impl FnOnce() -> R) -> R {
        struct Restore(Vec<Frame>);
        impl Drop for Restore {
            fn drop(&mut self) {
                let frames = std::mem::take(&mut self.0);
                FRAMES.with_borrow_mut(|current| *current = frames);
            }
        }
        let _restore = Restore(FRAMES.with_borrow_mut(std::mem::take));
        walk()
    }
}
