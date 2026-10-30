//! Fresh Source admission under its own typed binding on the Parent Product.
use super::{
    adapter::InstalledIndexProvider,
    source_completion::{complete_source, SourceDenial},
    source_image::{SourceBinding, SourceImage},
};
use crate::domain_computation::{
    primary_graph::{
        application_query::{
            one_shot::outcome::material::MaterialCompletion,
            WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryControls,
            WorthQueryObservedScopeSelector,
        },
        source_facts::SourceFactsAllocation,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
        WorthQuerySelectedProductOperation,
    },
    provider_session::WorthQueryManagedGraphWorkSession,
};
use std::{num::NonZeroUsize, sync::Arc};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryParameterSet,
        ApplicationQueryScopeBinding, ApplicationQueryScopeResolutionMode,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

mod temporary;

pub(super) struct AcquisitionDenial {
    pub(super) cause: AcquisitionCause,
    // Diagnostic names may share the newly admitted source/temporary backing.
    // Preserve both real owners through the error's final lifetime.
    _source: Arc<SourceFactsAllocation>,
    _temporary: Option<Arc<SourceFactsAllocation>>,
}

pub(super) enum AcquisitionCause {
    ForeignContext,
    Execution(Cause),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Admission(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial),
    Source(SourceDenial),
}
impl From<Cause> for AcquisitionCause {
    fn from(cause: Cause) -> Self {
        Self::Execution(cause)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn acquire<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    adapter: &dyn InstalledIndexProvider<S, I>,
    parent: &WorthQueryManagedGraphWorkSession,
    association: &WorthQueryInstalledManagedIndexParent,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    selector: &WorthQueryObservedScopeSelector,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    allocation: Arc<SourceFactsAllocation>,
) -> Result<Arc<SourceImage<S, I>>, AcquisitionDenial> {
    let mut temporary = None;
    let result = (|| {
        let storage = MaterialCompletion::new(execution, &allocation);
        storage.work(1)?;
        let context = parent
            .retain_query_source_context()
            .map_err(|_| AcquisitionCause::ForeignContext)?;
        let installed = adapter.installed();
        let access_contract = association.access_contract();
        if !std::ptr::eq(application, selected.application())
            || context.runtime() != application.runtime.authority_identity()
            || !installed.is_installed_in(application.installed_schema())
            || !installed.admits_association(association)
            || parent.obligation() != access_contract.parent_obligation()
            || parent.binding() != association.binding_identity()
            || selector.subject() != context.scope()
        {
            return Err(AcquisitionCause::ForeignContext);
        }
        if !parent.admits_managed_product(
            selected.product().read_lease_ref(),
            &mut |units| storage.work(units),
            || Cause::WorkCounterOverflow,
        )? {
            return Err(AcquisitionCause::ForeignContext);
        }
        let source = installed.source_read();
        if source.query().graph_obligations().identity() != access_contract.source_obligation() {
            return Err(AcquisitionCause::ForeignContext);
        }
        let principal = selected
            .resolve_authenticated_principal_admitted(
                execution,
                source.principal_binding(),
                external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(AcquisitionCause::Principal)?;
        if principal.principal_entity_id() != context.principal() {
            return Err(AcquisitionCause::ForeignContext);
        }
        let principal_scope = <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::RESOLUTION_MODE;
        if principal_scope == ApplicationQueryScopeResolutionMode::PrincipalIdentity
            && !principal.freshness().matches_principal_identity(
                selector.value(),
                &mut |units| storage.work(units),
                || Cause::WorkCounterOverflow,
            )?
        {
            return Err(AcquisitionCause::ForeignContext);
        }
        let field = SourceBinding::<S, I>::scope_field();
        // This local owner pays the resolver's new value/locator/name copies and
        // stays alive until the borrowed typed scope and Source plan are gone.
        temporary = Some(temporary::scope(
            execution,
            field.entity(),
            field.aspect(),
            field.field(),
            selector,
        )?);
        let scope = selected
            .resolve_retained_query_scope(execution, field, selector, request)
            .map_err(AcquisitionCause::Scope)?;
        if scope.entity_id() != context.scope() || scope.entity_id() != selector.subject() {
            return Err(AcquisitionCause::ForeignContext);
        }
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let selected = selected
            .retain_selection(execution)
            .map_err(AcquisitionCause::Product)?;
        let (_, product, basis) = selected.into_parts();
        let query_maximum = NonZeroUsize::new(source.query().read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let maximum = source.limits().maximum_results().min(query_maximum);
        let controls =
            WorthQueryApplicationQueryControls::product_one_shot(product, basis, maximum, request);
        let plan = application
            .admit_application_query(
                execution,
                source.query(),
                &access,
                ApplicationQueryParameterSet::new(),
                controls,
            )
            .map_err(AcquisitionCause::Admission)?;
        let result = complete_source(
            execution,
            application,
            installed.clone(),
            plan,
            Arc::clone(&allocation),
        )
        .map_err(AcquisitionCause::Source);
        drop(scope);
        result
    })();
    match result {
        Ok(image) => {
            drop(temporary);
            Ok(image)
        }
        Err(cause) => Err(AcquisitionDenial {
            cause,
            _source: allocation,
            _temporary: temporary,
        }),
    }
}
