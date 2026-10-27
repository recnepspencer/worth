//! Capture declared disclosure parameters inside the original admitted handler boundary.
use crate::domain_computation::{
    application_aftermath::OriginalCorrectionDisclosure,
    primary_graph::{
        WorthQueryApplicationAttemptDenial as Denial,
        WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryPrimaryGraphApplicationRuntime,
    },
};
use std::{
    any::{Any, TypeId},
    sync::Arc,
};
use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionVisitor, ApplicationInverseBinding,
        DeclaredCorrectionBinding,
    },
    application_operation::ApplicationMutationBinding,
    application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::InstalledCorrectionBinding;

pub(super) fn capture<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    original: &Binding::Input,
) -> Result<Option<Arc<OriginalCorrectionDisclosure>>, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    let installed = runtime
        .installed_schema()
        .installed_mutation_binding::<Binding>()
        .map_err(|_| denied("original mutation binding is unavailable"))?;
    let Some(association) = installed.retain_correction_binding() else {
        return Ok(None);
    };
    if association.source_type() != TypeId::of::<Binding>() {
        return Err(denied(
            "correction capture names another native source binding",
        ));
    }
    Schema::visit_correction(
        TypeId::of::<Binding>(),
        Capture {
            execution,
            runtime,
            original,
            association,
        },
    )
    .map(|captured| Some(Arc::new(captured)))
}

struct Capture<'a, 'execution, 'authority, Schema> {
    execution: &'a worth_execution::ExecutionRequest<'execution, 'authority>,
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    original: &'a dyn Any,
    association: Arc<InstalledCorrectionBinding>,
}
impl<Schema: ApplicationSchema> ApplicationCorrectionVisitor<Schema>
    for Capture<'_, '_, '_, Schema>
{
    type Output = Result<OriginalCorrectionDisclosure, Denial>;
    fn unavailable(self) -> Self::Output {
        Err(denied("declared correction dispatch is unavailable"))
    }

    fn declared<Correction: ApplicationCorrectionBinding<Schema>>(self) -> Self::Output {
        if self.association.source_type() != TypeId::of::<Correction::Source>()
            || self.association.inverse_type()
                != TypeId::of::<ApplicationInverseBinding<Schema, Correction>>()
            || self.association.disclosure_query_type()
                != TypeId::of::<<Correction::Disclosure as ApplicationQueryBinding<Schema>>::Query>(
                )
            || self.association.declaration()
                != &DeclaredCorrectionBinding::of::<Schema, Correction>()
        {
            return Err(denied(
                "typed correction differs from its installed association",
            ));
        }
        let original = self
            .original
            .downcast_ref::<<Correction::Source as ApplicationMutationBinding<Schema>>::Input>()
            .ok_or_else(|| denied("original correction input type differs"))?;
        let disclosure =
            crate::domain_computation::primary_graph::source_disclosure::capture_parameters::<
                Schema,
                Correction::Disclosure,
            >(
                self.execution,
                self.runtime,
                std::mem::size_of::<OriginalCorrectionDisclosure>()
                    + 2 * std::mem::size_of::<usize>(),
                || {
                    Correction::disclosure_parameters(original)
                        .map_err(|_| denied("original disclosure parameter projection was refused"))
                },
            )?;
        Ok(OriginalCorrectionDisclosure::from_original(
            self.association,
            disclosure,
        ))
    }
}

fn denied(reason: &'static str) -> Denial {
    Denial::new(Kind::CorrectionCaptureDenied, reason)
}
