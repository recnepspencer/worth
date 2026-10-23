//! Source material and exact target retained between inverse preparation and publication.
use super::*;
mod workflow;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionLineageBinding,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::workflow::correction::PreparedWorkflowCorrection;
pub(super) use workflow::prepare_workflow;
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationScopeResolution,
};
use worth_relational::facade::{
    history::{BranchId, RelationalCommitReceipt},
    identity::EntityId,
};

/// Native correction meaning retained from derivation through ordinary publication.
///
/// It binds the exact source, action input, caller, scope, target and lineage to
/// their checked program interpretation. Its ancestry pins preserve optional
/// material until publication releases this preparation; it grants no standalone
/// mutation or recovery authority.
pub struct WorthQueryPreparedCorrectionSource {
    authored_identity: Option<super::authored_identity::PreparedAuthoredIdentity>,
    workflow: Option<PreparedWorkflowCorrection>,
    ancestry: ProductBranchAncestorSelection,
    /// Keeps selected and active reversal material pinned for the prepared action's lifetime.
    _alternative_ancestry: Option<(ProductBranchAncestorSelection, ProductBranchAncestorSelection)>,
    lineage: WorthQueryCorrectionLineageBinding,
    material: WorthQueryPrimaryGraphCommittedApplication,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    actor: EntityId,
    scope: EntityId,
    action_input: [u8; 32],
    action_binding: std::any::TypeId,
    target_parent: RelationalCommitReceipt,
    target_branch: BranchId,
    intent: WorthQueryCorrectionIntent,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
    target_program: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation,
}

impl WorthQueryPreparedCorrectionSource {
    pub(super) fn retain<
        Schema: ApplicationSchema,
        C: ApplicationCorrectionBinding<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    >(
        target: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        ancestry: ProductBranchAncestorSelection,
        alternative_ancestry: Option<(
            ProductBranchAncestorSelection,
            ProductBranchAncestorSelection,
        )>,
        lineage: WorthQueryCorrectionLineageBinding,
        key: &WorthQueryCorrectionCommandKey,
        input: &ApplicationEncodedInput<Binding::InputBinding>,
        scope: Binding::ScopeBinding,
        target_program: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation,
        required_root: Option<crate::equality_type_id::EqualityTypeId>,
        workflow: Option<PreparedWorkflowCorrection>,
    ) -> Result<Self, WorthQueryCorrectionDerivationDenial>
    where
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
    {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let binding = target
            .application()
            .installed_schema()
            .installed_mutation_binding::<Binding>()
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
        let execution = phase
            .execution_request_for(&target.application().product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        let authored_identity = if lineage.role() == WorthQueryAftermathCausalRole::RecordedReverse
        {
            super::authored_identity::PreparedAuthoredIdentity::prepare::<Schema, C>(
                source, &execution,
            )?
        } else {
            None
        };
        let intent = WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(
            target, source, key, &lineage,
        );
        Ok(Self {
            authored_identity,
            workflow,
            ancestry,
            _alternative_ancestry: alternative_ancestry,
            lineage,
            target_program,
            required_root,
            material: source.occurrence().clone(),
            runtime: target.application().runtime.authority_identity(),
            actor: actor.principal_entity_id(),
            scope: scope.entity_id(),
            action_input: *input.identity(),
            action_binding: std::any::TypeId::of::<Binding>(),
            target_parent,
            target_branch: basis.identity().branch_id().clone(),
            intent,
        })
    }

    /// The immutable descriptive command intent carried into ordinary idempotency admission.
    pub fn intent(&self) -> WorthQueryCorrectionIntent {
        self.intent
    }
    pub(crate) fn lineage_binding(&self) -> &WorthQueryCorrectionLineageBinding {
        &self.lineage
    }
    pub(in crate::domain_computation::primary_graph) fn validate_workflow_candidate(
        &self,
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        match &self.workflow {
            Some(workflow) => workflow.validate_candidate(batch),
            None => Ok(()),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn validate_authored_candidate<'a>(
        &self,
        roles: impl Iterator<Item = (&'a str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &'a str, &'a worth_relational::facade::transactions::EntityReference)>,
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<(), super::AuthoredCorrectionCandidateDenial> {
        match &self.authored_identity {
            Some(identity) => identity.validate_candidate(roles, batch, request),
            None => Ok(()),
        }
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
    pub(crate) fn is_action_binding<Binding: 'static>(&self) -> bool {
        self.action_binding == std::any::TypeId::of::<Binding>()
    }
    pub(crate) fn matches_input(&self, identity: &[u8; 32]) -> bool {
        self.action_input == *identity
    }
    pub(crate) fn requires_root(
        &self,
        root: Option<crate::equality_type_id::EqualityTypeId>,
    ) -> bool {
        self.required_root == root
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
            && self.authored_identity == other.authored_identity
            && self.workflow == other.workflow
            && self.required_root == other.required_root
            && self.target_program.same_support_as(&other.target_program)
            && self.actor == other.actor
            && self.scope == other.scope
            && self.action_input == other.action_input
            && self.action_binding == other.action_binding
            && self.lineage == other.lineage
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
