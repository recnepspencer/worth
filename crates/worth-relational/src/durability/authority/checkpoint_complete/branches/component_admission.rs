//! Exact checkpoint components admitted against the folded original branch stream.
use super::*;
impl BranchImages<'_> {
    pub(in crate::durability::authority::checkpoint_complete) fn is_current_component(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
        roots: &FoldedRoots,
    ) -> bool {
        self.current
            .get(&descriptor.branch_id())
            .is_some_and(|axes| {
                !axes.removed
                    && axes.truth == descriptor.truth_version()
                    && axes.generation == descriptor.reference().generation()
                    && posture(axes.lifecycle)
                        == match descriptor.posture() {
                            RelationalBranchBasisPosture::Live => 0,
                            RelationalBranchBasisPosture::Archived => 1,
                            RelationalBranchBasisPosture::Deleting => 2,
                        }
                    && axes.matches_target(descriptor.reference().target(), roots)
            })
    }

    pub(in crate::durability::authority::checkpoint_complete) fn require_component(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
        roots: &FoldedRoots,
    ) -> Result<Arc<RelationalBranchRootSchemaAuthority>, Denial> {
        let key = (
            descriptor.branch_id(),
            descriptor.reference().generation().get(),
            match descriptor.posture() {
                RelationalBranchBasisPosture::Live => 0,
                RelationalBranchBasisPosture::Archived => 1,
                RelationalBranchBasisPosture::Deleting => 2,
            },
        );
        let axes = self
            .observations
            .get(&key)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        if axes.truth != descriptor.truth_version()
            || !axes.matches_target(descriptor.reference().target(), roots)
            || axes.schema.authority_digest() != descriptor.schema_commitment()
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let visibility = axes
            .commit
            .and_then(|commit| roots.get(commit))
            .and_then(|root| root.root.axes())
            .map_or([0; 32], |axes| axes.visibility.digest());
        if visibility != descriptor.visibility_commitment() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(axes.schema.clone())
    }
}
