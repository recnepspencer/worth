//! Retire the original sealed World after Query, before Native admission.
use super::*;
use worth_runtime_world::facade::{
    RetiredRuntimeWorldHome, RuntimeWorldHomeRetirement,
    RuntimeWorldHomeRetirementProgress as Progress,
};
pub(super) enum WorldOwnerCloseCustody {
    Absent,
    Retiring(Option<RuntimeWorldHomeRetirement>),
    Retired(RetiredRuntimeWorldHome),
}
impl WorldOwnerCloseCustody {
    pub(super) fn from_original(original: Option<RuntimeWorldHomeRetirement>) -> Self {
        match original {
            Some(original) => Self::Retiring(Some(original)),
            None => Self::Absent,
        }
    }
    pub(super) fn into_retired(self) -> Option<RetiredRuntimeWorldHome> {
        match self {
            Self::Absent => None,
            Self::Retired(original) => Some(original),
            Self::Retiring(_) => {
                unreachable!("physical Closed follows complete original World retirement")
            }
        }
    }
}
impl ApplicationHomeRecoveryCustody {
    pub(super) fn advance_world_owner_close(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeCloseDenial> {
        let WorldOwnerCloseCustody::Retiring(slot) = &mut self.world_owner else {
            return Ok(());
        };
        let NativeCloseCustody::Serving(native) = &self.native else {
            return Err(ApplicationHomeCloseDenial::WorldCompositionRetained);
        };
        loop {
            match slot
                .as_mut()
                .expect("same original World owner")
                .advance(request, native.runtime())
                .map_err(ApplicationHomeCloseDenial::WorldOwnerRetirement)?
            {
                Progress::Advanced => {}
                Progress::Complete => {
                    let original = slot
                        .take()
                        .expect("same completed World owner")
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("owner reported Complete"));
                    self.world_owner = WorldOwnerCloseCustody::Retired(original);
                    return Ok(());
                }
            }
        }
    }
}
