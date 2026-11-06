use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::{
    QueryWorkflowDeclaration, WorkflowBasisFamily, WorkflowFreshnessPolicy,
    WorkflowLoweringCounters,
};
use worth_relational::facade::commit_strategies::{
    IntentReconciliationInput, NativeStrategyCommitRequest, StrategyCallerProvenance,
    StrategyRequestOrigin,
};
use worth_relational::facade::history::BranchId;

use super::counters::{
    lowering_denial_counters, mutation_lowering_success_counters, LoweringDenialClass,
};
use super::errors::{
    ensure_mutation_workflow_family, WorkflowLoweringError, WorkflowLoweringFailureClass,
};
use super::terms::{
    MutationIntentFamily, MutationLoweringInput, RelationalStrategyTarget,
    WorkflowFreshnessBinding, WorkflowStalenessClass,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweredMutationIntentDeclaration {
    declaration: QueryWorkflowDeclaration,
    mutation_family: MutationIntentFamily,
    strategy_target: RelationalStrategyTarget,
    authority_binding: MutationAuthorityBinding,
    strategy_request: NativeStrategyCommitRequest,
    freshness_binding: WorkflowFreshnessBinding,
    staleness_class: WorkflowStalenessClass,
    lowering_identity: WorthQueryEvidenceIdentity,
    counters: WorkflowLoweringCounters,
}

impl LoweredMutationIntentDeclaration {
    pub fn declaration(&self) -> &QueryWorkflowDeclaration {
        &self.declaration
    }

    pub fn mutation_family(&self) -> &MutationIntentFamily {
        &self.mutation_family
    }

    pub fn strategy_target(&self) -> &RelationalStrategyTarget {
        &self.strategy_target
    }

    pub fn authority_binding(&self) -> &MutationAuthorityBinding {
        &self.authority_binding
    }

    pub fn strategy_request(&self) -> &NativeStrategyCommitRequest {
        &self.strategy_request
    }

    pub fn freshness_binding(&self) -> &WorkflowFreshnessBinding {
        &self.freshness_binding
    }

    pub fn staleness_class(&self) -> &WorkflowStalenessClass {
        &self.staleness_class
    }

    pub fn lowering_for_reporting(&self) -> &str {
        self.lowering_identity.as_str()
    }

    pub fn lowering_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowering_identity
    }

    pub fn counters(&self) -> &WorkflowLoweringCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationAuthorityBinding {
    binding_identity: WorthQueryEvidenceIdentity,
    runtime_snapshot_identity: Option<WorthQuerySnapshotIdentity>,
    runtime_target_branch: Option<BranchId>,
}

impl MutationAuthorityBinding {
    fn new(
        binding_identity: WorthQueryEvidenceIdentity,
        runtime_snapshot_identity: Option<WorthQuerySnapshotIdentity>,
        runtime_target_branch: Option<BranchId>,
    ) -> Self {
        Self {
            binding_identity,
            runtime_snapshot_identity,
            runtime_target_branch,
        }
    }

    pub fn binding_digest(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn runtime_snapshot_identity(&self) -> Option<&WorthQuerySnapshotIdentity> {
        self.runtime_snapshot_identity.as_ref()
    }

    pub fn runtime_target_branch(&self) -> Option<&BranchId> {
        self.runtime_target_branch.as_ref()
    }
}

pub(crate) fn lower_mutation_intent_declaration(
    declaration: &QueryWorkflowDeclaration,
    authority_binding_identity: &WorthQueryEvidenceIdentity,
    input: MutationLoweringInput,
) -> Result<LoweredMutationIntentDeclaration, WorkflowLoweringError> {
    ensure_mutation_workflow_family(declaration)?;
    let freshness_binding = mutation_freshness_binding(
        declaration.binding(),
        declaration.request().freshness_policy(),
    )?;
    let mutation_family = input.family();
    let strategy_target = match mutation_family {
        MutationIntentFamily::IntentReconciliation => {
            RelationalStrategyTarget::IntentReconciliation
        }
    };
    let request = intent_reconciliation_strategy_request(declaration, input)?;
    let lowering_identity = mutation_lowering_identity(
        declaration,
        &mutation_family,
        &strategy_target,
        authority_binding_identity,
        &freshness_binding,
        &request,
    );
    Ok(LoweredMutationIntentDeclaration {
        declaration: declaration.clone(),
        mutation_family,
        strategy_target,
        authority_binding: MutationAuthorityBinding::new(
            authority_binding_identity.clone(),
            declaration.binding().runtime_snapshot_identity().cloned(),
            declaration.binding().runtime_target_branch().cloned(),
        ),
        strategy_request: request,
        freshness_binding,
        staleness_class: WorkflowStalenessClass::ExactBasisPreserved,
        lowering_identity,
        counters: mutation_lowering_success_counters(1),
    })
}

fn intent_reconciliation_strategy_request(
    declaration: &QueryWorkflowDeclaration,
    input: MutationLoweringInput,
) -> Result<NativeStrategyCommitRequest, WorkflowLoweringError> {
    let MutationLoweringInput::IntentReconciliation {
        entity_id,
        desired_aspect_fields,
    } = input;
    IntentReconciliationInput {
        entity_id,
        desired_aspect_fields,
    }
    .into_native_canonical_request(StrategyCallerProvenance {
        request_origin: StrategyRequestOrigin::Api,
        actor_identity: Some("worth-query".to_string()),
        correlation_id: Some(declaration.report().declaration_digest().to_string()),
    })
    .map_err(|_| {
        WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::LoweringSerializationFailed,
            "mutation lowering could not encode native intent reconciliation input",
            WorkflowStalenessClass::ExactBasisPreserved,
            lowering_denial_counters(1, LoweringDenialClass::General),
        )
    })
}

fn mutation_freshness_binding(
    binding: &crate::workflow::WorkflowContextBinding,
    freshness_policy: &WorkflowFreshnessPolicy,
) -> Result<WorkflowFreshnessBinding, WorkflowLoweringError> {
    match binding.basis_family() {
        WorkflowBasisFamily::RuntimePreflight => Ok(WorkflowFreshnessBinding::RuntimeBasisExact),
        WorkflowBasisFamily::PreviewFoundation
        | WorkflowBasisFamily::PreviewPromotionComparison => {
            let (failure_class, message, staleness_class, denial_class) =
                mutation_preview_freshness_denial(freshness_policy);
            Err(WorkflowLoweringError::new(
                failure_class,
                message,
                staleness_class,
                lowering_denial_counters(1, denial_class),
            ))
        }
        WorkflowBasisFamily::CorrespondenceHistorical => Err(WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::UnsupportedRelationalStrategyTarget,
            "correspondence/historical contexts cannot lower query-owned mutation intent",
            WorkflowStalenessClass::ExplicitRebindRequired,
            lowering_denial_counters(1, LoweringDenialClass::AmbientBasisFallback),
        )),
    }
}

fn mutation_preview_freshness_denial(
    freshness_policy: &WorkflowFreshnessPolicy,
) -> (
    WorkflowLoweringFailureClass,
    &'static str,
    WorkflowStalenessClass,
    LoweringDenialClass,
) {
    if freshness_policy == &WorkflowFreshnessPolicy::ExactBasis {
        (
            WorkflowLoweringFailureClass::StaleWorkflowDenied,
            "mutation lowering cannot preserve exact basis from preview workflow contexts without authoritative revalidation",
            WorkflowStalenessClass::StaleDenied,
            LoweringDenialClass::StaleDenied,
        )
    } else {
        (
            WorkflowLoweringFailureClass::ExplicitRebindRequired,
            "mutation lowering remains runtime-basis only until relational authority rebind is explicit",
            WorkflowStalenessClass::ExplicitRebindRequired,
            LoweringDenialClass::ExplicitRebind,
        )
    }
}

fn mutation_lowering_identity(
    declaration: &QueryWorkflowDeclaration,
    mutation_family: &MutationIntentFamily,
    strategy_target: &RelationalStrategyTarget,
    authority_binding_identity: &WorthQueryEvidenceIdentity,
    freshness_binding: &WorkflowFreshnessBinding,
    request: &NativeStrategyCommitRequest,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                declaration.report().declaration_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("mutation_family"),
                mutation_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("strategy_target"),
                strategy_target.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authority_binding"),
                authority_binding_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("freshness"),
                freshness_binding.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("request_origin"),
                request_origin_label(&request.caller_provenance().request_origin),
            );
    if let Some(runtime_snapshot_identity) = declaration.binding().runtime_snapshot_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime_snapshot"),
            &runtime_snapshot_identity.evidence_identity()?,
        );
    }
    Ok(identity.seal()?)
}

fn request_origin_label(origin: &StrategyRequestOrigin) -> &'static str {
    match origin {
        StrategyRequestOrigin::Api => "api",
        StrategyRequestOrigin::Harness => "harness",
        StrategyRequestOrigin::Replay => "replay",
        StrategyRequestOrigin::Test => "test",
    }
}
