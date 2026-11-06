use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::ApplicationContentBinding;
use worth_relational::facade::content::RelationalContentFrame;

/// Completed field-bound admission retains actual Native byte custody.
///
/// A descriptive identity supplies neither the field admission nor byte custody:
/// ```compile_fail,E0308
/// use worth_query_execution::facade::primary_graph::AdmittedContent;
/// use worth_query_declaration::facade::application_schema::ApplicationContentBinding;
/// use worth_foundational::facade::ContentRefId;
/// fn cannot_forge<C:ApplicationContentBinding>(identity:ContentRefId)->AdmittedContent<C> {
///     identity
/// }
/// ```
#[derive(Clone, Debug)]
pub struct AdmittedContent<C: ApplicationContentBinding> {
    frame: RelationalContentFrame,
    _format: PhantomData<fn() -> C>,
}
impl<C: ApplicationContentBinding> AdmittedContent<C> {
    pub(in crate::domain_computation::primary_graph) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        frame: RelationalContentFrame,
        recipe: &worth_query_declaration::facade::application_schema::ApplicationContentBindingRecipe,
    ) -> Result<Self, worth_relational::facade::content::RelationalContentDenial> {
        super::staging::require_contract(execution, &frame, recipe)?;
        Ok(Self {
            frame,
            _format: PhantomData,
        })
    }
    pub fn identity(&self) -> worth_foundational::facade::ContentRefId {
        self.frame.identity()
    }
    pub fn payload(&self) -> &[u8] {
        self.frame.payload()
    }
    pub fn format(&self) -> &worth_foundational::facade::ContentFormat {
        self.frame.format()
    }
}

pub(in crate::domain_computation::primary_graph) fn read_selected(
    basis: Option<&worth_relational::facade::branch::AdmittedRelationalBranchBasis>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    session: &crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<'_>,
    identity: worth_foundational::facade::ContentRefId,
    recipe: &worth_query_declaration::facade::application_schema::ApplicationContentBindingRecipe,
) -> Result<RelationalContentFrame, super::ContentReadDenial> {
    use crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial as Stop;
    let Some(basis) = basis else {
        session.refuse_content(Stop::ContentBasisUnavailable);
        return Err(super::ContentReadDenial::UnavailableBasis);
    };
    basis
        .read_content(
            execution,
            identity,
            worth_foundational::facade::CanonicalDigestId::new(recipe.declaration().identity),
        )
        .map_err(|cause| {
            session.refuse_content(Stop::Content(cause));
            super::ContentReadDenial::Content(cause)
        })
}
