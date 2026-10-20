//! Business admission and typed emission for the actual archive request.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static DECISIONS: AtomicUsize = AtomicUsize::new(0);
static CANDIDATES: AtomicUsize = AtomicUsize::new(0);
pub(super) fn invocation_counts() -> [usize; 2] {
    [
        DECISIONS.load(Ordering::Relaxed),
        CANDIDATES.load(Ordering::Relaxed),
    ]
}
pub(super) struct ArchiveHandler;
impl OperationHandler<DocumentRetentionSchema, ArchiveBinding> for ArchiveHandler {
    fn decide(
        &self,
        input: &SetRetentionInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, ArchiveBinding>,
    ) -> HandlerResult<
        <ArchiveBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::Decision,
        SetRetentionDenial,
    > {
        DECISIONS.fetch_add(1, Ordering::Relaxed);
        let target = (|| {
            let document = reader
                .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())?;
            reader.mutation_target(&document)
        })();
        match target {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &SetRetentionInput,
        _: <ArchiveBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::Decision,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ArchiveBinding>,
    ) -> HandlerResult<ArchiveRequested, SetRetentionDenial> {
        CANDIDATES.fetch_add(1, Ordering::Relaxed);
        match writer.emit_external(ArchiveEffect::reference(), input.clone()) {
            Ok(()) => HandlerResult::Completed(ArchiveRequested {
                retention_days: input.retention_days,
            }),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
