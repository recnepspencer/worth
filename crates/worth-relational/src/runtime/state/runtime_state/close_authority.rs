use super::{RelationalRuntimeOwnerBinding, RelationalRuntimePublicationBinding};

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
    #[cfg(test)]
    runtime_instance_id: u64,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication: RelationalRuntimePublicationBinding,
}

impl RelationalRuntimeCloseAuthority {
    pub(super) fn new(
        _runtime_instance_id: u64,
        lifecycle: RelationalRuntimeOwnerBinding,
        publication: RelationalRuntimePublicationBinding,
    ) -> Self {
        Self {
            #[cfg(test)]
            runtime_instance_id: _runtime_instance_id,
            lifecycle,
            publication,
        }
    }
}

impl RelationalRuntimeCloseAuthority {
    pub(super) fn exclusively_holds_runtime_bindings(
        &self,
        owner: &super::RelationalRuntimeOwner,
        publication: &super::RelationalRuntimePublicationOwner,
    ) -> bool {
        owner.held_only_by_owner_and_close(&self.lifecycle)
            && publication.held_only_by_owner_and_close(&self.publication)
    }

    /// Close the owner on the owner's own thread, before the owner releases
    /// anything the close depends on.
    ///
    /// Admission stops first, so no further operation can start. The calling
    /// thread then blocks until every already-admitted operation has returned;
    /// those operations complete normally and produce their real results.
    /// Remaining publication settlement is resolved with typed owner-loss
    /// accounting. Unit tests also purge this runtime's index-query scratch hints.
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
        #[cfg(test)]
        crate::indexes::purge_index_query_scratch_hints(self.runtime_instance_id);
    }
}
