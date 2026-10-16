//! Access-sealed cleanup of the actual admitted read interest.
use super::*;
impl<
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        D: WorthQueryApplicationOutputDemand<S>,
    > WorthQueryAdmittedProgramOutput<S, P, D>
{
    pub fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputReadCloseDenial> {
        self.admitted
            .close_read_request(phase, runtime.runtime(), branch)
    }
}
