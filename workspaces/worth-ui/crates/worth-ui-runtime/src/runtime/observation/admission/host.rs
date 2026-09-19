use std::collections::BTreeSet;
use std::sync::Arc;

use worth_ui_host_contract::UiHostObservationFamily as MechanicalFamily;

use super::super::progress::UiObservationProgress;
use super::super::turn::{
    UiAdmittedObservation, UiAdmittedObservationPayload, UiAdmittedObservationSeal,
    UiObservationAdmissionDenial, UiObservationAdmissionReceipt, UiObservationTurn,
};
use super::super::UiObservationFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationSuccessorOwner {
    Intent,
    PointerPresence,
    Focus,
    Scroll,
    PresentationSampling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostObservationUnavailable {
    family: MechanicalFamily,
    successor: UiHostObservationSuccessorOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiHostObservationBatchAdmissionReceipt {
    admitted: Box<[UiObservationAdmissionReceipt]>,
    unavailable: Box<[UiHostObservationUnavailable]>,
}

#[derive(Debug)]
pub enum UiHostObservationAdmissionStop {
    Observation(UiObservationAdmissionDenial),
    RequiredFamilyUnavailable(Box<[UiHostObservationUnavailable]>),
}

pub struct UiHostObservation {
    batch: Arc<crate::facade::observation_report::UiValidatedHostObservationBatch>,
    family: MechanicalFamily,
    report_index: Option<usize>,
}

impl UiObservationTurn<'_> {
    pub fn admit_host(
        &mut self,
        batch: crate::facade::observation_report::UiValidatedHostObservationBatch,
    ) -> Result<UiHostObservationBatchAdmissionReceipt, UiHostObservationAdmissionStop> {
        self.admit_host_with_requirement(batch, false)
    }

    pub fn admit_required_host(
        &mut self,
        batch: crate::facade::observation_report::UiValidatedHostObservationBatch,
    ) -> Result<UiHostObservationBatchAdmissionReceipt, UiHostObservationAdmissionStop> {
        self.admit_host_with_requirement(batch, true)
    }

    fn admit_host_with_requirement(
        &mut self,
        batch: crate::facade::observation_report::UiValidatedHostObservationBatch,
        all_families_required: bool,
    ) -> Result<UiHostObservationBatchAdmissionReceipt, UiHostObservationAdmissionStop> {
        let families = batch_families(&batch);
        let unavailable = families
            .iter()
            .filter_map(|family| unavailable(*family))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        if all_families_required && !unavailable.is_empty() {
            self.poison();
            return Err(UiHostObservationAdmissionStop::RequiredFamilyUnavailable(
                unavailable,
            ));
        }
        let observations = seal_supported_observations(
            Arc::new(batch),
            &families,
            self.session,
            self.source_basis,
        );
        let admitted = if observations.is_empty() {
            Box::new([])
        } else {
            self.admit_batch(observations)
                .map_err(UiHostObservationAdmissionStop::Observation)?
        };
        Ok(UiHostObservationBatchAdmissionReceipt {
            admitted,
            unavailable,
        })
    }
}

fn batch_families(
    batch: &crate::facade::observation_report::UiValidatedHostObservationBatch,
) -> BTreeSet<MechanicalFamily> {
    let mut families = batch
        .reports()
        .iter()
        .map(|report| report.report().family())
        .collect::<BTreeSet<_>>();
    match batch.disposition() {
        crate::facade::observation_report::UiHostObservationBatchDisposition::Complete => {}
        crate::facade::observation_report::UiHostObservationBatchDisposition::Coalesced {
            family,
            ..
        }
        | crate::facade::observation_report::UiHostObservationBatchDisposition::Overflow {
            family,
            ..
        } => {
            families.insert(family);
        }
    }
    families
}

fn seal_supported_observations(
    batch: Arc<crate::facade::observation_report::UiValidatedHostObservationBatch>,
    families: &BTreeSet<MechanicalFamily>,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    source_basis: u64,
) -> Vec<UiAdmittedObservation> {
    let supported = families
        .iter()
        .filter_map(|family| map_supported(*family).map(|mapped| (*family, mapped)))
        .collect::<Vec<_>>();
    let shared_bytes =
        std::mem::size_of_val(batch.as_ref()).saturating_add(batch.canonical_core().byte_count());
    supported
        .into_iter()
        .enumerate()
        .map(|(ordinal, (mechanical, semantic))| {
            let report_index = batch
                .reports()
                .iter()
                .rposition(|report| report.report().family() == mechanical);
            let owner_order = report_index.map_or_else(
                || batch.canonical_core().sequences().last().value(),
                |index| batch.reports()[index].report().sequence().value(),
            );
            let retained_bytes = std::mem::size_of::<UiHostObservation>()
                .saturating_add(if ordinal == 0 { shared_bytes } else { 0 });
            let progress = match semantic {
                UiObservationFamily::HostViewport => UiObservationProgress::host_viewport(
                    batch.canonical_core().host_session(),
                    owner_order,
                ),
                UiObservationFamily::HostDeviceScale => UiObservationProgress::host_device_scale(
                    batch.canonical_core().host_session(),
                    owner_order,
                ),
                _ => unreachable!("host admission maps only host-owned semantic families"),
            };
            UiAdmittedObservation::seal(UiAdmittedObservationSeal {
                family: semantic,
                owner_order,
                retained_bytes,
                session,
                source_basis,
                progress: Some(progress),
                payload: UiAdmittedObservationPayload::Host(UiHostObservation {
                    batch: Arc::clone(&batch),
                    family: mechanical,
                    report_index,
                }),
            })
        })
        .collect()
}

const fn map_supported(family: MechanicalFamily) -> Option<UiObservationFamily> {
    match family {
        MechanicalFamily::Viewport => Some(UiObservationFamily::HostViewport),
        MechanicalFamily::DeviceScale => Some(UiObservationFamily::HostDeviceScale),
        MechanicalFamily::PointerMotion => None,
        MechanicalFamily::PointerButton
        | MechanicalFamily::Keyboard
        | MechanicalFamily::WindowFocus
        | MechanicalFamily::ScrollDelta
        | MechanicalFamily::Clock
        | MechanicalFamily::Tick
        | MechanicalFamily::TextComposition
        | MechanicalFamily::ImeComposition => None,
    }
}

const fn unavailable(family: MechanicalFamily) -> Option<UiHostObservationUnavailable> {
    let successor = match family {
        MechanicalFamily::Viewport | MechanicalFamily::DeviceScale => return None,
        MechanicalFamily::PointerMotion => UiHostObservationSuccessorOwner::PointerPresence,
        MechanicalFamily::PointerButton
        | MechanicalFamily::Keyboard
        | MechanicalFamily::TextComposition
        | MechanicalFamily::ImeComposition => UiHostObservationSuccessorOwner::Intent,
        MechanicalFamily::WindowFocus => UiHostObservationSuccessorOwner::Focus,
        MechanicalFamily::ScrollDelta => UiHostObservationSuccessorOwner::Scroll,
        MechanicalFamily::Clock | MechanicalFamily::Tick => {
            UiHostObservationSuccessorOwner::PresentationSampling
        }
    };
    Some(UiHostObservationUnavailable { family, successor })
}

impl UiHostObservationUnavailable {
    pub const fn family(self) -> MechanicalFamily {
        self.family
    }

    pub const fn successor(self) -> UiHostObservationSuccessorOwner {
        self.successor
    }
}

impl UiHostObservationBatchAdmissionReceipt {
    pub fn admitted(&self) -> &[UiObservationAdmissionReceipt] {
        &self.admitted
    }

    pub fn unavailable(&self) -> &[UiHostObservationUnavailable] {
        &self.unavailable
    }
}

impl UiHostObservation {
    pub const fn family(&self) -> MechanicalFamily {
        self.family
    }

    pub fn report(
        &self,
    ) -> Option<&crate::facade::observation_report::UiValidatedHostObservationReport> {
        self.report_index.map(|index| &self.batch.reports()[index])
    }

    pub fn frame_relation(
        &self,
    ) -> crate::facade::observation_report::UiHostObservationFrameRelation {
        self.batch.frame_relation()
    }

    pub fn batch_disposition(
        &self,
    ) -> crate::facade::observation_report::UiHostObservationBatchDisposition {
        self.batch.disposition()
    }
}
