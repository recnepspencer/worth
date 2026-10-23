//! Disclose one original subject under the target's current query authority.
use super::{WorthQuerySelectedCorrectionSource, WorthQuerySelectedProductOperation};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryControls, WorthQueryPrincipalResolutionMode,
};
use std::{any::TypeId, num::NonZeroUsize};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionDerivationDenial,
        ApplicationInverseBinding, DeclaredCorrectionBinding,
    },
    application_query::{
        ApplicationQueryBinding, ApplicationQueryScopeBinding, ApplicationQueryScopeResolutionMode,
    },
    application_schema::{ApplicationScalarValueBinding, ApplicationSchema},
};

#[derive(Debug)]
/// A source-disclosure, semantic-comparison or inverse-derivation refusal.
/// Current principal, execution and resource causes remain typed in their original owner.
pub enum WorthQueryCorrectionDerivationDenial {
    ForeignPhase,
    ForeignSelection,
    TargetIncarnationChanged,
    UnsupportedSourceMeaning,
    OriginalSubjectChanged,
    SubjectNotDisclosed,
    MissingPreImage,
    SourceOutput(super::material::WorthQueryCorrectionOutputProjectionDenial),
    SemanticMaterialUnavailable,
    MissingTargetParent,
    InputCapture(crate::domain_computation::primary_graph::WorthQueryOperationInputCaptureDenial),
    CommandKey(worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial),
    SourceDiverged,
    UnsupportedTargetProgram,
    ProgramComparison(
        worth_query_installation::facade::WorthQueryProgramAdoptionRequirementsDenial,
    ),
    ProgramMeaningChanged(
        Box<worth_query_installation::facade::WorthQueryProgramAdoptionRequirements>,
    ),
    InverseAction(worth_query_installation::facade::WorthQueryActionResolutionDenial),
    RequiredOutputReconciliation,
    Workflow(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    TargetProgram(
        crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial,
    ),
    Basis(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
    DependencyComparison(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    Ancestry(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::ProductBranchAncestorSelectionDenial,
        >,
    ),
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Admission(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial),
    Read(crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenial),
    Domain(ApplicationCorrectionDerivationDenial),
}

type Disclosure<Schema, C> = <C as ApplicationCorrectionBinding<Schema>>::Disclosure;

pub(super) fn disclose_correction_source<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
) -> Result<
    worth_runtime_world::facade::ProductBranchAncestorSelection,
    WorthQueryCorrectionDerivationDenial,
>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    use WorthQueryCorrectionDerivationDenial as Denial;
    let application = target.application();
    if !std::ptr::eq(application, source.application()) {
        return Err(Denial::ForeignSelection);
    }
    if target.product().observation().lifecycle_incarnation()
        != source.ancestry().target().lifecycle_incarnation()
    {
        return Err(Denial::TargetIncarnationChanged);
    }
    let execution = phase
        .execution_request_for(&application.product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    let ancestry = application
        .product_runtime
        .owner
        .observation_port()
        .select_product_branch_ancestor(
            &execution,
            target.product().observation(),
            source.ancestry().source().identity(),
            maximum_history,
        )
        .map_err(Denial::Ancestry)?;
    let original = source.occurrence().commit_evidence().original_authority();
    let retained = original
        .retained_governed_input_carrier()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let disclosure = retained
        .correction()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let association = disclosure.association();
    let original_program = original
        .original_program()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    if !original_program.declares_binding(association.source_type()) {
        return Err(Denial::UnsupportedSourceMeaning);
    }
    if association.source_type() != TypeId::of::<C::Source>()
        || association.inverse_type() != TypeId::of::<ApplicationInverseBinding<Schema, C>>()
        || association.disclosure_query_type()
            != TypeId::of::<<Disclosure<Schema, C> as ApplicationQueryBinding<Schema>>::Query>()
        || association.declaration() != &DeclaredCorrectionBinding::of::<Schema, C>()
    {
        return Err(Denial::UnsupportedSourceMeaning);
    }
    let installed = application
        .installed_schema()
        .installed_query_binding::<Disclosure<Schema, C>>()
        .map_err(|_| Denial::UnsupportedSourceMeaning)?;
    if installed.query().identity() != disclosure.query() {
        return Err(Denial::UnsupportedSourceMeaning);
    }
    let principal = target
        .resolve_authenticated_principal_admitted(
            &execution,
            installed.principal_binding(),
            principal,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(Denial::Principal)?;
    let selector = retained
        .original_scope()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    if <<Disclosure<Schema, C> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::RESOLUTION_MODE
        == ApplicationQueryScopeResolutionMode::PrincipalIdentity
    {
        let identity = <Disclosure<Schema, C> as ApplicationQueryBinding<Schema>>::PrincipalIdentityBinding::encode(principal.principal_identity())
            .map_err(|_| Denial::OriginalSubjectChanged)?;
        if &identity != selector.value() { return Err(Denial::OriginalSubjectChanged); }
    }
    let scope = target
        .resolve_retained_query_scope(
            &execution,
            Disclosure::<Schema, C>::scope_field(),
            selector,
            request,
        )
        .map_err(Denial::Scope)?;
    if scope.entity_id() != selector.subject() {
        return Err(Denial::OriginalSubjectChanged);
    }
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let selected = target
        .retain_selection(&execution)
        .map_err(Denial::Product)?;
    let (_, product, basis) = selected.into_parts();
    let controls = WorthQueryApplicationQueryControls::product_one_shot(
        product,
        basis,
        maximum_results,
        request,
    );
    let plan = application
        .admit_correction_disclosure(
            &execution,
            installed.query(),
            &access,
            disclosure.parameters(),
            controls,
        )
        .map_err(Denial::Admission)?;
    if !application
        .execute_correction_disclosure(&execution, plan, selector.subject())
        .map_err(Denial::Read)?
    {
        return Err(Denial::SubjectNotDisclosed);
    }
    Ok(ancestry)
}
