//! Sealed placement carried through the actual Host/Store opening recipe.
use std::sync::{Arc, Mutex, Weak};
use worth_store::physical_runtime::{FilesystemMediaAdmission, PhysicalRecoveryProcessYieldpoint};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHomeOpeningStage {
    DescriptorIncarnation,
    NativeReadmission,
    WorldReconciliation,
    QueryReadmission,
    Custody,
    ServiceHandoff,
}
impl ApplicationHomeOpeningStage {
    pub const ALL: [Self; 6] = [
        Self::DescriptorIncarnation,
        Self::NativeReadmission,
        Self::WorldReconciliation,
        Self::QueryReadmission,
        Self::Custody,
        Self::ServiceHandoff,
    ];
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHomeOpeningEdge {
    Before,
    After,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHomeOpeningArrival {
    pub stage: ApplicationHomeOpeningStage,
    pub edge: ApplicationHomeOpeningEdge,
}

/// An inert observation can be defaulted; an armed observer is minted only by
/// the Store certification constructor below. It returns no proof or authority.
#[derive(Clone, Default)]
pub struct ApplicationHomeOpeningObservation {
    observer: Option<Arc<dyn Fn(ApplicationHomeOpeningArrival) + Send + Sync>>,
    physical: Option<Arc<Mutex<Weak<worth_store::physical_runtime::ServingPhysicalRuntime>>>>,
}
impl ApplicationHomeOpeningObservation {
    pub(crate) fn bind_physical(
        &self,
        serving: &Arc<worth_store::physical_runtime::ServingPhysicalRuntime>,
    ) {
        if let Some(slot) = &self.physical {
            *slot.lock().unwrap_or_else(|e| e.into_inner()) = Arc::downgrade(serving);
        }
    }
    pub fn arrive_for_test(
        &self,
        stage: ApplicationHomeOpeningStage,
        edge: ApplicationHomeOpeningEdge,
    ) {
        if let Some(observer) = &self.observer {
            observer(ApplicationHomeOpeningArrival { stage, edge });
        }
    }
}

/// No public ordinary constructor, clone, Default, or deserializer.
pub struct ApplicationHomeOpeningControlForTest {
    observation: ApplicationHomeOpeningObservation,
    media: Option<FilesystemMediaAdmission>,
    recovery: Option<PhysicalRecoveryProcessYieldpoint>,
}
impl ApplicationHomeOpeningControlForTest {
    #[cfg(feature = "certification-test-authority")]
    pub fn new_for_test(
        observer: impl Fn(ApplicationHomeOpeningArrival) + Send + Sync + 'static,
    ) -> Self {
        Self {
            observation: ApplicationHomeOpeningObservation {
                observer: Some(Arc::new(observer)),
                physical: Some(Arc::new(Mutex::new(Weak::new()))),
            },
            media: None,
            recovery: None,
        }
    }
    #[cfg(feature = "certification-test-authority")]
    pub fn with_media_for_test(mut self, media: FilesystemMediaAdmission) -> Self {
        self.media = Some(media);
        self
    }
    #[cfg(feature = "certification-test-authority")]
    pub fn with_recovery_yieldpoint_for_test(
        mut self,
        recovery: PhysicalRecoveryProcessYieldpoint,
    ) -> Self {
        self.recovery = Some(recovery);
        self
    }
    #[cfg(feature = "certification-test-authority")]
    pub fn physical_control_for_test(&self) -> ApplicationHomePhysicalControlForTest {
        ApplicationHomePhysicalControlForTest {
            _private: (),
            serving: self
                .observation
                .physical
                .as_ref()
                .expect("certification constructor owns its scheduler slot")
                .clone(),
        }
    }
    pub fn observation(&self) -> ApplicationHomeOpeningObservation {
        self.observation.clone()
    }
}
impl crate::ApplicationHomeRequest {
    /// Transfers only placement into this same production request. The owned
    /// control is impossible to construct in an ordinary Store build.
    pub fn with_opening_control_for_test(
        mut self,
        control: ApplicationHomeOpeningControlForTest,
    ) -> Self {
        self.opening_observation = control.observation;
        #[cfg(feature = "certification-test-authority")]
        {
            let mut opening = crate::open::physical_admission::PhysicalOpening::production();
            if let Some(media) = control.media {
                opening = opening.with_media(media);
            }
            if let Some(recovery) = control.recovery {
                opening = opening.with_recovery_yieldpoint(recovery);
            }
            self.controls.opening = Some(opening);
        }
        #[cfg(not(feature = "certification-test-authority"))]
        let _ = (control.media, control.recovery);
        self
    }
}

/// Scheduling only, weakly attached to this exact actual Store receiver.
pub struct ApplicationHomePhysicalControlForTest {
    _private: (),
    #[cfg(feature = "certification-test-authority")]
    serving: Arc<Mutex<Weak<worth_store::physical_runtime::ServingPhysicalRuntime>>>,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ApplicationHomePhysicalControlDenial {
    Unavailable,
    Poisoned,
}
impl ApplicationHomePhysicalControlForTest {
    #[cfg(feature = "certification-test-authority")]
    pub fn pause_mutation_for_test(
        &self,
        seam: worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint,
    ) -> Result<
        worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate,
        ApplicationHomePhysicalControlDenial,
    > {
        let serving = self
            .serving
            .lock()
            .map_err(|_| ApplicationHomePhysicalControlDenial::Poisoned)?
            .upgrade()
            .ok_or(ApplicationHomePhysicalControlDenial::Unavailable)?;
        Ok(serving.certification_pause_physical_mutation_at(seam))
    }
}
