//! Recover exact prepared DATA through fresh disclosure and current receivers.
use super::*;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationScopeBinding, ApplicationMutationScopeResolutionMode,
    },
    application_schema::ApplicationScalarValueBinding,
};
impl WorthQueryPreparedCorrectionSource {
    pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn recover<
        Schema,
        C,
        B,
    >(
        checked: super::super::compatibility::CheckedCorrectionSource<'_, '_, '_, Schema, C, B>,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        lineage: WorthQueryCorrectionLineageBinding,
        key: &WorthQueryCorrectionCommandKey,
        factory: impl FnOnce(
            &[u8],
            worth_foundational::facade::CanonicalDigestId,
            worth_foundational::facade::ValueDecodeBounds,
        ) -> Result<
            ApplicationEncodedInput<B::InputBinding>,
            worth_query_declaration::facade::application_operation::ApplicationRetainedInputDenial,
        >,
    ) -> Result<
        (
            ApplicationEncodedInput<B::InputBinding>,
            std::sync::Arc<Self>,
        ),
        WorthQueryCorrectionDerivationDenial,
    >
    where
        Schema: ApplicationSchema,
        C: ApplicationCorrectionBinding<Schema>,
        B: ApplicationMutationBinding<Schema>,
        B::Input: serde::Serialize,
    {
        use WorthQueryCorrectionDerivationDenial as D;
        let target = checked.target();
        let execution = phase
            .execution_request_for(&target.application().runtime.product_runtime)
            .map_err(|_| D::ForeignPhase)?;
        let (_, source, _, program, _) = checked.borrow_parts();
        let material = source
            .occurrence()
            .home_material()
            .ok_or(D::UnsupportedSourceMeaning)?;
        let (_, snapshot) = target
            .application()
            .runtime
            .primary_provider
            .recover_correction_command(material.capture.original, key.identity())
            .map_err(D::Material)?
            .ok_or(D::SemanticMaterialUnavailable)?;
        let binding = target
            .application()
            .installed_schema()
            .installed_mutation_binding::<B>()
            .map_err(|_| D::UnsupportedTargetProgram)?;
        let actor = target.resolve_authenticated_principal(phase, binding.principal_binding(), principal, request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary).map_err(D::Principal)?;
        let selector = crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector::readmit_recorded(
            &snapshot.scope.location.aspect, &snapshot.scope.location.fields, snapshot.scope.value.clone(), snapshot.scope.subject, &execution).map_err(D::Material)?;
        if <B::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::RESOLUTION_MODE
            == ApplicationMutationScopeResolutionMode::PrincipalIdentity
        {
            let identity = B::PrincipalIdentityBinding::encode(actor.principal_identity())
                .map_err(|_| D::OriginalSubjectChanged)?;
            if &identity != selector.value() {
                return Err(D::OriginalSubjectChanged);
            }
        }
        let scope = target
            .resolve_retained_query_scope(&execution, B::scope_field(), &selector, request)
            .map_err(D::Scope)?;
        if actor.principal_entity_id() != snapshot.actor
            || scope.entity_id() != snapshot.scope.subject
        {
            return Err(D::OriginalSubjectChanged);
        }
        let intent = WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(
            target, source, key, &lineage,
        );
        if snapshot.action != B::IDENTITY
            || snapshot.intent != intent.identity()
            || snapshot.target.program != *program.revision().as_bytes()
            || !snapshot.target.matches(target.product().observation())
        {
            return Err(D::Material(
                crate::facade::correction_material::QueryCorrectionMaterialDenial::TargetChanged,
            ));
        }
        let input = super::super::material::recorded_input::recover_bytes(
            &snapshot.input,
            snapshot.input_identity,
            factory,
        )?;
        let prepared = Self::retain_resolved::<Schema, C, B>(
            checked,
            phase,
            None,
            lineage,
            key,
            &input,
            actor.principal_entity_id(),
            scope.retain_scope_selector(),
            None,
        )?;
        Ok((input, std::sync::Arc::new(prepared)))
    }
}
