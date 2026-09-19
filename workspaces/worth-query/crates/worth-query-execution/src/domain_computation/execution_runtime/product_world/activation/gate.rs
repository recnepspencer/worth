use std::sync::{Arc, Mutex};

use super::WorthQueryProductActivationDenial;

pub(crate) struct WorthQueryProductActivationGate {
    active: Mutex<ProductActivationState>,
    _capacity: super::capacity::ActivationCapacityReservation,
}

#[derive(Default)]
struct ProductActivationState {
    readers: usize,
    publishing: bool,
}

impl WorthQueryProductActivationGate {
    pub(super) fn new(capacity: super::capacity::ActivationCapacityReservation) -> Self {
        Self {
            active: Mutex::new(ProductActivationState::default()),
            _capacity: capacity,
        }
    }

    pub(crate) fn with_admission<R, E>(
        &self,
        inspect: impl FnOnce() -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<WorthQueryProductActivationDenial>,
    {
        {
            let mut active = self
                .active
                .lock()
                .map_err(|_| E::from(WorthQueryProductActivationDenial::GateUnavailable))?;
            if active.publishing {
                return Err(E::from(
                    WorthQueryProductActivationDenial::PublicationInProgress,
                ));
            }
            active.readers += 1;
        }
        let _admission = ProductReadAdmission { gate: self };
        inspect()
    }

    pub(crate) fn begin_publication(
        self: &Arc<Self>,
    ) -> Result<WorthQueryProductPublicationAdmission, WorthQueryProductActivationDenial> {
        {
            let mut active = self
                .active
                .lock()
                .map_err(|_| WorthQueryProductActivationDenial::GateUnavailable)?;
            if active.publishing || active.readers != 0 {
                return Err(WorthQueryProductActivationDenial::PublicationInProgress);
            }
            active.publishing = true;
        }
        Ok(WorthQueryProductPublicationAdmission {
            gate: Arc::clone(self),
        })
    }

    pub(crate) fn publish<R>(
        self: &Arc<Self>,
        publish: impl FnOnce() -> R,
    ) -> Result<R, WorthQueryProductActivationDenial> {
        let _publication = self.begin_publication()?;
        Ok(publish())
    }
}

struct ProductReadAdmission<'a> {
    gate: &'a WorthQueryProductActivationGate,
}

impl Drop for ProductReadAdmission<'_> {
    fn drop(&mut self) {
        self.gate
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .readers -= 1;
    }
}

pub(crate) struct WorthQueryProductPublicationAdmission {
    gate: Arc<WorthQueryProductActivationGate>,
}

impl Drop for WorthQueryProductPublicationAdmission {
    fn drop(&mut self) {
        self.gate
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .publishing = false;
    }
}
