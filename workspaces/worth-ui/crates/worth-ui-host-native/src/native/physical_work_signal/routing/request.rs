use worth_signal::facade::ResourceRequestHandle;
use worth_ui_host_contract::UiGlyphRasterTransactionPending;

use super::super::declarations::PHYSICAL_SIGNAL_ROUTE_CAPACITY;
use super::super::identity::UiNativePhysicalSignalRuntimeIdentity;
use super::super::identity::{
    UiNativePhysicalAtlasRequestIdentity, UiNativePhysicalAtlasUploadIdentity,
    UiNativePhysicalPresentationIdentity,
};
use super::{
    UiNativePhysicalSignalExternalBasis, UiNativePhysicalSignalExternalObservation,
    UiNativePhysicalSignalExternalStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativePhysicalSignalWork {
    AtlasPlanning(UiNativePhysicalAtlasRequestIdentity),
    AtlasUpload(UiNativePhysicalAtlasUploadIdentity),
    Presentation(UiNativePhysicalPresentationIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalSignalRequest {
    work: UiNativePhysicalSignalWork,
    handle: ResourceRequestHandle,
    owner_handle: ResourceRequestHandle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalSignalRequestToken {
    runtime: UiNativePhysicalSignalRuntimeIdentity,
    work: UiNativePhysicalSignalWork,
    handle: ResourceRequestHandle,
}

impl UiNativePhysicalSignalRequestToken {
    pub(crate) const fn work(self) -> UiNativePhysicalSignalWork {
        self.work
    }

    pub(crate) const fn handle(self) -> ResourceRequestHandle {
        self.handle
    }

    pub(crate) const fn observe(
        self,
        status: UiNativePhysicalSignalExternalStatus,
    ) -> UiNativePhysicalSignalExternalObservation {
        self.external_basis().observe(status)
    }

    pub(crate) const fn external_basis(self) -> UiNativePhysicalSignalExternalBasis {
        UiNativePhysicalSignalExternalBasis::new(self.runtime, self.work, self.handle)
    }
}

impl UiNativePhysicalSignalWork {
    pub(in crate::native::physical_work_signal) const fn slot_lineage(
        self,
    ) -> super::super::identity::UiNativePhysicalSignalSlotLineage {
        self.request_identity().presentation_basis().slot_lineage()
    }

    pub(in crate::native::physical_work_signal) const fn sequence(self) -> u64 {
        match self {
            Self::AtlasPlanning(identity) => identity.sequence(),
            Self::AtlasUpload(identity) => identity.request().sequence(),
            Self::Presentation(identity) => identity.sequence(),
        }
    }

    pub(in crate::native::physical_work_signal) const fn request_identity(
        self,
    ) -> super::super::identity::UiNativePhysicalRequestIdentity {
        match self {
            Self::AtlasPlanning(identity) => {
                super::super::identity::UiNativePhysicalRequestIdentity::new(
                    identity.sequence(),
                    identity.presentation_basis(),
                )
            }
            Self::AtlasUpload(identity) => {
                super::super::identity::UiNativePhysicalRequestIdentity::new(
                    identity.request().sequence(),
                    identity.request().presentation_basis(),
                )
            }
            Self::Presentation(identity) => identity.request_identity(),
        }
    }
}

pub(crate) struct UiNativePhysicalSignalRoute {
    requests: Vec<UiNativePhysicalSignalRequest>,
}

impl UiNativePhysicalSignalRoute {
    pub(crate) fn new() -> Self {
        Self {
            requests: Vec::with_capacity(PHYSICAL_SIGNAL_ROUTE_CAPACITY),
        }
    }

    pub(crate) fn record(
        &mut self,
        work: UiNativePhysicalSignalWork,
        handle: ResourceRequestHandle,
    ) -> Result<(), UiNativePhysicalSignalRegistryError> {
        if self.requests.len() >= PHYSICAL_SIGNAL_ROUTE_CAPACITY {
            return Err(UiNativePhysicalSignalRegistryError::Full);
        }
        if self
            .requests
            .iter()
            .any(|request| request.work == work || request.handle == handle)
        {
            return Err(UiNativePhysicalSignalRegistryError::AlreadyRegistered);
        }
        self.requests.push(UiNativePhysicalSignalRequest {
            work,
            handle,
            owner_handle: handle,
        });
        Ok(())
    }

    pub(crate) fn owner_handle(
        &self,
        work: UiNativePhysicalSignalWork,
    ) -> Option<ResourceRequestHandle> {
        self.requests
            .iter()
            .find(|request| request.work == work)
            .map(|request| request.owner_handle)
    }

    pub(crate) fn adopt_owner_handle(
        &mut self,
        work: UiNativePhysicalSignalWork,
        owner_handle: ResourceRequestHandle,
    ) -> bool {
        let Some(request) = self
            .requests
            .iter_mut()
            .find(|request| request.work == work)
        else {
            return false;
        };
        request.owner_handle = owner_handle;
        true
    }

    pub(crate) fn acknowledge_owner_handle(
        &mut self,
        work: UiNativePhysicalSignalWork,
        retained: ResourceRequestHandle,
        current: ResourceRequestHandle,
    ) -> bool {
        let Some(request) = self
            .requests
            .iter_mut()
            .find(|request| request.work == work)
        else {
            return false;
        };
        if request.owner_handle != retained || request.handle != current {
            return false;
        }
        request.owner_handle = current;
        true
    }

    pub(crate) fn token_for(
        &self,
        runtime: UiNativePhysicalSignalRuntimeIdentity,
        work: UiNativePhysicalSignalWork,
    ) -> Result<UiNativePhysicalSignalRequestToken, UiNativePhysicalSignalRegistryError> {
        self.requests
            .iter()
            .find(|request| request.work == work)
            .map(|request| UiNativePhysicalSignalRequestToken {
                runtime,
                work: request.work,
                handle: request.handle,
            })
            .ok_or(UiNativePhysicalSignalRegistryError::Unknown)
    }

    pub(crate) fn remove(&mut self, token: UiNativePhysicalSignalRequestToken) -> bool {
        let Some(index) = self
            .requests
            .iter()
            .position(|request| request.work == token.work && request.handle == token.handle)
        else {
            return false;
        };
        self.requests.remove(index);
        true
    }

    pub(crate) fn replace_handle(
        &mut self,
        token: UiNativePhysicalSignalRequestToken,
        handle: ResourceRequestHandle,
    ) -> bool {
        let Some(request) = self
            .requests
            .iter_mut()
            .find(|request| request.work == token.work && request.handle == token.handle)
        else {
            return false;
        };
        request.handle = handle;
        true
    }

    pub(crate) fn replace_work(
        &mut self,
        token: UiNativePhysicalSignalRequestToken,
        work: UiNativePhysicalSignalWork,
    ) -> bool {
        if self.requests.iter().any(|request| request.work == work) {
            return false;
        }
        let Some(request) = self
            .requests
            .iter_mut()
            .find(|request| request.work == token.work && request.handle == token.handle)
        else {
            return false;
        };
        request.work = work;
        true
    }

    pub(crate) fn len(&self) -> usize {
        self.requests.len()
    }

    pub(crate) fn contains_presentation_basis(
        &self,
        basis: super::super::identity::UiNativePhysicalPresentationBasis,
    ) -> bool {
        self.requests.iter().any(|request| {
            matches!(
                request.work,
                UiNativePhysicalSignalWork::Presentation(identity) if identity.basis() == basis
            )
        })
    }

    pub(crate) fn atlas_upload(
        &self,
        pending: UiGlyphRasterTransactionPending,
    ) -> Option<UiNativePhysicalAtlasUploadIdentity> {
        self.requests.iter().find_map(|request| match request.work {
            UiNativePhysicalSignalWork::AtlasUpload(identity) if identity.pending() == pending => {
                Some(identity)
            }
            _ => None,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.requests.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativePhysicalSignalRegistryError {
    Full,
    AlreadyRegistered,
    Unknown,
}
