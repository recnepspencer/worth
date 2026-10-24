//! Borrowed original fields qualified by the installed correction manifest.
use super::super::WorthQuerySelectedCorrectionSource;
use crate::domain_computation::primary_graph::application_attempt::OutputRoleUse;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputProjectionDenial;
use std::{cell::RefCell, marker::PhantomData};
use worth_execution::{ExecutionRequest, MapKernelStop};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionOutputField,
        ApplicationCorrectionSourceFieldDenial, ApplicationCorrectionSourceOutputs,
    },
    application_operation::{
        ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
    },
    application_schema::ApplicationSchema,
};
mod preflight;
pub(in crate::domain_computation::primary_graph) use preflight::validate_witness;

/// The original source field could not be lawfully projected.
#[derive(Debug)]
pub enum WorthQueryCorrectionOutputProjectionDenial {
    UndeclaredField,
    ForeignWitness,
    FieldMeaningMismatch,
    MaterialUnavailable,
    Output(WorthQueryApplicationOutputProjectionDenial),
    Stopped(MapKernelStop),
}

pub(super) struct SourceOutputs<'source, 'runtime, 'request, 'budget, S, C> {
    source: &'source WorthQuerySelectedCorrectionSource<'runtime, S>,
    request: ExecutionRequest<'request, 'budget>,
    failure: RefCell<Option<WorthQueryCorrectionOutputProjectionDenial>>,
    binding: PhantomData<fn() -> C>,
}

impl<'source, 'runtime, 'request, 'budget, S, C>
    SourceOutputs<'source, 'runtime, 'request, 'budget, S, C>
where
    S: ApplicationSchema,
    C: ApplicationCorrectionBinding<S>,
{
    pub(super) fn new(
        source: &'source WorthQuerySelectedCorrectionSource<'runtime, S>,
        request: ExecutionRequest<'request, 'budget>,
    ) -> Self {
        Self {
            source,
            request,
            failure: RefCell::new(None),
            binding: PhantomData,
        }
    }
    pub(super) fn take_failure(&self) -> Option<WorthQueryCorrectionOutputProjectionDenial> {
        self.failure.borrow_mut().take()
    }

    fn read(
        &self,
        field: &ApplicationCorrectionOutputField<S>,
    ) -> Result<&AspectValue, WorthQueryCorrectionOutputProjectionDenial> {
        use WorthQueryCorrectionOutputProjectionDenial as D;
        self.request.check_live().map_err(D::Stopped)?;
        let evidence = self.source.occurrence().commit_evidence();
        let original = evidence
            .original_authority()
            .retained_governed_input_carrier()
            .and_then(|input| input.correction())
            .ok_or(D::MaterialUnavailable)?;
        validate_witness(
            std::any::TypeId::of::<C::Source>(),
            original.association().declaration().source_output_fields(),
            self.source
                .application()
                .installed_schema()
                .value_bindings(),
            field,
        )?;
        let parts = field.declaration().parts();
        let correspondence = evidence.output_correspondence();
        correspondence
            .require_source_binding::<C::Source>()
            .map_err(D::Output)?;
        let role = OutputRoleUse {
            name: parts.role.clone(),
            posture: ApplicationMutationOutputPosture::Create,
            cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
            entity_name: field.entity_name(),
            entity_type: field.entity_type().into(),
            contract_type: field.output_contract_type().into(),
        };
        let entity = correspondence
            .bound_entity(&role)
            .map_err(D::Output)?
            .ok_or(D::MaterialUnavailable)?;
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new(&parts.aspect).ok_or(D::FieldMeaningMismatch)?,
            CanonicalFieldPath::single(FieldKey::new(&parts.field).ok_or(D::FieldMeaningMismatch)?),
        );
        evidence
            .committed_changes()
            .committed_field_ref(&self.request, entity, &locator)
            .map_err(D::Stopped)?
            .ok_or(D::MaterialUnavailable)
    }
}

impl<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>> ApplicationCorrectionSourceOutputs<S>
    for SourceOutputs<'_, '_, '_, '_, S, C>
{
    fn field(
        &self,
        field: &ApplicationCorrectionOutputField<S>,
    ) -> Result<&AspectValue, ApplicationCorrectionSourceFieldDenial> {
        read_checked(&self.failure, || self.read(field))
    }
}

pub(in crate::domain_computation::primary_graph) fn read_checked<T>(
    failure: &RefCell<Option<WorthQueryCorrectionOutputProjectionDenial>>,
    read: impl FnOnce() -> Result<T, WorthQueryCorrectionOutputProjectionDenial>,
) -> Result<T, ApplicationCorrectionSourceFieldDenial> {
    if failure.borrow().is_some() {
        return Err(ApplicationCorrectionSourceFieldDenial::Refused);
    }
    match read() {
        Ok(value) => Ok(value),
        Err(cause) => {
            *failure.borrow_mut() = Some(cause);
            Err(ApplicationCorrectionSourceFieldDenial::Refused)
        }
    }
}
