//! Durable current publication custody; historical payload selection stays in History.
mod certification;
mod observation;
pub use certification::CurrentOwnershipContactsForTest;
mod completion;
pub(crate) mod corruption;
mod evidence;
mod merge_custody;
mod metadata_guard_court;
mod read;
pub(crate) mod record;
pub(crate) mod selection_pairing;
pub(crate) use completion::complete;
pub(crate) use evidence::PublicationEvidence;
pub(crate) use read::{
    decode_payload, linked_selectors, read, read_candidate, read_selected, CurrentPublicationRoster,
};

pub(crate) fn select_current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    records: &[RecordRef],
) -> Result<std::sync::Arc<CurrentPublicationRoster>, TransactionCommitError> {
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        answer = Some(read(execution, basis, records, context).map(std::sync::Arc::new));
    });
    match (handoff, answer) {
        (Ok(_), Some(answer)) => answer,
        (Err(_), Some(Err(cause))) => Err(cause),
        (Err(cause), _) => Err(TransactionCommitError::execution(cause.into())),
        (Ok(_), None) => unreachable!("completed Native read selected its roster"),
    }
}

pub(crate) fn reserve_symbols(
    registry: &crate::schema::data::RelationalSchemaRegistry,
    symbols: &mut crate::runtime::PendingSymbolMeanings,
) -> Result<(), crate::authority::mutation::MutationApplyDenial> {
    if registry.current_publication_storage.is_some() {
        use crate::schema::current_publication_ownership::{
            CATALOG, EDGE, LINK, POSITION, PUBLICATION,
        };
        for text in [
            CATALOG,
            PUBLICATION,
            EDGE,
            POSITION,
            LINK,
            "publication",
            "payload",
        ] {
            symbols.reserve(text)?;
        }
    }
    Ok(())
}

use crate::{
    history::data::CommitId,
    publication::data::{AuthoredRetirementRefusal, PublicationError},
    transactions::data::{RecordRef, TransactionCommitError},
};

fn corrupt(publication: CommitId, record: RecordRef) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        AuthoredRetirementRefusal::CorruptCurrentOwnership { publication },
        record,
    ))
}

pub(crate) fn checkpoint(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), TransactionCommitError> {
    crate::execution::request_stage::checkpoint(context, work).map_err(request)
}

pub(crate) fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
