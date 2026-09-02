use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiMountedInstanceIdentity};

pub(crate) const UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerPresencePresentationTrigger {
    presentation: UiHostObservationPresentationBasis,
    changed_instances: Box<[UiMountedInstanceIdentity]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPointerPresencePresentationTriggerDenial {
    EmptyChangedNeighborhood,
    ChangedNeighborhoodCapacityExceeded,
}

impl UiPointerPresencePresentationTrigger {
    pub(crate) fn new(
        presentation: UiHostObservationPresentationBasis,
        changed_instances: &[UiMountedInstanceIdentity],
    ) -> Result<Self, UiPointerPresencePresentationTriggerDenial> {
        let mut changed_instances = changed_instances.to_vec();
        changed_instances.sort_unstable();
        changed_instances.dedup();
        if changed_instances.is_empty() {
            return Err(UiPointerPresencePresentationTriggerDenial::EmptyChangedNeighborhood);
        }
        if changed_instances.len() > UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY {
            return Err(
                UiPointerPresencePresentationTriggerDenial::ChangedNeighborhoodCapacityExceeded,
            );
        }
        Ok(Self {
            presentation,
            changed_instances: changed_instances.into_boxed_slice(),
        })
    }

    pub(crate) const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(crate) fn changed_instances(&self) -> &[UiMountedInstanceIdentity] {
        &self.changed_instances
    }
}
