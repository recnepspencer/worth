use std::rc::Rc;

use worth_ui_host_contract::{UiMountedFrameIdentity, UiMountedNodeReceiptIdentity};

use super::super::authority::{UiMountedRetainedFrameLookup, UiMountedRetentionPinAdmissionDenial};
use super::super::{
    UiMountedDiagnosticInspectionBasis, UiMountedDiagnosticInspectionDenial,
    UiMountedDiagnosticRetentionLease, UiMountedFrameInspectionBasis,
    UiMountedFrameInspectionDenial, UiMountedFrameInspectionSelection,
    UiMountedFrameInspectionTarget, UiMountedRetentionClass, UiMountedRetentionLease,
    UiPresentedFrameBasisRelation, UiRetainedMountedDiagnostics,
};
use super::UiMountedFrameRetentionCoordinator;

impl UiMountedFrameRetentionCoordinator {
    pub(crate) fn inspect(
        &self,
        selection: UiMountedFrameInspectionSelection,
    ) -> Result<UiMountedFrameInspectionBasis, UiMountedFrameInspectionDenial> {
        let selected = self.select_inspection_basis(selection)?;
        self.reserve_inspection_lease(selected)
    }

    fn select_inspection_basis(
        &self,
        selection: UiMountedFrameInspectionSelection,
    ) -> Result<UiSelectedMountedFrameInspection, UiMountedFrameInspectionDenial> {
        let authority = self.authority.borrow();
        if !authority.reservations.is_empty() {
            return Err(UiMountedFrameInspectionDenial::FrameTransitionInFlight);
        }
        let lookup = match selection.target {
            UiMountedFrameInspectionTarget::Current => authority.current_frame(),
            UiMountedFrameInspectionTarget::Frame(frame) => authority.frame(frame),
        };
        let (evidence, relation, frame_index_probes) = inspection_lookup(selection.target, lookup)?;
        let (selected_node_receipt, instance_index_probes) = match selection.instance {
            Some(instance) => {
                let (receipt, probes) = evidence.receipt_for_with_probes(instance);
                let receipt =
                    receipt.ok_or(UiMountedFrameInspectionDenial::InstanceNotPresented {
                        frame_index_probes,
                        instance_index_probes: probes,
                    })?;
                (Some(receipt), probes)
            }
            None => (None, 0),
        };
        Ok(UiSelectedMountedFrameInspection {
            frame: evidence.frame(),
            relation,
            presentation: evidence
                .presentation_receipt()
                .expect("published retained frames carry completed presentation truth")
                .clone(),
            presented_binding_count: evidence.presented_binding_count(),
            mounted_instance_count: evidence.mounted_instance_count(),
            selected_node_receipt,
            mount_cost: evidence.mount_cost(),
            retained_structural_bytes: evidence.structural_bytes(),
            frame_index_probes,
            instance_index_probes,
            diagnostics: if selection.diagnostics {
                UiSelectedDiagnostics::Requested(authority.diagnostics(evidence.frame()))
            } else {
                UiSelectedDiagnostics::NotRequested
            },
        })
    }

    fn reserve_inspection_lease(
        &self,
        selected: UiSelectedMountedFrameInspection,
    ) -> Result<UiMountedFrameInspectionBasis, UiMountedFrameInspectionDenial> {
        self.authority
            .borrow_mut()
            .reserve_pin(
                selected.frame,
                UiMountedRetentionClass::PredecessorInspection,
                selected.retained_structural_bytes,
            )
            .map_err(map_inspection_pin_denial)?;
        let diagnostics = self.reserve_diagnostic_lease(&selected);
        Ok(UiMountedFrameInspectionBasis {
            frame: selected.frame,
            relation: selected.relation,
            presentation: selected.presentation,
            presented_binding_count: selected.presented_binding_count,
            mounted_instance_count: selected.mounted_instance_count,
            selected_node_receipt: selected.selected_node_receipt,
            mount_cost: selected.mount_cost,
            retained_structural_bytes: selected.retained_structural_bytes,
            frame_index_probes: selected.frame_index_probes,
            instance_index_probes: selected.instance_index_probes,
            diagnostics,
            lease: UiMountedRetentionLease::from_reserved(
                &self.authority,
                selected.frame,
                selected.retained_structural_bytes,
            ),
        })
    }

    fn reserve_diagnostic_lease(
        &self,
        selected: &UiSelectedMountedFrameInspection,
    ) -> UiMountedDiagnosticInspectionBasis {
        let evidence = match &selected.diagnostics {
            UiSelectedDiagnostics::NotRequested => {
                return UiMountedDiagnosticInspectionBasis::NotRequested;
            }
            UiSelectedDiagnostics::Requested(None) => {
                return UiMountedDiagnosticInspectionBasis::Omitted(
                    UiMountedDiagnosticInspectionDenial::NotRetained,
                );
            }
            UiSelectedDiagnostics::Requested(Some(evidence)) => evidence.clone(),
        };
        let structural_bytes = evidence.structural_bytes();
        match self.authority.borrow_mut().reserve_pin(
            selected.frame,
            UiMountedRetentionClass::Diagnostic,
            structural_bytes,
        ) {
            Ok(()) => UiMountedDiagnosticInspectionBasis::Available {
                evidence,
                lease: UiMountedDiagnosticRetentionLease::from_reserved(
                    &self.authority,
                    selected.frame,
                    structural_bytes,
                ),
            },
            Err(denial) => {
                UiMountedDiagnosticInspectionBasis::Omitted(map_diagnostic_pin_denial(denial))
            }
        }
    }
}

struct UiSelectedMountedFrameInspection {
    frame: UiMountedFrameIdentity,
    relation: UiPresentedFrameBasisRelation,
    presentation: crate::mounting::UiMountedPresentationReceipt,
    presented_binding_count: usize,
    mounted_instance_count: usize,
    selected_node_receipt: Option<UiMountedNodeReceiptIdentity>,
    mount_cost: crate::mounting::UiMountCostReport,
    retained_structural_bytes: usize,
    frame_index_probes: usize,
    instance_index_probes: usize,
    diagnostics: UiSelectedDiagnostics,
}

/// Diagnostics the inspection asked for, and what the authority still retains
/// for the selected frame when it did.
enum UiSelectedDiagnostics {
    NotRequested,
    Requested(Option<Rc<UiRetainedMountedDiagnostics>>),
}

fn inspection_lookup<'a>(
    target: UiMountedFrameInspectionTarget,
    lookup: UiMountedRetainedFrameLookup<'a>,
) -> Result<
    (
        &'a super::super::UiRetainedPresentedFrame,
        UiPresentedFrameBasisRelation,
        usize,
    ),
    UiMountedFrameInspectionDenial,
> {
    match lookup {
        UiMountedRetainedFrameLookup::Found {
            evidence,
            relation,
            frame_index_probes,
        } => Ok((evidence, relation, frame_index_probes)),
        UiMountedRetainedFrameLookup::Expired { frame_index_probes } => {
            Err(UiMountedFrameInspectionDenial::ExpiredFrame { frame_index_probes })
        }
        UiMountedRetainedFrameLookup::Unknown { frame_index_probes } => match target {
            UiMountedFrameInspectionTarget::Current => {
                Err(UiMountedFrameInspectionDenial::NoCurrentFrame)
            }
            UiMountedFrameInspectionTarget::Frame(_) => {
                Err(UiMountedFrameInspectionDenial::UnknownFrame { frame_index_probes })
            }
        },
    }
}

fn map_inspection_pin_denial(
    denial: UiMountedRetentionPinAdmissionDenial,
) -> UiMountedFrameInspectionDenial {
    match denial {
        UiMountedRetentionPinAdmissionDenial::CapacityExceeded {
            required_leases,
            required_structural_bytes,
            budget,
        } => UiMountedFrameInspectionDenial::CapacityExceeded {
            required_leases,
            required_structural_bytes,
            budget,
        },
        UiMountedRetentionPinAdmissionDenial::AccountingOverflow => {
            UiMountedFrameInspectionDenial::AccountingOverflow
        }
    }
}

fn map_diagnostic_pin_denial(
    denial: UiMountedRetentionPinAdmissionDenial,
) -> UiMountedDiagnosticInspectionDenial {
    match denial {
        UiMountedRetentionPinAdmissionDenial::CapacityExceeded {
            required_leases,
            required_structural_bytes,
            budget,
        } => UiMountedDiagnosticInspectionDenial::CapacityExceeded {
            required_leases,
            required_structural_bytes,
            budget,
        },
        UiMountedRetentionPinAdmissionDenial::AccountingOverflow => {
            UiMountedDiagnosticInspectionDenial::AccountingOverflow
        }
    }
}
