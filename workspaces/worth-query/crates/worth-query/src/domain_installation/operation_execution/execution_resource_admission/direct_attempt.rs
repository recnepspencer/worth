use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryOperationPhaseProof,
    WorthQueryResourceAdmittedOperationPhase,
};
use worth_proof::TransitionOutcome;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;

use super::{
    admit_execution_resource_plan, WorthQueryAdmittedExecutionResourcePlan,
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionResourceAdmissionCounters,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenialKind as Kind,
};

pub type WorthQueryDirectResourceAdmissionOutcome<D, O, F, L> = TransitionOutcome<
    WorthQueryAdmittedDirectOperation<D, O, F, L>,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
>;

pub struct WorthQueryAdmittedDirectOperation<D, O, F, L: BasisOperationLane>
where
    O: crate::domain_installation::WorthQueryExecutableDomainOperation<D, F>,
{
    pub(crate) bound: crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    pub(crate) input: O::Input,
    pub(crate) executor:
        std::sync::Arc<crate::domain_installation::WorthQueryInstalledDomainOperationExecutor>,
    pub(crate) resource_attempt: WorthQueryDirectExecutionResourceAttempt,
    pub(crate) phase_proof: WorthQueryOperationPhaseProof<WorthQueryResourceAdmittedOperationPhase>,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryAdmittedDirectOperation<D, O, F, L>
where
    O: crate::domain_installation::WorthQueryExecutableDomainOperation<D, F>,
{
    pub fn resources(&self) -> &WorthQueryAdmittedExecutionResourcePlan {
        self.resource_attempt.resources()
    }

    pub fn provider_session_identity(&self) -> &str {
        self.resource_attempt.evidence().provider_session_identity()
    }

    pub fn provider_session_attempt_identity(&self) -> &str {
        self.resource_attempt
            .evidence()
            .provider_session_attempt_identity()
    }
}

impl<D: 'static, O, F: 'static, L: BasisOperationLane>
    crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>
where
    O: crate::domain_installation::WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = crate::domain_installation::WorthQueryDirectOperation,
    >,
{
    pub fn admit_execution_resources(
        self,
        input: O::Input,
        request: WorthQueryExecutionResourceRequest,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> WorthQueryDirectResourceAdmissionOutcome<D, O, F, L> {
        let mut counters = WorthQueryExecutionResourceAdmissionCounters {
            runtime_authority_checks: 1,
            ..Default::default()
        };
        let witness =
            crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
                std::sync::Arc::clone(self.operation().domain_authority()),
            );
        if let Err(denial) = workspace.validate_installed_domain_witness::<D>(&witness) {
            let kind = denial.kind();
            let denial = WorthQueryExecutionResourceAdmissionDenial::new(
                Kind::RuntimeAuthority(kind),
                format!("{denial:?}"),
                counters,
            );
            return runtime_denial(kind, denial);
        }
        counters.input_contract_checks += 1;
        if !super::super::operation_input::input_satisfies_contract(
            &input,
            &self.definition().semantics().parameters,
        ) {
            return TransitionOutcome::Denied(WorthQueryExecutionResourceAdmissionDenial::new(
                Kind::InputContract,
                "operation input does not satisfy the installed parameter contract",
                counters,
            ));
        }
        counters.execution_contract_checks += 1;
        let semantics = self.definition().semantics();
        if !direct_graph_evidence_can_realize(semantics)
            || !matches!(
                semantics.invariants,
                crate::domain_installation::WorthQueryOperationInvariantContract::NotRequired
            )
            || !matches!(
                semantics.lineage,
                crate::domain_installation::WorthQueryOperationLineageContract::NotRequired
            )
        {
            return TransitionOutcome::Denied(WorthQueryExecutionResourceAdmissionDenial::new(
                Kind::DirectExecutionContract,
                "direct execution lacks an admitted evidence route for the declared effects, invariants, or lineage",
                counters,
            ));
        }
        let executor = std::sync::Arc::clone(self.direct_executor());
        let support = self
            .execution_authority()
            .direct_support()
            .expect("bound direct execution authority carries installed support")
            .clone();
        let resources = match admit_execution_resource_plan(
            self.binding_identity(),
            &self.definition().semantics().resources,
            &request,
            support,
            counters,
        ) {
            Ok(resources) => resources,
            Err(denial) => return admission_denial_outcome(denial),
        };
        let resource_attempt = match workspace
            .query_execution_runtime()
            .start_direct_resource_attempt(self.execution_authority(), resources)
        {
            Ok(attempt) => attempt,
            Err(denial) => return admission_denial_outcome(denial),
        };
        let mut basis = operation_phase_basis(self.authority_proof()).clone();
        basis.resource_admission_identity =
            Some(resource_attempt.resources().identity().to_owned());
        let phase_proof = mint_operation_phase_proof(
            resource_attempt.resources().identity().to_owned(),
            Some(self.authority_proof().payload().identity()),
            basis,
        );
        TransitionOutcome::Success(WorthQueryAdmittedDirectOperation {
            bound: self,
            input,
            executor,
            resource_attempt,
            phase_proof,
        })
    }
}

fn direct_graph_evidence_can_realize(
    semantics: &worth_query_installation::facade::WorthQueryDomainOperationSemanticClosure,
) -> bool {
    use crate::domain_installation::{
        WorthQueryOperationEffectContract as Effects,
        WorthQueryOperationEffectFamily as EffectFamily,
        WorthQueryOperationTouchContract as Touches,
    };

    match (&semantics.touches, &semantics.effects) {
        (Touches::NotRequired, Effects::NotRequired) => true,
        (Touches::Declared { graph_roles, .. }, Effects::Declared { effect_families }) => {
            !graph_roles.is_empty()
                && !effect_families.is_empty()
                && effect_families
                    .iter()
                    .all(|family| *family == EffectFamily::Mutation)
        }
        (Touches::Declared { graph_roles, .. }, Effects::NotRequired) => !graph_roles.is_empty(),
        (Touches::NotRequired, Effects::Declared { .. }) => false,
    }
}

fn runtime_denial<T>(
    kind: crate::domain_installation::WorthQueryDomainHandleDenialKind,
    denial: WorthQueryExecutionResourceAdmissionDenial,
) -> TransitionOutcome<
    T,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
> {
    use crate::domain_installation::WorthQueryDomainHandleDenialKind as Runtime;
    match kind {
        Runtime::StaleInstallationGeneration => TransitionOutcome::Stale(denial),
        Runtime::PackageIdentityChanged => TransitionOutcome::RebindRequired(denial),
        Runtime::DomainNotInstalled | Runtime::ForeignRuntime => TransitionOutcome::Denied(denial),
    }
}

fn admission_denial_outcome<T>(
    denial: WorthQueryExecutionResourceAdmissionDenial,
) -> TransitionOutcome<
    T,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
> {
    match denial.kind() {
        Kind::Backpressured | Kind::AsyncExecutionRequired => TransitionOutcome::Deferred(denial),
        _ => TransitionOutcome::Denied(denial),
    }
}
