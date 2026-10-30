use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use worth_ui_host_contract::UiMountedFrameIdentity;

use super::authority::{UiMountedFrameRetentionAuthority, UiMountedRetainedFrameLookup};
use super::{UiMountedRetentionClass, UiPresentedFrameBasisRelation};

pub struct UiMountedRetentionLease {
    pin: UiMountedRetentionPin,
}

pub(crate) struct UiMountedDiagnosticRetentionLease {
    _pin: UiMountedRetentionPin,
}

pub(crate) trait UiMountedVisualLeaseClass {
    const CLASS: UiMountedRetentionClass;
}

pub(crate) struct UiMountedVisualSnapshotClass;
pub(crate) struct UiMountedVisualOverlayClass;

pub(crate) struct UiMountedVisualLease<Class: UiMountedVisualLeaseClass> {
    pin: UiMountedRetentionPin,
    _class: PhantomData<Class>,
}

pub(crate) type UiMountedVisualSnapshotLease = UiMountedVisualLease<UiMountedVisualSnapshotClass>;
pub(crate) type UiMountedVisualOverlayLease = UiMountedVisualLease<UiMountedVisualOverlayClass>;

#[derive(Clone)]
pub(crate) struct UiMountedObservationBasisLease {
    _pin: UiMountedRetentionPin,
}

#[derive(Clone)]
struct UiMountedRetentionPin {
    release: Rc<UiMountedRetentionPinRelease>,
}

struct UiMountedRetentionPinRelease {
    authority: Weak<RefCell<UiMountedFrameRetentionAuthority>>,
    frame: UiMountedFrameIdentity,
    class: UiMountedRetentionClass,
    structural_bytes: usize,
}

impl UiMountedRetentionLease {
    pub(super) fn from_reserved(
        authority: &Rc<RefCell<UiMountedFrameRetentionAuthority>>,
        frame: UiMountedFrameIdentity,
        structural_bytes: usize,
    ) -> Self {
        Self {
            pin: UiMountedRetentionPin::new(
                authority,
                frame,
                UiMountedRetentionClass::PredecessorInspection,
                structural_bytes,
            ),
        }
    }

    pub fn frame(&self) -> UiMountedFrameIdentity {
        self.pin.release.frame
    }

    pub fn class(&self) -> UiMountedRetentionClass {
        self.pin.release.class
    }
}

impl UiMountedObservationBasisLease {
    pub(super) fn from_reserved(
        authority: &Rc<RefCell<UiMountedFrameRetentionAuthority>>,
        frame: UiMountedFrameIdentity,
        structural_bytes: usize,
    ) -> Self {
        Self {
            _pin: UiMountedRetentionPin::new(
                authority,
                frame,
                UiMountedRetentionClass::ObservationBasis,
                structural_bytes,
            ),
        }
    }
}

impl UiMountedDiagnosticRetentionLease {
    pub(super) fn from_reserved(
        authority: &Rc<RefCell<UiMountedFrameRetentionAuthority>>,
        frame: UiMountedFrameIdentity,
        structural_bytes: usize,
    ) -> Self {
        Self {
            _pin: UiMountedRetentionPin::new(
                authority,
                frame,
                UiMountedRetentionClass::Diagnostic,
                structural_bytes,
            ),
        }
    }
}

impl<Class: UiMountedVisualLeaseClass> UiMountedVisualLease<Class> {
    pub(super) fn from_reserved(
        authority: &Rc<RefCell<UiMountedFrameRetentionAuthority>>,
        frame: UiMountedFrameIdentity,
        structural_bytes: usize,
    ) -> Self {
        Self {
            pin: UiMountedRetentionPin::new(authority, frame, Class::CLASS, structural_bytes),
            _class: PhantomData,
        }
    }

    pub(crate) fn structural_bytes(&self) -> usize {
        self.pin.release.structural_bytes
    }

    pub(crate) fn relation(
        &self,
    ) -> Result<worth_ui_inspection::UiVisualSnapshotRelation, super::UiMountedVisualRetentionDenial>
    {
        let authority = self
            .pin
            .release
            .authority
            .upgrade()
            .ok_or(super::UiMountedVisualRetentionDenial::ExpiredFrame)?;

        match authority.borrow().frame(self.pin.release.frame) {
            UiMountedRetainedFrameLookup::Found { relation, .. } => match relation {
                UiPresentedFrameBasisRelation::Current => {
                    Ok(worth_ui_inspection::UiVisualSnapshotRelation::Current)
                }
                UiPresentedFrameBasisRelation::Retained => {
                    Ok(worth_ui_inspection::UiVisualSnapshotRelation::RetainedPredecessor)
                }
            },
            UiMountedRetainedFrameLookup::Expired { .. } => {
                Err(super::UiMountedVisualRetentionDenial::ExpiredFrame)
            }
            UiMountedRetainedFrameLookup::Unknown { .. } => {
                Err(super::UiMountedVisualRetentionDenial::UnknownFrame)
            }
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn frame(&self) -> UiMountedFrameIdentity {
        self.pin.release.frame
    }
}

impl UiMountedVisualLease<UiMountedVisualSnapshotClass> {
    pub(crate) fn derive_overlay(
        &self,
    ) -> Result<UiMountedVisualOverlayLease, super::UiMountedVisualRetentionDenial> {
        let authority = self
            .pin
            .release
            .authority
            .upgrade()
            .ok_or(super::UiMountedVisualRetentionDenial::ExpiredFrame)?;
        let frame = self.pin.release.frame;
        let structural_bytes = self.pin.release.structural_bytes;
        authority
            .borrow_mut()
            .reserve_pin(
                frame,
                UiMountedRetentionClass::VisualOverlay,
                structural_bytes,
            )
            .map_err(|denial| match denial {
                super::authority::UiMountedRetentionPinAdmissionDenial::CapacityExceeded {
                    required_leases,
                    required_structural_bytes,
                    budget,
                } => super::UiMountedVisualRetentionDenial::CapacityExceeded {
                    class: UiMountedRetentionClass::VisualOverlay,
                    required_leases,
                    required_structural_bytes,
                    budget,
                },
                super::authority::UiMountedRetentionPinAdmissionDenial::AccountingOverflow => {
                    super::UiMountedVisualRetentionDenial::AccountingOverflow {
                        class: UiMountedRetentionClass::VisualOverlay,
                    }
                }
            })?;
        Ok(UiMountedVisualOverlayLease::from_reserved(
            &authority,
            frame,
            structural_bytes,
        ))
    }
}

impl UiMountedVisualLeaseClass for UiMountedVisualSnapshotClass {
    const CLASS: UiMountedRetentionClass = UiMountedRetentionClass::VisualSnapshot;
}

impl UiMountedVisualLeaseClass for UiMountedVisualOverlayClass {
    const CLASS: UiMountedRetentionClass = UiMountedRetentionClass::VisualOverlay;
}

impl UiMountedRetentionPin {
    fn new(
        authority: &Rc<RefCell<UiMountedFrameRetentionAuthority>>,
        frame: UiMountedFrameIdentity,
        class: UiMountedRetentionClass,
        structural_bytes: usize,
    ) -> Self {
        Self {
            release: Rc::new(UiMountedRetentionPinRelease {
                authority: Rc::downgrade(authority),
                frame,
                class,
                structural_bytes,
            }),
        }
    }
}

impl Drop for UiMountedRetentionPinRelease {
    fn drop(&mut self) {
        let Some(authority) = self.authority.upgrade() else {
            return;
        };
        authority
            .borrow_mut()
            .release_pin(self.frame, self.class, self.structural_bytes);
    }
}
