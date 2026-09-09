use std::sync::Mutex;

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

    pub(crate) fn publish<R>(
        &self,
        publish: impl FnOnce() -> R,
    ) -> Result<R, WorthQueryProductActivationDenial> {
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
        let _publication = ProductDefinitionPublication { gate: self };
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

struct ProductDefinitionPublication<'a> {
    gate: &'a WorthQueryProductActivationGate,
}

impl Drop for ProductDefinitionPublication<'_> {
    fn drop(&mut self) {
        self.gate
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .publishing = false;
    }
}
