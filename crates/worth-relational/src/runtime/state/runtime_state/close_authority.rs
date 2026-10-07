use super::{
    RelationalRuntimeAdmissionHoldDenial, RelationalRuntimeOwnerBinding,
    RelationalRuntimePublicationBinding,
};

/// The right, and the obligation, to finish one Relational runtime owner.
///
/// Only the handle the constructing owner holds carries this authority, so
/// closing a runtime is an owner lifecycle event rather than a consequence of
/// whichever reference happened to be released last. A service that is
/// mid-operation holds the state alive, but it holds no close authority and
/// therefore can never run the close on its own thread while its own admission
/// is still outstanding.
#[derive(Debug)]
pub(in crate::runtime) struct RelationalRuntimeCloseAuthority {
    runtime_instance_id: u64,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication: RelationalRuntimePublicationBinding,
}

impl RelationalRuntimeCloseAuthority {
    pub(super) fn new(
        runtime_instance_id: u64,
        lifecycle: RelationalRuntimeOwnerBinding,
        publication: RelationalRuntimePublicationBinding,
    ) -> Self {
        Self {
            runtime_instance_id,
            lifecycle,
            publication,
        }
    }
}

impl RelationalRuntimeCloseAuthority {
    /// Close the owner on the owner's own thread, before the owner releases
    /// anything the close depends on.
    ///
    /// Admission stops first, so no further operation can start. The calling
    /// thread then blocks until every already-admitted operation has returned;
    /// those operations complete normally and produce their real results.
    /// Remaining publication settlement is resolved with typed owner-loss
    /// accounting, and this runtime's query scratch hints are purged.
    ///
    /// The one caller is [`RelationalRuntime`]'s own `Drop`, which the language
    /// runs before any of that handle's fields, so the state, the configuration
    /// and the admissions being waited on are all still alive here. Nothing
    /// about the order those fields are declared in matters.
    ///
    /// An owner must not be closed from inside one of its own admitted
    /// operations. That is unreachable through the public surface: a service
    /// receives an admitted handle, which carries no close authority, and the
    /// owner handle is never reachable from a service.
    ///
    /// [`RelationalRuntime`]: super::RelationalRuntime
    pub(super) fn close(&self) {
        self.lifecycle.close();
        self.publication.close();
        crate::indexes::purge_index_query_scratch_hints(self.runtime_instance_id);
    }

    pub(super) fn try_hold_admission(
        &self,
    ) -> Result<RelationalRuntimeOwnerBinding, RelationalRuntimeAdmissionHoldDenial> {
        self.lifecycle.try_hold_admission()?;
        Ok(self.lifecycle.clone())
    }

    pub(super) fn seal_held(&self) -> RelationalRuntimeSeal {
        self.lifecycle.seal_held();
        RelationalRuntimeSeal {
            runtime_instance_id: self.runtime_instance_id,
        }
    }

    /// Resolve remaining publication settlement with typed owner-loss
    /// accounting, as at close, and spend this authority doing it.
    ///
    /// Taking the authority by value is what makes the resolution happen once:
    /// a sealed owner no longer holds anything that could resolve it again. The
    /// owner already records the seal when this runs, so a panic here unwinds
    /// through a sealed owner, whose drop closes nothing.
    pub(super) fn resolve_sealed_publication(self) {
        self.publication.close();
    }
}

/// A runtime owner sealed in place: admission is stopped for good with nothing
/// in flight, and the close authority is spent.
#[derive(Debug)]
pub(in crate::runtime) struct RelationalRuntimeSeal {
    runtime_instance_id: u64,
}

impl RelationalRuntimeSeal {
    /// Finish a sealed owner on drop. Admission and publication were closed by
    /// the seal, so neither is closed again; only this runtime's query scratch
    /// hints, which reads after the seal may still warm, are purged.
    pub(super) fn finish(&self) {
        crate::indexes::purge_index_query_scratch_hints(self.runtime_instance_id);
    }
}
