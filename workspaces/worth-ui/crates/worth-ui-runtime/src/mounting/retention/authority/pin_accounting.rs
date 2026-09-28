use super::UiMountedRetentionPinAdmissionDenial;
use crate::mounting::{
    UiMountedRetentionClass, UiMountedRetentionClassBudget, UiMountedRetentionUsageSnapshot,
};

#[derive(Clone, Copy, Default)]
pub(super) struct UiMountedFramePinCounts {
    inspection: usize,
    observation_basis: usize,
    diagnostic: usize,
    visual_snapshot: usize,
    visual_overlay: usize,
}

pub(super) struct UiMountedPinAdmission {
    pub(super) next_frame_pin_count: usize,
    pub(super) required_leases: usize,
    pub(super) required_structural_bytes: usize,
}

impl UiMountedPinAdmission {
    pub(super) fn admit(
        frame_pin_count: usize,
        usage: UiMountedRetentionUsageSnapshot,
        budget: UiMountedRetentionClassBudget,
        structural_bytes: usize,
    ) -> Result<Self, UiMountedRetentionPinAdmissionDenial> {
        let next_frame_pin_count = checked_increment(frame_pin_count)?;
        let required_leases = checked_increment(usage.active_leases)?;
        let required_structural_bytes = usage
            .lease_charged_structural_bytes
            .checked_add(structural_bytes)
            .ok_or(UiMountedRetentionPinAdmissionDenial::AccountingOverflow)?;
        if !budget.admits(required_leases, required_structural_bytes) {
            return Err(UiMountedRetentionPinAdmissionDenial::CapacityExceeded {
                required_leases,
                required_structural_bytes,
                budget,
            });
        }
        Ok(Self {
            next_frame_pin_count,
            required_leases,
            required_structural_bytes,
        })
    }
}

impl UiMountedFramePinCounts {
    pub(super) fn count(self, class: UiMountedRetentionClass) -> usize {
        match class {
            UiMountedRetentionClass::PredecessorInspection => self.inspection,
            UiMountedRetentionClass::ObservationBasis => self.observation_basis,
            UiMountedRetentionClass::Diagnostic => self.diagnostic,
            UiMountedRetentionClass::VisualSnapshot => self.visual_snapshot,
            UiMountedRetentionClass::VisualOverlay => self.visual_overlay,
            UiMountedRetentionClass::Current
            | UiMountedRetentionClass::InFlight
            | UiMountedRetentionClass::Quarantine => {
                unreachable!("only lease-backed retention classes own counts")
            }
        }
    }

    pub(super) fn set_count(&mut self, class: UiMountedRetentionClass, count: usize) {
        match class {
            UiMountedRetentionClass::PredecessorInspection => self.inspection = count,
            UiMountedRetentionClass::ObservationBasis => self.observation_basis = count,
            UiMountedRetentionClass::Diagnostic => self.diagnostic = count,
            UiMountedRetentionClass::VisualSnapshot => self.visual_snapshot = count,
            UiMountedRetentionClass::VisualOverlay => self.visual_overlay = count,
            UiMountedRetentionClass::Current
            | UiMountedRetentionClass::InFlight
            | UiMountedRetentionClass::Quarantine => {
                unreachable!("only lease-backed retention classes own counts")
            }
        }
    }

    pub(super) fn decrement(&mut self, class: UiMountedRetentionClass) {
        self.set_count(
            class,
            self.count(class)
                .checked_sub(1)
                .expect("released pin count is nonzero"),
        );
    }

    pub(super) fn is_empty(self) -> bool {
        self.inspection == 0
            && self.observation_basis == 0
            && self.diagnostic == 0
            && self.visual_snapshot == 0
            && self.visual_overlay == 0
    }

    pub(super) fn protects_frame(self) -> bool {
        self.inspection > 0
            || self.observation_basis > 0
            || self.visual_snapshot > 0
            || self.visual_overlay > 0
    }

    pub(super) fn protects_diagnostics(self) -> bool {
        self.diagnostic > 0
    }
}

fn checked_increment(value: usize) -> Result<usize, UiMountedRetentionPinAdmissionDenial> {
    value
        .checked_add(1)
        .ok_or(UiMountedRetentionPinAdmissionDenial::AccountingOverflow)
}
