//! One non-cloneable Signal slot is minted with the admitted Home's World slot.
use super::*;
use std::sync::Weak;
use worth_foundational::facade::{DurableSignalOwnerIdentity, SignalOwnerSlotIdentity};

pub(crate) struct UnclaimedSignalSlot {
    serving: Weak<ServingPhysicalRuntime>,
    binding: SignalDurableHomeBinding,
    medium: Mutex<Option<Arc<StoreSignalMedium>>>,
    initial: Mutex<Option<Arc<dyn super::initial_source::RetainedInitialSignalSource>>>,
}
impl UnclaimedSignalSlot {
    pub(crate) fn from_admitted_home(
        serving: &Arc<ServingPhysicalRuntime>,
        descriptor: &crate::descriptor::HomeDescriptor,
    ) -> Self {
        Self {
            serving: Arc::downgrade(serving),
            binding: SignalDurableHomeBinding::from_admitted_adapter(
                DurableSignalOwnerIdentity::from_recorded(
                    descriptor.identity,
                    SignalOwnerSlotIdentity::from_recorded(0),
                ),
                descriptor.incarnation,
                *descriptor.binding.selected().revision().bytes(),
                descriptor.build,
            ),
            medium: Mutex::new(None),
            initial: Mutex::new(None),
        }
    }
    pub(crate) fn retain_initial<T: super::OriginalSignalInitialSource + 'static>(
        &self,
        original: T,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<super::HomeInitialSignalSource<T>>, D> {
        let mut slot = self.initial.try_lock().map_err(|_| D::PublicationBusy)?;
        if slot.is_some() {
            return Err(D::PublicationBusy);
        }
        let original = super::HomeInitialSignalSource::retain(original, request)?;
        *slot = Some(original.clone());
        Ok(original)
    }
    pub(crate) fn settle_initial(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), super::HomeSignalInitialSourceDenial> {
        let retained = self
            .initial
            .try_lock()
            .map_err(|_| super::HomeSignalInitialSourceDenial::Medium(D::PublicationBusy))?
            .clone();
        if let Some(original) = retained {
            original.settle_for_release(request)?;
            self.initial
                .try_lock()
                .map_err(|_| super::HomeSignalInitialSourceDenial::Medium(D::PublicationBusy))?
                .take();
        }
        Ok(())
    }
    pub(crate) fn attach(
        &self,
        checkpoint: Option<worth_signal::facade::durability::SignalAuthoredHomeImage>,
        original_source: Option<crate::checkpoint::SelectedHomeSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<StoreSignalMedium>, D> {
        let mut retained = self.medium.try_lock().map_err(|_| D::PublicationBusy)?;
        if let Some(medium) = retained.as_ref() {
            return Ok(Arc::clone(medium));
        }
        let serving = self.serving.upgrade().ok_or(D::Unavailable)?;
        let (funding, admission) = StoreSignalMedium::prepare_attachment(
            &serving,
            &self.binding,
            checkpoint.as_ref(),
            request,
        )?;
        let medium = Arc::new(StoreSignalMedium {
            serving: Mutex::new(Some(serving)),
            binding: self.binding.clone(),
            home_source: Mutex::new(original_source),
            checkpoint,
            placement: crate::open::physical_admission::placement(),
            publications: SignalPublications::default(),
            admission,
            _funding: funding,
        });
        *retained = Some(Arc::clone(&medium));
        Ok(medium)
    }
}
impl StoreSignalMedium {
    /// Called only after the real Signal owner has completed source admission.
    /// Subsequent captures serve new publications; admission retries keep the original cut.
    pub(crate) fn complete_home_source_admission(&self) -> Result<(), D> {
        self.home_source
            .try_lock()
            .map_err(|_| D::PublicationBusy)?
            .take();
        Ok(())
    }
}
