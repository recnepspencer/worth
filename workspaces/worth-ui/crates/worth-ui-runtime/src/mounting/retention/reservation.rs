use std::cell::RefCell;
use std::rc::Rc;

use super::authority::{
    UiMountedFrameRetentionAuthority, UiMountedRetainedFrameState,
    UiMountedRetentionReservationIdentity,
};
use super::successor_admission::UiMountedSuccessorRetentionAdmission;

pub(crate) struct UiRetentionPreparedMountedFrame {
    frame: super::super::UiPreparedMountedFrame,
    reservation: UiMountedRetentionReservation,
}

pub(crate) struct UiMountedRetentionReservation {
    successor: UiMountedRetainedFrameState,
    expected_revision: u64,
    successor_revision: u64,
    structural_bytes: usize,
    identity: UiMountedRetentionReservationIdentity,
    authority: Rc<RefCell<UiMountedFrameRetentionAuthority>>,
    release_on_drop: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedRetentionCommitDenial {
    RevisionChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedRetentionRefreshDenial {
    RevisionChanged,
    Capacity(super::UiMountedFrameRetentionDenial),
}

impl UiRetentionPreparedMountedFrame {
    pub(super) fn new(
        frame: super::super::UiPreparedMountedFrame,
        reservation: UiMountedRetentionReservation,
    ) -> Self {
        Self { frame, reservation }
    }

    pub(crate) fn frame(&self) -> &super::super::UiPreparedMountedFrame {
        &self.frame
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::super::UiPreparedMountedFrame,
        UiMountedRetentionReservation,
    ) {
        (self.frame, self.reservation)
    }
}

impl UiMountedRetentionReservation {
    pub(super) fn new(
        admission: UiMountedSuccessorRetentionAdmission,
        identity: UiMountedRetentionReservationIdentity,
        authority: Rc<RefCell<UiMountedFrameRetentionAuthority>>,
    ) -> Self {
        let structural_bytes = admission.structural_bytes();
        let (successor, expected_revision, successor_revision) = admission.into_parts();
        Self {
            successor,
            expected_revision,
            successor_revision,
            structural_bytes,
            identity,
            authority,
            release_on_drop: true,
        }
    }

    pub(crate) fn commit(
        mut self,
        mount_cost: super::super::UiMountCostReport,
        presentation: super::super::UiMountedPresentationReceipt,
    ) -> Result<(), UiMountedRetentionCommitDenial> {
        let mut authority = self.authority.borrow_mut();
        if authority.revision != self.expected_revision {
            drop(authority);
            return Err(UiMountedRetentionCommitDenial::RevisionChanged);
        }
        if let Some(current) = self.successor.current.as_mut() {
            let current = Rc::make_mut(current);
            assert_eq!(
                current.presented_binding_count(),
                presentation.surfaces().len(),
                "settlement completed every admitted surface"
            );
            if let Some(previous) = authority
                .frames
                .current
                .as_ref()
                .filter(|previous| previous.frame() == current.frame())
            {
                current.preserve_reconciled_presentations(previous);
            }
            for surface in presentation.surfaces() {
                assert_eq!(
                    self.successor
                        .surface_frames
                        .get(&surface.semantic_surface()),
                    Some(&current.frame()),
                    "accepted surface ownership was reserved before host effects"
                );
            }
            current.set_mount_cost(mount_cost);
            current.set_presentation_receipt(presentation);
        }
        authority.frames = std::mem::take(&mut self.successor);
        authority.revision = self.successor_revision;
        release_reservation(&mut authority, self.identity, self.structural_bytes);
        self.release_on_drop = false;
        Ok(())
    }

    pub(crate) fn refresh_visual_regions(
        &mut self,
        mut visual_regions: super::super::UiMountedVisualRegionBasis,
        surfaces: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
    ) -> Result<crate::mounting::UiHitTestSpatialWork, UiMountedRetentionRefreshDenial> {
        let mut hit_work = crate::mounting::UiHitTestSpatialWork::default();
        {
            let authority = self.authority.borrow();
            if authority.revision != self.expected_revision {
                return Err(UiMountedRetentionRefreshDenial::RevisionChanged);
            }
            for surface in surfaces {
                if let Some(previous) = authority.surface_evidence(surface.semantic_surface()) {
                    hit_work.merge(
                        visual_regions
                            .presented_hits
                            .inherit_accepted_scroll(&previous.hit_index(), surface.binding()),
                    );
                }
            }
        }
        let candidate = self
            .successor
            .current
            .as_mut()
            .and_then(Rc::get_mut)
            .ok_or(UiMountedRetentionRefreshDenial::RevisionChanged)?;
        candidate.replace_visual_regions(visual_regions).ok_or(
            UiMountedRetentionRefreshDenial::Capacity(
                super::UiMountedFrameRetentionDenial::AccountingOverflow {
                    class: super::UiMountedRetentionClass::Current,
                },
            ),
        )?;
        let replacement_bytes = candidate
            .structural_bytes()
            .checked_add(
                self.successor
                    .surface_frames
                    .retained_structural_bytes()
                    .ok_or(UiMountedRetentionRefreshDenial::RevisionChanged)?,
            )
            .ok_or(UiMountedRetentionRefreshDenial::RevisionChanged)?;
        let mut authority = self.authority.borrow_mut();
        if authority.revision != self.expected_revision
            || authority.reservations.get(&self.identity) != Some(&self.structural_bytes)
        {
            return Err(UiMountedRetentionRefreshDenial::RevisionChanged);
        }
        if !authority.budget.current().admits(1, replacement_bytes) {
            return Err(UiMountedRetentionRefreshDenial::Capacity(
                super::UiMountedFrameRetentionDenial::CapacityExceeded {
                    class: super::UiMountedRetentionClass::Current,
                    required_frames: 1,
                    required_structural_bytes: replacement_bytes,
                    budget: authority.budget.current(),
                },
            ));
        }
        let in_flight_bytes = authority
            .in_flight_structural_bytes
            .checked_sub(self.structural_bytes)
            .and_then(|bytes| bytes.checked_add(replacement_bytes))
            .ok_or(UiMountedRetentionRefreshDenial::Capacity(
                super::UiMountedFrameRetentionDenial::AccountingOverflow {
                    class: super::UiMountedRetentionClass::InFlight,
                },
            ))?;
        if !authority
            .budget
            .in_flight()
            .admits(authority.reservations.len(), in_flight_bytes)
        {
            return Err(UiMountedRetentionRefreshDenial::Capacity(
                super::UiMountedFrameRetentionDenial::CapacityExceeded {
                    class: super::UiMountedRetentionClass::InFlight,
                    required_frames: authority.reservations.len(),
                    required_structural_bytes: in_flight_bytes,
                    budget: authority.budget.in_flight(),
                },
            ));
        }
        authority
            .reservations
            .insert(self.identity, replacement_bytes);
        authority.in_flight_structural_bytes = in_flight_bytes;
        self.structural_bytes = replacement_bytes;
        Ok(hit_work)
    }

    pub(crate) const fn identity(&self) -> UiMountedRetentionReservationIdentity {
        self.identity
    }
}

impl Drop for UiMountedRetentionReservation {
    fn drop(&mut self) {
        if self.release_on_drop {
            let mut authority = self.authority.borrow_mut();
            release_reservation(&mut authority, self.identity, self.structural_bytes);
        }
    }
}

fn release_reservation(
    authority: &mut UiMountedFrameRetentionAuthority,
    identity: UiMountedRetentionReservationIdentity,
    structural_bytes: usize,
) {
    let removed = authority
        .reservations
        .remove(&identity)
        .expect("retention authority includes the released reservation");
    assert_eq!(removed, structural_bytes, "reservation bytes remain exact");
    authority.in_flight_structural_bytes = authority
        .in_flight_structural_bytes
        .checked_sub(structural_bytes)
        .expect("retention reservation bytes include the released reservation");
}
