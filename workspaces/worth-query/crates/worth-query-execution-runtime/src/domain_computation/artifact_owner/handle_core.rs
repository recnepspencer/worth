use std::sync::Arc;

use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDisposition, WorthQueryDisposedArtifact,
    WorthQueryRuntimeArtifactOwner,
};

#[derive(Clone, Copy)]
pub(super) enum WorthQueryArtifactHandleGuard {
    Owner(u64),
    Lease(u64),
}

impl WorthQueryArtifactHandleGuard {
    pub(super) const fn generation(self) -> u64 {
        match self {
            Self::Owner(generation) | Self::Lease(generation) => generation,
        }
    }
}

pub(super) struct WorthQueryArtifactHandleCore {
    pub(super) owner: Arc<WorthQueryRuntimeArtifactOwner>,
    pub(super) handle_identity: String,
    pub(super) holder_stage: String,
    pub(super) disposition: WorthQueryArtifactDisposition,
    pub(super) guard: WorthQueryArtifactHandleGuard,
    pub(super) active: bool,
}

impl WorthQueryArtifactHandleCore {
    pub(super) fn new_owner(
        owner: Arc<WorthQueryRuntimeArtifactOwner>,
        holder_stage: String,
        disposition: WorthQueryArtifactDisposition,
    ) -> Self {
        Self::new(
            owner,
            holder_stage,
            disposition,
            WorthQueryArtifactHandleGuard::Owner(1),
        )
    }

    pub(super) fn new(
        owner: Arc<WorthQueryRuntimeArtifactOwner>,
        holder_stage: String,
        disposition: WorthQueryArtifactDisposition,
        guard: WorthQueryArtifactHandleGuard,
    ) -> Self {
        let handle_identity =
            crate::domain_computation::artifact_identity::hash_artifact_identity_parts(&[
                "worth_query_move_only_artifact_handle_v1".into(),
                format!("owner:{}", owner.binding().owner_identity),
                format!("stage:{holder_stage}"),
                format!("generation:{}", guard.generation()),
                format!("disposition:{}", disposition.canonical_name()),
            ]);
        Self {
            owner,
            handle_identity,
            holder_stage,
            disposition,
            guard,
            active: true,
        }
    }

    pub(super) fn owner_generation(&self) -> u64 {
        match self.guard {
            WorthQueryArtifactHandleGuard::Owner(generation) => generation,
            WorthQueryArtifactHandleGuard::Lease(_) => {
                unreachable!("move-only owner handle cannot carry a lease guard")
            }
        }
    }

    pub(super) fn dispose(
        &mut self,
        disposition: WorthQueryArtifactDisposition,
        require_no_lease: bool,
    ) -> Result<WorthQueryDisposedArtifact, WorthQueryArtifactDenial> {
        self.owner.validate_guard(self.guard)?;
        let provider_release = match self.guard {
            WorthQueryArtifactHandleGuard::Owner(generation) => {
                self.owner
                    .release_owner(generation, disposition, require_no_lease)?
            }
            WorthQueryArtifactHandleGuard::Lease(generation) => {
                self.owner.release_lease(generation, disposition)?
            }
        };
        self.active = false;
        Ok(WorthQueryDisposedArtifact::new(
            self.owner.binding().owner_identity.clone(),
            self.owner.binding().occurrence_identity.clone(),
            disposition,
            provider_release,
        ))
    }
}

impl Drop for WorthQueryArtifactHandleCore {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        match self.guard {
            WorthQueryArtifactHandleGuard::Owner(_) | WorthQueryArtifactHandleGuard::Lease(_) => {
                self.owner
                    .release_guard_on_drop(self.guard, WorthQueryArtifactDisposition::Released);
            }
        }
    }
}
