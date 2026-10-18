//! Admit replacement only while the actual codec recovery destination is unbound.
use super::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn admit_codec_recovery_destination(&mut self) -> Result<(), &'static str> {
        let close = self
            .tenure
            .close_authority()
            .ok_or("codec recovery destination is not the runtime owner")?;
        // An exclusive borrow alone does not exclude independently held native
        // services. Check the real state allocation, including weak bindings.
        let state = std::sync::Arc::get_mut(&mut self.state)
            .ok_or("codec recovery destination state is already bound to a service")?;
        if !close
            .exclusively_holds_runtime_bindings(&state.owner_lifecycle, &state.publication_owner)
        {
            return Err(
                "codec recovery destination has retained native service or publication custody",
            );
        }
        // With no external state/lifecycle/publication bindings, no service can
        // begin work during restore. The exclusive owner borrow prevents new
        // bindings. Schema installation receipts retain none of those handles.
        if state.history.latest_commit_identity().is_some()
            || state.history.has_published_branch_basis()
        {
            return Err("codec recovery destination already contains published history");
        }
        if state.visibility.active_snapshot_count() != 0
            || state.history.has_detached_retention_obligations()
        {
            return Err("codec recovery destination retains native read or snapshot pins");
        }
        Ok(())
    }
}
