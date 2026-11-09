//! Read-only birth classification accepts hostile Body twins without sealing them.
use super::*;

pub(super) fn run(
    execution: &ExecutionRequest<'_, '_>,
    state: &dyn CurrentPartitionAccess,
    id: RelationId,
    selected: &CanonicalRequiredOutputOrigin,
    birth: &CanonicalCommitEnvelopeBody,
    write: &Arc<CanonicalCommitEnvelope>,
    owned: bool,
) -> Result<(), Stop> {
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    // The original rewritten frame is the only history source.
                    // Birth twins are semantic inputs to the actual owner validator;
                    // neither an Original seal nor liveness/custody is fabricated.
                    let mut lookup = |version, _context: &mut MapKernelContext<'_, '_>| {
                        Ok((version == write.commit.version_id).then(|| Arc::clone(write)))
                    };
                    let current = Some(super::super::super::CurrentPublicationFrame::Retiring(
                        birth,
                    ));
                    let mut history = OutputOriginHistory::default();
                    if owned {
                        if !ownership_at_creation(
                            state,
                            id,
                            selected,
                            current,
                            &mut lookup,
                            &mut history,
                            execution,
                            context,
                        )? {
                            return Err("claimed relation was not producer-owned at birth".into());
                        }
                        Ok(())
                    } else {
                        validate(
                            state,
                            id,
                            selected,
                            current,
                            &mut lookup,
                            &mut history,
                            execution,
                            context,
                        )
                    }
                },
            )
        })
        .map_err(crate::history::data::codec_retirement_closure::request)?
        .map_err(crate::history::data::codec_retirement_closure::request)?
        .0
}
