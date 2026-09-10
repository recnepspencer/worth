use std::cell::RefCell;
use std::rc::Rc;

use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
};

use super::authority::{
    UiMountedFrameRetentionAuthority, UiMountedRetainedFrameLookup,
    UiMountedRetentionPinAdmissionDenial, UiMountedRetentionReservationIdentity,
};
use super::successor_admission::admit_successor;
use super::{
    UiMountedFrameRetentionBudget, UiMountedFrameRetentionRejection,
    UiMountedObservationBasisLease, UiMountedObservationBasisRetentionDenial,
    UiMountedRetentionClass, UiPresentedFrameBasisDenial, UiPresentedFrameBasisRelation,
    UiPresentedHitTestBasis, UiRetentionPreparedMountedFrame,
};

mod inspection;
mod presented_hits;
mod visual_lease;
pub(crate) use presented_hits::UiPresentedPointLookupDenial;

pub(crate) struct UiMountedFrameRetentionCoordinator {
    authority: Rc<RefCell<UiMountedFrameRetentionAuthority>>,
}

impl UiMountedFrameRetentionCoordinator {
    pub(crate) fn with_budget(budget: UiMountedFrameRetentionBudget) -> Self {
        Self {
            authority: Rc::new(RefCell::new(UiMountedFrameRetentionAuthority::new(budget))),
        }
    }

    pub(crate) fn prepare_publication(
        &mut self,
        admitted: super::super::UiAuthorityAdmittedMountedFrame,
    ) -> Result<UiRetentionPreparedMountedFrame, UiMountedFrameRetentionRejection> {
        self.prepare(admitted.into_frame(), false, None)
    }

    pub(crate) fn observation_basis_admission_ready(&self) -> bool {
        self.authority.borrow().reservations.is_empty()
    }

    pub(crate) fn prepare_superseding_publication(
        &mut self,
        admitted: super::super::UiAuthorityAdmittedMountedFrame,
        predecessor: UiMountedRetentionReservationIdentity,
    ) -> Result<UiRetentionPreparedMountedFrame, UiMountedFrameRetentionRejection> {
        self.prepare(admitted.into_frame(), false, Some(predecessor))
    }

    pub(crate) fn prepare_reconciliation(
        &mut self,
        admitted: super::super::UiAuthorityAdmittedMountedFrame,
    ) -> Result<UiRetentionPreparedMountedFrame, UiMountedFrameRetentionRejection> {
        self.prepare(admitted.into_frame(), true, None)
    }

    pub(crate) fn classify(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        mounted_instance: Option<UiMountedInstanceIdentity>,
        node_receipt: Option<UiMountedNodeReceiptIdentity>,
    ) -> Result<UiPresentedFrameBasisRelation, UiPresentedFrameBasisDenial> {
        let authority = self.authority.borrow();
        let (evidence, relation) = match authority.frame(presentation.frame()) {
            UiMountedRetainedFrameLookup::Found {
                evidence, relation, ..
            } => (evidence, relation),
            UiMountedRetainedFrameLookup::Expired { .. } => {
                return Err(UiPresentedFrameBasisDenial::Expired)
            }
            UiMountedRetainedFrameLookup::Unknown { .. } => {
                return Err(UiPresentedFrameBasisDenial::Unknown)
            }
        };
        evidence.classify(presentation, mounted_instance, node_receipt)?;
        Ok(relation)
    }

    pub(crate) fn update_current_presentation_epoch(
        &mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<(), UiPresentedFrameBasisDenial> {
        let mut authority = self.authority.borrow_mut();
        let evidence = authority
            .frames
            .current
            .as_mut()
            .filter(|evidence| evidence.frame() == presentation.frame())
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        Rc::make_mut(evidence).update_presentation_epoch(presentation)
    }

    pub(crate) fn interaction_hit_test_basis(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<UiPresentedHitTestBasis, UiPresentedFrameBasisDenial> {
        let authority = self.authority.borrow();
        let (evidence, relation) = match authority.frame(presentation.frame()) {
            UiMountedRetainedFrameLookup::Found {
                evidence, relation, ..
            } => (evidence, relation),
            UiMountedRetainedFrameLookup::Expired { .. } => {
                return Err(UiPresentedFrameBasisDenial::Expired)
            }
            UiMountedRetainedFrameLookup::Unknown { .. } => {
                return Err(UiPresentedFrameBasisDenial::Unknown)
            }
        };
        evidence.classify(presentation, None, None)?;
        let rows = evidence
            .visual_region_basis(presentation.binding())
            .hit_test();
        Ok(UiPresentedHitTestBasis::new(presentation, relation, rows))
    }

    pub(crate) fn current_projection_input(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<worth_ui_query_binding::UiProjectionInputFactReference> {
        let authority = self.authority.borrow();
        match authority.current_frame() {
            UiMountedRetainedFrameLookup::Found { evidence, .. } => {
                evidence.projection_input(slot).cloned()
            }
            UiMountedRetainedFrameLookup::Expired { .. }
            | UiMountedRetainedFrameLookup::Unknown { .. } => None,
        }
    }

    pub(crate) fn acquire_observation_basis(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Result<UiMountedObservationBasisLease, UiMountedObservationBasisRetentionDenial> {
        let structural_bytes = {
            let authority = self.authority.borrow();
            if !authority.reservations.is_empty() {
                return Err(UiMountedObservationBasisRetentionDenial::FrameTransitionInFlight);
            }
            match authority.frame(frame) {
                UiMountedRetainedFrameLookup::Found { evidence, .. } => evidence.structural_bytes(),
                UiMountedRetainedFrameLookup::Expired { .. } => {
                    return Err(UiMountedObservationBasisRetentionDenial::ExpiredFrame)
                }
                UiMountedRetainedFrameLookup::Unknown { .. } => {
                    return Err(UiMountedObservationBasisRetentionDenial::UnknownFrame)
                }
            }
        };
        self.authority
            .borrow_mut()
            .reserve_pin(
                frame,
                UiMountedRetentionClass::ObservationBasis,
                structural_bytes,
            )
            .map_err(map_observation_pin_denial)?;
        Ok(UiMountedObservationBasisLease::from_reserved(
            &self.authority,
            frame,
            structural_bytes,
        ))
    }

    pub(crate) fn retention_snapshot(&self) -> super::UiMountedFrameRetentionSnapshot {
        self.authority.borrow().snapshot()
    }

    fn prepare(
        &mut self,
        frame: super::super::UiPreparedMountedFrame,
        reconciliation: bool,
        superseding: Option<UiMountedRetentionReservationIdentity>,
    ) -> Result<UiRetentionPreparedMountedFrame, UiMountedFrameRetentionRejection> {
        let admission = {
            let mut authority = self.authority.borrow_mut();
            let admission = admit_successor(&authority, &frame, reconciliation, superseding);
            let identity = if let Ok(admission) = &admission {
                let identity = UiMountedRetentionReservationIdentity::mint()
                    .expect("retention reservation identity space is not exhausted");
                let replaced = authority
                    .reservations
                    .insert(identity, admission.structural_bytes());
                assert!(
                    replaced.is_none(),
                    "retention reservation identities are unique"
                );
                authority.in_flight_structural_bytes = authority
                    .in_flight_structural_bytes
                    .checked_add(admission.structural_bytes())
                    .expect("admitted in-flight retention bytes fit usize");
                Some(identity)
            } else {
                None
            };
            (admission, identity)
        };
        match admission {
            (Ok(admission), Some(identity)) => {
                let reservation = super::UiMountedRetentionReservation::new(
                    admission,
                    identity,
                    Rc::clone(&self.authority),
                );
                Ok(UiRetentionPreparedMountedFrame::new(frame, reservation))
            }
            (Err(denial), None) => Err(UiMountedFrameRetentionRejection::new(frame, denial)),
            _ => unreachable!("retention reservation identity follows successful admission"),
        }
    }
}

impl Default for UiMountedFrameRetentionCoordinator {
    fn default() -> Self {
        Self::with_budget(Default::default())
    }
}

fn map_observation_pin_denial(
    denial: UiMountedRetentionPinAdmissionDenial,
) -> UiMountedObservationBasisRetentionDenial {
    match denial {
        UiMountedRetentionPinAdmissionDenial::CapacityExceeded {
            required_leases,
            required_structural_bytes,
            budget,
        } => UiMountedObservationBasisRetentionDenial::CapacityExceeded {
            required_leases,
            required_structural_bytes,
            budget,
        },
        UiMountedRetentionPinAdmissionDenial::AccountingOverflow => {
            UiMountedObservationBasisRetentionDenial::AccountingOverflow
        }
    }
}
