//! The one original-affine read-origin cleanup path for every handoff.
use super::*;
impl<S, P, R> ReadCustody<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn close_native(
        &mut self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: WorthQueryProductBranch,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseDenial> {
        use WorthQueryProgramOutputCloseDenial as Denial;
        let result =
            if !std::ptr::eq(runtime.runtime(), request.application) || branch != request.branch {
                Err(Denial::FreshRequestMismatch)
            } else {
                request
                    .application
                    .with_application_advancement(request.scope, |phase| {
                        phase
                            .checked_execution_request_for(request.application.product_runtime())
                            .map_err(Denial::Execution)?;
                        if let Some(cleanup) = self.cleanup.as_mut() {
                            cleanup
                                .close_in_advancement(&phase, request.application)
                                .map_err(Denial::Native)?;
                        }
                        if let Some(tree) = self.tree.as_mut() {
                            tree.close_read_request(&phase, runtime, branch)
                                .map_err(Denial::Native)?;
                        }
                        Ok(())
                    })
                    .map_err(Denial::Execution)
                    .and_then(|result| result)
            };
        result?;
        Ok(WorthQueryClosedProgramOutputRequest::from_cleanup(
            self.cleanup.take(),
        ))
    }
}
