use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use worth_ui_host_contract::UiMountedFrameIdentity;

use super::{
    UiMountedFrameRetentionBudget, UiMountedRetentionClass, UiMountedRetentionClassBudget,
    UiMountedRetentionUsageSnapshot, UiPresentedFrameBasisRelation, UiRetainedPresentedFrame,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiMountedRetentionReservationIdentity(u64);

impl UiMountedRetentionReservationIdentity {
    pub(super) fn mint() -> Option<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};

        static NEXT: AtomicU64 = AtomicU64::new(1);
        let value = NEXT.fetch_add(1, Ordering::Relaxed);
        (value != 0).then_some(Self(value))
    }
}

mod pin_accounting;
mod surface_current;

use pin_accounting::{UiMountedFramePinCounts, UiMountedPinAdmission};

#[derive(Clone, Default)]
pub(super) struct UiMountedRetainedFrameState {
    pub(super) current: Option<Rc<UiRetainedPresentedFrame>>,
    pub(super) surface_frames: crate::runtime::persistent_index::UiPersistentOrdMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        UiMountedFrameIdentity,
    >,
    pub(super) predecessors: crate::runtime::persistent_index::UiPersistentOrdMap<
        UiMountedFrameIdentity,
        Rc<UiRetainedPresentedFrame>,
    >,
    pub(super) predecessor_order: VecDeque<UiMountedFrameIdentity>,
    pub(super) predecessor_structural_bytes: usize,
    pub(super) expired:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedFrameIdentity>,
    pub(super) expiration_order: VecDeque<UiMountedFrameIdentity>,
    pub(super) diagnostics: crate::runtime::persistent_index::UiPersistentOrdMap<
        UiMountedFrameIdentity,
        Rc<super::UiRetainedMountedDiagnostics>,
    >,
    pub(super) diagnostic_order: VecDeque<UiMountedFrameIdentity>,
    pub(super) diagnostic_structural_bytes: usize,
}

pub(super) struct UiMountedFrameRetentionAuthority {
    pub(super) budget: UiMountedFrameRetentionBudget,
    pub(super) frames: UiMountedRetainedFrameState,
    pub(super) revision: u64,
    pub(super) reservations: BTreeMap<UiMountedRetentionReservationIdentity, usize>,
    pub(super) in_flight_structural_bytes: usize,
    pins: BTreeMap<UiMountedFrameIdentity, UiMountedFramePinCounts>,
    inspection_usage: UiMountedRetentionUsageSnapshot,
    observation_basis_usage: UiMountedRetentionUsageSnapshot,
    diagnostic_usage: UiMountedRetentionUsageSnapshot,
    visual_snapshot_usage: UiMountedRetentionUsageSnapshot,
    visual_overlay_usage: UiMountedRetentionUsageSnapshot,
}

pub(super) enum UiMountedRetainedFrameLookup<'a> {
    Found {
        evidence: &'a UiRetainedPresentedFrame,
        relation: UiPresentedFrameBasisRelation,
        frame_index_probes: usize,
    },
    Expired {
        frame_index_probes: usize,
    },
    Unknown {
        frame_index_probes: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiMountedRetentionPinAdmissionDenial {
    CapacityExceeded {
        required_leases: usize,
        required_structural_bytes: usize,
        budget: UiMountedRetentionClassBudget,
    },
    AccountingOverflow,
}

impl UiMountedFrameRetentionAuthority {
    pub(super) fn new(budget: UiMountedFrameRetentionBudget) -> Self {
        Self {
            budget,
            frames: Default::default(),
            revision: 0,
            reservations: BTreeMap::new(),
            in_flight_structural_bytes: 0,
            pins: BTreeMap::new(),
            inspection_usage: Default::default(),
            observation_basis_usage: Default::default(),
            diagnostic_usage: Default::default(),
            visual_snapshot_usage: Default::default(),
            visual_overlay_usage: Default::default(),
        }
    }

    pub(super) fn current_frame(&self) -> UiMountedRetainedFrameLookup<'_> {
        match self.frames.current.as_deref() {
            Some(evidence) => UiMountedRetainedFrameLookup::Found {
                evidence,
                relation: UiPresentedFrameBasisRelation::Current,
                frame_index_probes: 1,
            },
            None => UiMountedRetainedFrameLookup::Unknown {
                frame_index_probes: 1,
            },
        }
    }

    pub(super) fn frame(&self, frame: UiMountedFrameIdentity) -> UiMountedRetainedFrameLookup<'_> {
        let mut probes = 1;
        if self
            .frames
            .current
            .as_ref()
            .is_some_and(|evidence| evidence.frame() == frame)
        {
            return UiMountedRetainedFrameLookup::Found {
                evidence: self
                    .frames
                    .current
                    .as_deref()
                    .expect("the current frame matched"),
                relation: UiPresentedFrameBasisRelation::Current,
                frame_index_probes: probes,
            };
        }
        let (predecessor, predecessor_probes) = self.frames.predecessors.get_with_probes(&frame);
        probes = probes
            .checked_add(predecessor_probes)
            .expect("frame index probe accounting fits usize");
        if let Some(evidence) = predecessor {
            return UiMountedRetainedFrameLookup::Found {
                evidence,
                relation: UiPresentedFrameBasisRelation::Retained,
                frame_index_probes: probes,
            };
        }
        let (expired, expired_probes) = self.frames.expired.contains_with_probes(&frame);
        probes = probes
            .checked_add(expired_probes)
            .expect("frame index probe accounting fits usize");
        if expired {
            UiMountedRetainedFrameLookup::Expired {
                frame_index_probes: probes,
            }
        } else {
            UiMountedRetainedFrameLookup::Unknown {
                frame_index_probes: probes,
            }
        }
    }

    pub(super) fn reserve_pin(
        &mut self,
        frame: UiMountedFrameIdentity,
        class: UiMountedRetentionClass,
        structural_bytes: usize,
    ) -> Result<(), UiMountedRetentionPinAdmissionDenial> {
        let existing = self.pins.get(&frame).copied().unwrap_or_default();
        let admission = UiMountedPinAdmission::admit(
            existing.count(class),
            self.pin_usage(class),
            self.pin_budget(class),
            structural_bytes,
        )?;
        self.pins
            .entry(frame)
            .or_default()
            .set_count(class, admission.next_frame_pin_count);
        let usage = self.pin_usage_mut(class);
        usage.active_leases = admission.required_leases;
        usage.lease_charged_structural_bytes = admission.required_structural_bytes;
        Ok(())
    }

    pub(super) fn release_pin(
        &mut self,
        frame: UiMountedFrameIdentity,
        class: UiMountedRetentionClass,
        structural_bytes: usize,
    ) {
        let Some(existing) = self.pins.get(&frame).copied() else {
            debug_assert!(
                false,
                "a retention lease must release an existing frame pin"
            );
            return;
        };
        if existing.count(class) == 0 {
            debug_assert!(
                false,
                "a retention lease cannot release an absent class pin"
            );
            return;
        }
        self.release_pin_usage(class, structural_bytes);
        let pins = self
            .pins
            .get_mut(&frame)
            .expect("the frame pin was present before accounting");
        pins.decrement(class);
        if pins.is_empty() {
            self.pins.remove(&frame);
        }
    }

    pub(super) fn frame_is_pinned(&self, frame: UiMountedFrameIdentity) -> bool {
        self.pins
            .get(&frame)
            .is_some_and(|pins| pins.protects_frame())
    }

    pub(super) fn diagnostic_is_pinned(&self, frame: UiMountedFrameIdentity) -> bool {
        self.pins
            .get(&frame)
            .is_some_and(|pins| pins.protects_diagnostics())
    }

    pub(super) fn diagnostics(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Option<Rc<super::UiRetainedMountedDiagnostics>> {
        self.frames.diagnostics.get(&frame).cloned()
    }

    pub(super) fn snapshot(&self) -> super::UiMountedFrameRetentionSnapshot {
        let mut current = retained_frame_usage(self.frames.current.as_deref());
        current.retained_structural_bytes += self
            .frames
            .surface_frames
            .retained_structural_bytes()
            .expect("admitted surface index bytes fit usize");
        super::UiMountedFrameRetentionSnapshot {
            current,
            in_flight: UiMountedRetentionUsageSnapshot {
                retained_items: self.reservations.len(),
                retained_structural_bytes: self.in_flight_structural_bytes,
                active_leases: 0,
                lease_charged_structural_bytes: 0,
            },
            observation_basis: self.observation_basis_usage,
            predecessor_inspection: UiMountedRetentionUsageSnapshot {
                retained_items: self.frames.predecessors.len(),
                retained_structural_bytes: self.frames.predecessor_structural_bytes,
                active_leases: self.inspection_usage.active_leases,
                lease_charged_structural_bytes: self
                    .inspection_usage
                    .lease_charged_structural_bytes,
            },
            diagnostic: UiMountedRetentionUsageSnapshot {
                retained_items: self.frames.diagnostics.len(),
                retained_structural_bytes: self.frames.diagnostic_structural_bytes,
                active_leases: self.diagnostic_usage.active_leases,
                lease_charged_structural_bytes: self
                    .diagnostic_usage
                    .lease_charged_structural_bytes,
            },
            visual_snapshot: self.visual_snapshot_usage,
            visual_overlay: self.visual_overlay_usage,
            budget: self.budget,
        }
    }

    fn pin_usage_mut(
        &mut self,
        class: UiMountedRetentionClass,
    ) -> &mut UiMountedRetentionUsageSnapshot {
        match class {
            UiMountedRetentionClass::PredecessorInspection => &mut self.inspection_usage,
            UiMountedRetentionClass::ObservationBasis => &mut self.observation_basis_usage,
            UiMountedRetentionClass::Diagnostic => &mut self.diagnostic_usage,
            UiMountedRetentionClass::VisualSnapshot => &mut self.visual_snapshot_usage,
            UiMountedRetentionClass::VisualOverlay => &mut self.visual_overlay_usage,
            _ => unreachable!("only lease-backed retention classes own usage"),
        }
    }

    fn pin_usage(&self, class: UiMountedRetentionClass) -> UiMountedRetentionUsageSnapshot {
        match class {
            UiMountedRetentionClass::PredecessorInspection => self.inspection_usage,
            UiMountedRetentionClass::ObservationBasis => self.observation_basis_usage,
            UiMountedRetentionClass::Diagnostic => self.diagnostic_usage,
            UiMountedRetentionClass::VisualSnapshot => self.visual_snapshot_usage,
            UiMountedRetentionClass::VisualOverlay => self.visual_overlay_usage,
            _ => unreachable!("only lease-backed retention classes own usage"),
        }
    }

    fn pin_budget(&self, class: UiMountedRetentionClass) -> UiMountedRetentionClassBudget {
        match class {
            UiMountedRetentionClass::PredecessorInspection => self.budget.predecessor_inspection(),
            UiMountedRetentionClass::ObservationBasis => self.budget.observation_basis(),
            UiMountedRetentionClass::Diagnostic => self.budget.diagnostic(),
            UiMountedRetentionClass::VisualSnapshot => self.budget.visual_snapshot(),
            UiMountedRetentionClass::VisualOverlay => self.budget.visual_overlay(),
            _ => unreachable!("only lease-backed retention classes own budgets"),
        }
    }

    fn release_pin_usage(&mut self, class: UiMountedRetentionClass, structural_bytes: usize) {
        let usage = self.pin_usage_mut(class);
        usage.active_leases = usage
            .active_leases
            .checked_sub(1)
            .expect("retention lease accounting includes the released lease");
        usage.lease_charged_structural_bytes = usage
            .lease_charged_structural_bytes
            .checked_sub(structural_bytes)
            .expect("retention byte accounting includes the released lease");
    }
}

fn retained_frame_usage(
    evidence: Option<&UiRetainedPresentedFrame>,
) -> UiMountedRetentionUsageSnapshot {
    UiMountedRetentionUsageSnapshot {
        retained_items: usize::from(evidence.is_some()),
        retained_structural_bytes: evidence.map_or(0, UiRetainedPresentedFrame::structural_bytes),
        active_leases: 0,
        lease_charged_structural_bytes: 0,
    }
}
