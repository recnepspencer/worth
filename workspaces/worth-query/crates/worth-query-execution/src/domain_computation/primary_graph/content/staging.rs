//! One concrete field-bound Native staging handoff for all application formats.
use super::super::WorthQueryValueDecodeDenial as Stop;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_query_declaration::facade::application_schema::ApplicationContentBindingRecipe;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, content::RelationalContentAttachments,
};
pub(in crate::domain_computation::primary_graph) fn stage(
    basis: &AdmittedRelationalBranchBasis,
    execution: &ExecutionRequest<'_, '_>,
    current: &RelationalContentAttachments,
    format: &ApplicationContentBindingRecipe,
    input: &dyn std::any::Any,
) -> Result<(RelationalContentAttachments, ContentRefId), Stop> {
    let frame = basis
        .encode_content(
            execution,
            CanonicalDigestId::new(format.declaration().identity),
            input,
        )
        .map_err(Stop::Content)?;
    require_contract(execution, &frame, format).map_err(Stop::Content)?;
    let mut staged = current.clone();
    staged.attach(execution, &frame).map_err(Stop::Content)?;
    Ok((staged, frame.identity()))
}

pub(in crate::domain_computation::primary_graph) fn require_contract(
    execution: &ExecutionRequest<'_, '_>,
    frame: &worth_relational::facade::content::RelationalContentFrame,
    recipe: &ApplicationContentBindingRecipe,
) -> Result<(), worth_relational::facade::content::RelationalContentDenial> {
    let expected = recipe.declaration();
    let maximum = usize::try_from(expected.maximum_frame_bytes).map_err(|_| {
        worth_relational::facade::content::RelationalContentDenial::ArithmeticOverflow
    })?;
    frame.require_contract(execution, &expected.name, expected.version, maximum)
}

use super::ContentWriteDenial;
use crate::domain_computation::primary_graph::value_decode::{
    ValueDecodeSession, ValueDecodeState,
};
use std::sync::Arc;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationValueBinding as Installed,
    WorthQueryInstalledApplicationValueBindingCatalog as Catalog,
};

pub(in crate::domain_computation::primary_graph) type AcquireBinding =
    fn(&ValueDecodeSession<'_>, &str, &str, &str) -> Result<Arc<Installed>, Stop>;

pub(in crate::domain_computation::primary_graph) struct ContentStaging<'borrow, 'run, 'host> {
    pub basis: &'borrow AdmittedRelationalBranchBasis,
    pub execution: &'borrow ExecutionRequest<'run, 'host>,
    pub current: &'borrow RelationalContentAttachments,
    pub bindings: &'borrow Catalog,
    pub denial: &'borrow mut Option<Stop>,
}
impl ContentStaging<'_, '_, '_> {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        self,
        locus: (&str, &str, &str),
        input: &dyn std::any::Any,
        acquire: AcquireBinding,
    ) -> Result<(RelationalContentAttachments, ContentRefId), ContentWriteDenial> {
        if let Some(cause) = *self.denial {
            return Err(ContentWriteDenial::Admission(cause));
        }
        let state = ValueDecodeState::admitted(self.bindings.clone());
        state
            .run(self.execution, |session| {
                acquire(session, locus.0, locus.1, locus.2)
            })
            .and_then(|result| result)
            .and_then(|binding| {
                let format = binding
                    .content_format()
                    .ok_or(Stop::BindingContractMismatch)?;
                stage(self.basis, self.execution, self.current, format, input)
            })
            .map_err(|cause| {
                *self.denial = Some(cause);
                match cause {
                    Stop::Content(cause) => ContentWriteDenial::Content(cause),
                    cause => ContentWriteDenial::Admission(cause),
                }
            })
    }
}
