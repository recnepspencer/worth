//! Source material and exact target retained between inverse preparation and publication.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_query_declaration::facade::application_aftermath::ApplicationInverseBinding;
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationScopeResolution,
};
use worth_relational::facade::{
    history::{BranchId, RelationalCommitReceipt},
    identity::EntityId,
};

pub struct WorthQueryPreparedCorrectionSource {
    ancestry: ProductBranchAncestorSelection,
    material: WorthQueryPrimaryGraphCommittedApplication,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    actor: EntityId,
    scope: EntityId,
    inverse_input: [u8; 32],
    inverse: std::any::TypeId,
    target_parent: RelationalCommitReceipt,
    target_branch: BranchId,
    intent: WorthQueryCorrectionIntent,
    target_program: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation,
}

impl WorthQueryPreparedCorrectionSource {
    pub(super) fn retain<Schema: ApplicationSchema, C: ApplicationCorrectionBinding<Schema>>(
        target: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        ancestry: ProductBranchAncestorSelection,
        key: &WorthQueryCorrectionCommandKey,
        input: &ApplicationEncodedInput<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>,
        scope: <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding,
        target_program: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation,
    ) -> Result<Self, WorthQueryCorrectionDerivationDenial>
    where
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let binding = target
            .application()
            .installed_schema()
            .installed_mutation_binding::<ApplicationInverseBinding<Schema, C>>()
            .map_err(|_| Denial::UnsupportedTargetProgram)?;
        let actor = target.resolve_authenticated_principal(phase, binding.principal_binding(), principal, request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary)
            .map_err(Denial::Principal)?;
        let (field, value) = scope.into_field_parts(actor.principal_identity());
        let scope = target.resolve_entity(phase, field, value, request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary)
            .map_err(Denial::Scope)?;
        let basis = target.product().relational_basis();
        let target_parent = basis
            .observation()
            .commit_receipt()
            .cloned()
            .ok_or(Denial::MissingTargetParent)?;
        Ok(Self {
            ancestry,
            target_program,
            material: source.occurrence().clone(),
            runtime: target.application().runtime.authority_identity(),
            actor: actor.principal_entity_id(),
            scope: scope.entity_id(),
            inverse_input: *input.identity(),
            inverse: std::any::TypeId::of::<ApplicationInverseBinding<Schema, C>>(),
            target_parent,
            target_branch: basis.identity().branch_id().clone(),
            intent: WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(
                target, source, key,
            ),
        })
    }

    pub fn intent(&self) -> WorthQueryCorrectionIntent {
        self.intent
    }

    pub(crate) fn source_commit(&self) -> &RelationalCommitReceipt {
        self.material.commit_reference()
    }
    pub(crate) fn source_world(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.material
            .committed_product_publication()
            .composite_commit()
    }
    pub(crate) fn target_parent(&self) -> &RelationalCommitReceipt {
        &self.target_parent
    }
    pub(crate) fn target_branch(&self) -> &BranchId {
        &self.target_branch
    }
    pub(crate) fn is_inverse<Binding: 'static>(&self) -> bool {
        self.inverse == std::any::TypeId::of::<Binding>()
    }
    pub(crate) fn matches_input(&self, identity: &[u8; 32]) -> bool {
        self.inverse_input == *identity
    }

    pub(crate) fn matches_admission<Schema, Operation, Input, Scope>(
        &self,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> bool {
        self.runtime == admission.runtime_authority()
            && admission.operation_scope_binding().principal()
                == crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(self.actor)
            && admission.operation_scope_binding().scope()
                == crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(self.scope)
            && admission.graph_work().mutation_product().is_some_and(|product|
                WorthQueryProductBranchReadIdentity::from_observation(product.observation())
                    == WorthQueryProductBranchReadIdentity::from_observation(self.ancestry.target()))
    }
}

impl std::fmt::Debug for WorthQueryPreparedCorrectionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryPreparedCorrectionSource")
            .field("source", self.source_world())
            .field("target", self.ancestry.target().selected_commit())
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryPreparedCorrectionSource {
    fn eq(&self, other: &Self) -> bool {
        self.runtime == other.runtime
            && self.target_program.same_support_as(&other.target_program)
            && self.actor == other.actor
            && self.scope == other.scope
            && self.inverse_input == other.inverse_input
            && self.inverse == other.inverse
            && self.intent == other.intent
            && self.target_parent == other.target_parent
            && self.target_branch == other.target_branch
            && self.source_world() == other.source_world()
            && self.source_commit() == other.source_commit()
            && WorthQueryProductBranchReadIdentity::from_observation(self.ancestry.target())
                == WorthQueryProductBranchReadIdentity::from_observation(other.ancestry.target())
    }
}
impl Eq for WorthQueryPreparedCorrectionSource {}
