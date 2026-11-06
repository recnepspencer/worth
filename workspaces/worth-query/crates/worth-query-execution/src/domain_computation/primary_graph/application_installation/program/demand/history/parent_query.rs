//! Actual parent queries share C admission while retaining distinct parent custody.
use super::dependent::{DiscoveryBinding, QueryRows};
use super::*;
mod verification;
use crate::domain_computation::primary_graph::{
    application_query::{ApplicationQueryNativeAccess, SourceQueryReaderAccess},
    product_operation::document_history::{
        navigation::HistoryDependentParent, HistoryDependentSource,
    },
    WorthQueryApplicationReadObservation, WorthQueryHistoryDependentOutputRead,
    WorthQueryHistoryOutputReadDenial as Denial, WorthQueryVerifiedHistoryParent,
};
pub(in crate::domain_computation::primary_graph) use verification::HistoryAdmittedChildSource;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryIntent, ApplicationQueryScopeResolution,
};

mod parent;
use parent::Parent;
mod discovery;
#[derive(Clone, Copy)]
enum ParentQueryPurpose {
    ChildSource,
    Discovery,
}
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn execute_owned_history_dependent_output_query<'authority, 'phase, 'scope, D, C, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledHistoryProgramOutput<S, P, D>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        basis: &'authority std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &'authority WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: B::Input,
    ) -> Result<
        WorthQueryHistoryDependentOutputRead<'authority, 'phase, 'scope, B::Query, QueryRows<S, B>>,
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        QueryRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        self.execute_parent_query::<D, C, B>(
            phase,
            Parent::Settled(parent),
            ParentQueryPurpose::ChildSource,
            authority,
            basis,
            minimum,
            external,
            request,
            intent,
        )
    }
    pub fn execute_verified_history_dependent_output_query<'authority, 'phase, 'scope, D, C, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        basis: &'authority std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &'authority WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: B::Input,
    ) -> Result<
        WorthQueryHistoryDependentOutputRead<'authority, 'phase, 'scope, B::Query, QueryRows<S, B>>,
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        QueryRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        self.execute_parent_query::<D, C, B>(
            phase,
            Parent::Verified(parent),
            ParentQueryPurpose::ChildSource,
            authority,
            basis,
            minimum,
            external,
            request,
            intent,
        )
    }
    fn execute_parent_query<'authority, 'phase, 'scope, D, C, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        parent: Parent<'_, S, P, D>,
        purpose: ParentQueryPurpose,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        basis: &'authority std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &'authority WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: B::Input,
    ) -> Result<
        WorthQueryHistoryDependentOutputRead<'authority, 'phase, 'scope, B::Query, QueryRows<S, B>>,
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        QueryRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let (result, meaning, query) = self.with_parent_query_admission::<D, C, B, _>(
            phase,
            &parent,
            purpose,
            authority,
            basis,
            minimum,
            external,
            request,
            intent,
            |reader, execution, plan, native, _| {
                reader
                    .read_admitted_result::<B>(execution, plan, native)
                    .map_err(Denial::from)
            },
        )?;
        let parent = match parent {
            Parent::Settled(parent) => {
                HistoryDependentParent::Settled(std::sync::Arc::clone(&parent.settled.retained))
            }
            Parent::Verified(parent) => {
                HistoryDependentParent::Verified(std::sync::Arc::clone(parent.custody()))
            }
        };
        Ok(WorthQueryHistoryDependentOutputRead {
            source: HistoryDependentSource {
                authority,
                parent,
                connection: crate::equality_type_id::EqualityTypeId::of::<C>().into_external(),
                source: basis,
                minimum,
                output_binding: meaning.output_binding,
                discovery_binding: meaning.discovery_binding,
                executed_binding: meaning.executed_binding,
                query,
            },
            result,
        })
    }
    fn with_parent_query_admission<D, C, B, T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        parent: &Parent<'_, S, P, D>,
        purpose: ParentQueryPurpose,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        basis: &std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: B::Input,
        consume: impl for<'plan> FnOnce(
            &SourceQueryReaderAccess<'_, S>, &worth_execution::ExecutionRequest<'_, '_>,
            crate::domain_computation::primary_graph::application_query::output_source_observation::SourceReadPlan<'plan, S, B>,
            &mut ApplicationQueryNativeAccess<'_>, Option<Artifact>,
        ) -> Result<T, Denial>,
    ) -> Result<
        (
            T,
            ParentQueryMeaning,
            worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        ),
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        QueryRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let runtime = self.runtime();
        authority
            .validate(phase, runtime, request)
            .map_err(Denial::Authority)?;
        if !authority.uses_owned_model_graph() {
            return Err(foreign());
        }
        let execution = phase.checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        let schema = runtime.installed_schema();
        let binding = schema
            .installed_query_binding::<B>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        let meaning = self.parent_query_meaning::<D, C, B>(&binding, purpose)?;
        let selected = runtime.select_application_read_observation(phase, basis)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?
            .prepare_shared_query_basis().map_err(|(_, cause)| Denial::Resource(cause))?;
        let current = runtime.on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(authority.current().observation().lifecycle_incarnation()))
            .select_with_request(&execution)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?
            .prepare_shared_query_basis().map_err(|(_, cause)| Denial::Resource(cause))?;
        let reader = SourceQueryReaderAccess::new(
            runtime,
            &selected,
            Some(&current),
            Some(authority.producer_invocation().for_family(meaning.family)),
            external,
            request,
        )
        .map_err(Denial::Demand)?;
        let context = std::cell::Cell::default();
        let result = reader.with_plan::<B, _, Denial>(
            &execution,
            intent,
            &context,
            &mut ApplicationQueryNativeAccess::acquire(),
            |plan, native| {
                // C READ is admitted before parent/basis diagnostics or D projection.
                let artifact = parent.validate::<C>(self, &execution, authority, basis, minimum)?;
                consume(&reader, &execution, plan, native, artifact)
            },
        )?;
        Ok((result, meaning, binding.query().identity().clone()))
    }
    fn parent_query_meaning<D, C, B>(
        &self,
        binding: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding<
            S,
            B,
        >,
        purpose: ParentQueryPurpose,
    ) -> Result<ParentQueryMeaning, Denial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
    {
        let schema = self.runtime().installed_schema();
        let output = schema
            .installed_query_binding::<Source<S, ConnectionDemand<S, C>>>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        let discovery = schema
            .installed_query_binding::<DiscoveryBinding<S, C>>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        let executed_binding = crate::equality_type_id::EqualityTypeId::of::<B>().into_external();
        let output_binding =
            crate::equality_type_id::EqualityTypeId::of::<Source<S, ConnectionDemand<S, C>>>()
                .into_external();
        let discovery_binding =
            crate::equality_type_id::EqualityTypeId::of::<DiscoveryBinding<S, C>>().into_external();
        let family = if matches!(purpose, ParentQueryPurpose::ChildSource)
            && executed_binding == output_binding
            && binding.query().identity() == output.query().identity()
        {
            <Family<S, ConnectionDemand<S, C>> as WorthQueryProducerOutputFamily<S>>::IDENTITY
        } else if matches!(purpose, ParentQueryPurpose::Discovery)
            && executed_binding == discovery_binding
            && binding.query().identity() == discovery.query().identity()
        {
            <Family<S, D> as WorthQueryProducerOutputFamily<S>>::IDENTITY
        } else {
            return Err(foreign());
        };
        Ok(ParentQueryMeaning {
            family,
            output_binding,
            discovery_binding,
            executed_binding,
        })
    }
}

fn foreign() -> Denial {
    Denial::Demand(WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
        "owned history parent query",
    ))
}
fn installation(
    kind: worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(kind), "owned history parent query")
}

/// Installed query correspondence and its diagnostic subject, not read authority.
struct ParentQueryMeaning {
    family: &'static str,
    output_binding: std::any::TypeId,
    discovery_binding: std::any::TypeId,
    executed_binding: std::any::TypeId,
}
