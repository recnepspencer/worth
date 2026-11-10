//! Source material and exact target retained between inverse preparation and publication.
use super::*;
mod custody;
mod directory_admission;
mod home;
mod recovery;
mod relation_inverse;
mod workflow;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionLineageBinding,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::workflow::correction::PreparedWorkflowCorrection;
use custody::PreparedCorrectionCustody;
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
/// their checked program interpretation. Memory retains ancestry. A Home owner
/// records the mandatory material use and its transfer to an actual publication.
/// Owning close durably discharges unperformed preparations; they never become
/// first use after reopen. This value grants no standalone mutation authority.
pub struct WorthQueryPreparedCorrectionSource {
    coupled_capture: Option<fn(&super::occurrence::CorrectionOccurrence, &worth_execution::ExecutionRequest<'_, '_>) -> crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands>,
    content: worth_relational::facade::content::RelationalContentAttachments,
    authored_identity: Option<super::authored_identity::PreparedAuthoredIdentity>,
    workflow: Option<PreparedWorkflowCorrection>,
    custody: PreparedCorrectionCustody,
    target: WorthQueryProductBranchReadIdentity,
    lineage: WorthQueryCorrectionLineageBinding,
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
    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
    pub(super) fn retain<
        Schema: ApplicationSchema,
        C: ApplicationCorrectionBinding<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    >(
        checked: super::compatibility::CheckedCorrectionSource<'_, '_, '_, Schema, C, Binding>,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        alternative_ancestry: Option<(
            ProductBranchAncestorSelection,
            ProductBranchAncestorSelection,
        )>,
        lineage: WorthQueryCorrectionLineageBinding,
        key: &WorthQueryCorrectionCommandKey,
        input: &ApplicationEncodedInput<Binding::InputBinding>,
        scope: Binding::ScopeBinding,
        workflow: Option<PreparedWorkflowCorrection>,
    ) -> Result<Self, WorthQueryCorrectionDerivationDenial>
    where
        Binding::Input: serde::Serialize,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
    {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let target = checked.target();
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
        Self::retain_resolved::<Schema, C, Binding>(
            checked,
            phase,
            alternative_ancestry,
            lineage,
            key,
            input,
            actor.principal_entity_id(),
            scope.retain_scope_selector(),
            workflow,
        )
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Exact installed type lookup never persists TypeId"
    )]
    fn retain_resolved<Schema, C, Binding>(
        checked: super::compatibility::CheckedCorrectionSource<'_, '_, '_, Schema, C, Binding>,
        phase: &WorthQueryAdvancementPhase<'_>,
        alternative_ancestry: Option<(
            ProductBranchAncestorSelection,
            ProductBranchAncestorSelection,
        )>,
        lineage: WorthQueryCorrectionLineageBinding,
        key: &WorthQueryCorrectionCommandKey,
        input: &ApplicationEncodedInput<Binding::InputBinding>,
        actor: EntityId,
        scope: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector,
        >,
        workflow: Option<PreparedWorkflowCorrection>,
    ) -> Result<Self, WorthQueryCorrectionDerivationDenial>
    where
        Schema: ApplicationSchema,
        C: ApplicationCorrectionBinding<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: serde::Serialize,
    {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let (target, source, ancestry, target_program, required_root) = checked.into_parts();
        directory_admission::admit(target, source, &ancestry, &lineage, required_root.is_some())?;
        let basis = target.product().relational_basis();
        let target_parent = basis
            .observation()
            .commit_receipt()
            .cloned()
            .ok_or(Denial::MissingTargetParent)?;
        let execution = phase
            .execution_request_for(&target.application().runtime.product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        let authored_identity = if lineage.role() == WorthQueryAftermathCausalRole::RecordedReverse
        {
            super::authored_identity::PreparedAuthoredIdentity::prepare::<Schema, C>(
                source, &execution,
            )?
        } else {
            None
        };
        let mut content = source
            .occurrence()
            .preimage_content()
            .cloned()
            .unwrap_or_default();
        let outputs =
            super::material::source_outputs::SourceOutputs::<Schema, C>::new(source, execution)
                .retained_content()
                .map_err(Denial::SourceOutput)?;
        content
            .attach_retained(&execution, &outputs)
            .map_err(|cause| {
                Denial::SourceOutput(
                    super::material::WorthQueryCorrectionOutputProjectionDenial::Content(cause),
                )
            })?;
        let intent = WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(
            target, source, key, &lineage,
        );
        let target_identity =
            WorthQueryProductBranchReadIdentity::from_observation(ancestry.target());
        let custody = match source.occurrence().home_material() {
            Some(_) => home::prepare::<Schema, Binding>(
                target,
                source,
                input,
                &intent,
                key,
                actor,
                &scope,
                &target_program,
                &execution,
            )?,
            None => PreparedCorrectionCustody::Memory {
                _ancestry: ancestry,
                _alternatives: alternative_ancestry,
                material: source.occurrence().clone(),
            },
        };
        Ok(Self {
            coupled_capture: (lineage.role() == WorthQueryAftermathCausalRole::RecordedReverse)
                .then_some(super::material::capture_coupled::<Schema, C>),
            content,
            authored_identity,
            workflow,
            custody,
            target: target_identity,
            lineage,
            target_program,
            required_root,
            runtime: target.application().runtime.authority_identity(),
            actor,
            scope: scope.subject(),
            action_input: *input.identity(),
            action_binding: std::any::TypeId::of::<Binding>(),
            target_parent,
            target_branch: basis.identity().branch_id().clone(),
            intent,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn capture_coupled_outputs(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands>{
        self.coupled_capture
            .map(|capture| capture(&self.custody.occurrence(), execution))
    }

    pub(in crate::domain_computation::primary_graph) fn content(
        &self,
    ) -> &worth_relational::facade::content::RelationalContentAttachments {
        &self.content
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
        roles: impl Iterator<Item = (&'a str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &'a str, &'a worth_relational::facade::transactions::EntityReference)> + Clone,
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<(), super::AuthoredCorrectionCandidateDenial> {
        match &self.authored_identity {
            Some(identity) => identity.validate_candidate(roles, batch, request),
            None => Ok(()),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn authored_retirement_anchor(
        &self,
    ) -> Option<EntityId> {
        self.authored_identity
            .as_ref()
            .and_then(|identity| identity.retirement_anchor())
    }

    pub(in crate::domain_computation::primary_graph) fn authored_retirement_co_anchors(
        &self,
    ) -> Vec<EntityId> {
        self.authored_identity
            .as_ref()
            .map_or_else(Vec::new, |identity| identity.retirement_co_anchors())
    }
    pub(in crate::domain_computation::primary_graph) fn authored_restoration_co_anchors<'a>(
        &self,
        roles: impl Iterator<Item = (&'a str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &'a str, &'a worth_relational::facade::transactions::EntityReference)> + Clone,
    ) -> Vec<(
        EntityId,
        worth_relational::facade::transactions::CreatedEntityRef,
    )> {
        self.authored_identity
            .as_ref()
            .map_or_else(Vec::new, |identity| identity.restoration_co_anchors(roles))
    }

    pub(in crate::domain_computation::primary_graph) fn authored_restoration_anchor<'a>(
        &self,
        roles: impl Iterator<Item = (&'a str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &'a str, &'a worth_relational::facade::transactions::EntityReference)> + Clone,
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Option<(
            EntityId,
            worth_relational::facade::transactions::CreatedEntityRef,
        )>,
        super::AuthoredCorrectionCandidateDenial,
    > {
        self.validate_authored_candidate(roles.clone(), batch, request)?;
        Ok(self
            .authored_identity
            .as_ref()
            .and_then(|identity| identity.restoration_anchor(roles)))
    }

    pub(in crate::domain_computation::primary_graph) fn material_pin(
        &self,
    ) -> Option<worth_foundational::facade::CorrectionMaterialPinIdentity> {
        self.custody.pin()
    }
    pub(in crate::domain_computation::primary_graph) fn admit_home_receiver(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), worth_proof::HomeIncarnationDenial> {
        self.custody.admit_receiver(owner)
    }
    pub(in crate::domain_computation) fn admit_receiver(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial>
    {
        self.admit_home_receiver(owner).map_err(|cause| match cause {
            worth_proof::HomeIncarnationDenial::StaleIncarnation => crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::stale_incarnation(),
            worth_proof::HomeIncarnationDenial::ForeignHome => crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::correction_context_mismatch(),
        })
    }

    pub(crate) fn source_commit(&self) -> &RelationalCommitReceipt {
        self.custody.commit()
    }
    pub(crate) fn source_world(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.custody.world()
    }
    pub(crate) fn target_parent(&self) -> &RelationalCommitReceipt {
        &self.target_parent
    }
    pub(crate) fn target_branch(&self) -> &BranchId {
        &self.target_branch
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
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
                    == self.target)
    }
}

impl std::fmt::Debug for WorthQueryPreparedCorrectionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryPreparedCorrectionSource")
            .field("source", self.source_world())
            .field("target", &self.target)
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
            && self.target == other.target
    }
}
impl Eq for WorthQueryPreparedCorrectionSource {}
