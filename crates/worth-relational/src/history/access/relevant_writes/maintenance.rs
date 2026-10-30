use super::RelationalAuthoredWriteDenial as Denial;
use crate::change_source::RelationalSelectedCommit;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Discard the optional acceleration for this exact owner-selected commit.
    /// Existing immutable readers keep their own original grants. Canonical
    /// history, semantic revisions and correction records are unchanged.
    pub fn discard_authored_write_generation(
        &self,
        execution: ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
    ) -> Result<(), Denial> {
        require_exact(self, &selected)?;
        super::run(execution, super::visit)?;
        self.history
            .discard_authored_write_generation(selected.commit_id())
    }

    /// Reconstruct the selected single-parent canonical ancestry explicitly.
    /// O(ancestry × log ancestry + authored writes × log distinct keys), paid
    /// from this original request. Ordinary correction reads never invoke it.
    /// The result counts actual canonical occurrences, independently of page visits.
    pub fn rebuild_authored_write_generation(
        &self,
        execution: ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
    ) -> Result<usize, Denial> {
        require_exact(self, &selected)?;
        self.history
            .rebuild_authored_write_generation(execution, selected.commit_id())
    }
}

fn require_exact(
    runtime: &RelationalRuntime,
    selected: &RelationalSelectedCommit,
) -> Result<(), Denial> {
    if selected.runtime_instance_id() != runtime.runtime_instance_id() {
        return Err(Denial::ForeignOwner);
    }
    if selected.observation().commit_id() != Some(selected.commit_id()) {
        return Err(Denial::NotExactSelection);
    }
    Ok(())
}
