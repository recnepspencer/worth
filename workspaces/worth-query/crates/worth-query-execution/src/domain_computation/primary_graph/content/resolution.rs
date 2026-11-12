//! One admission path for field-bound immutable content on every reader.
use super::{AdmittedContent, ContentReadDenial as Denial};
use crate::domain_computation::primary_graph::{
    value_decode::ValueDecodeSession, WorthQueryValueDecodeDenial as Stop,
};
use worth_foundational::facade::ContentRefId;
use worth_query_declaration::facade::application_schema::ApplicationContentBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding;
use worth_relational::facade::content::RelationalContentFrame;

pub(in crate::domain_computation::primary_graph) fn typed<C: ApplicationContentBinding>(
    result: Result<Option<RelationalContentFrame>, Denial>,
) -> Result<Option<AdmittedContent<C>>, Denial> {
    result.map(|frame| frame.map(AdmittedContent::from_resolved_frame))
}

pub(in crate::domain_computation::primary_graph) fn resolve_frame(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    session: &ValueDecodeSession<'_>,
    binding: &WorthQueryInstalledApplicationValueBinding,
    identity: ContentRefId,
    acquire: &mut dyn FnMut() -> Result<RelationalContentFrame, Denial>,
) -> Result<RelationalContentFrame, Denial> {
    let result = (|| {
        let format = binding
            .content_format()
            .ok_or(Denial::BindingContractMismatch)?;
        let frame = acquire()?;
        if frame.identity() != identity {
            return Err(Denial::BindingContractMismatch);
        }
        super::require_contract(execution, &frame, format).map_err(Denial::Content)?;
        Ok(frame)
    })();
    if let Err(cause) = &result {
        session.refuse_content(match cause {
            Denial::Content(cause) => Stop::Content(*cause),
            Denial::UnavailableBasis => Stop::ContentBasisUnavailable,
            Denial::Admission(cause) => *cause,
            _ => Stop::BindingContractMismatch,
        });
    }
    result
}
