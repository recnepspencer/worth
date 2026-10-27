//! The actual source binding dispatches its original input once, before acceptance.
use super::{capture_parameters, OriginalSourceInspectionDisclosure};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryObservedScopeSelector, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::{any::TypeId, sync::Arc};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationSourceDisclosureBinding,
        ApplicationSourceDisclosureVisitor, DeclaredApplicationSourceDisclosure,
    },
    application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::InstalledSourceDisclosureBinding;

pub(in crate::domain_computation::primary_graph) fn capture_inspection<Schema, Source>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    original: &Source::Input,
    scope: Option<&Arc<WorthQueryObservedScopeSelector>>,
) -> Result<Option<Arc<OriginalSourceInspectionDisclosure>>, Denial>
where
    Schema: ApplicationSchema,
    Source: ApplicationMutationBinding<Schema>,
{
    let installed = runtime
        .installed_schema()
        .installed_mutation_binding::<Source>()
        .map_err(|_| denied("original mutation binding is unavailable"))?;
    let Some(association) = installed.retain_inspection_disclosure() else {
        return Ok(None);
    };
    if association.source_type() != TypeId::of::<Source>() {
        return Err(denied(
            "source inspection names another native mutation binding",
        ));
    }
    let scope = scope.ok_or_else(|| denied("original source scope is unavailable"))?;
    Source::visit_source_disclosure(Capture::<Schema, Source> {
        execution,
        runtime,
        original,
        scope,
        association,
    })
    .map(|captured| Some(Arc::new(captured)))
}

struct Capture<
    'a,
    'execution,
    'authority,
    Schema: ApplicationSchema,
    Source: ApplicationMutationBinding<Schema>,
> {
    execution: &'a worth_execution::ExecutionRequest<'execution, 'authority>,
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    original: &'a Source::Input,
    scope: &'a Arc<WorthQueryObservedScopeSelector>,
    association: Arc<InstalledSourceDisclosureBinding>,
}

impl<Schema: ApplicationSchema, Source: ApplicationMutationBinding<Schema>>
    ApplicationSourceDisclosureVisitor<Schema, Source> for Capture<'_, '_, '_, Schema, Source>
{
    type Output = Result<OriginalSourceInspectionDisclosure, Denial>;
    fn unavailable(self) -> Self::Output {
        Err(denied("declared source inspection dispatch is unavailable"))
    }
    fn declared<Binding: ApplicationSourceDisclosureBinding<Schema, Source = Source>>(
        self,
    ) -> Self::Output {
        if self.association.declaration()
            != &DeclaredApplicationSourceDisclosure::of::<Schema, Binding>()
            || self.association.query_type()
                != TypeId::of::<<Binding::Disclosure as ApplicationQueryBinding<Schema>>::Query>()
        {
            return Err(denied(
                "typed source inspection differs from installed meaning",
            ));
        }
        let disclosure = capture_parameters::<Schema, Binding::Disclosure>(
            self.execution,
            self.runtime,
            std::mem::size_of::<OriginalSourceInspectionDisclosure>()
                + 2 * std::mem::size_of::<usize>(),
            || {
                Binding::disclosure_parameters(self.original)
                    .map_err(|_| denied("source inspection parameter projection was refused"))
            },
        )?;
        Ok(OriginalSourceInspectionDisclosure::captured(
            self.association,
            Arc::clone(self.scope),
            disclosure,
        ))
    }
}

fn denied(reason: &'static str) -> Denial {
    Denial::new(Kind::SourceDisclosureCaptureDenied, reason)
}
