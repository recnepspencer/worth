use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::{
    QueryWorkflowDeclaration, WorkflowBasisFamily, WorkflowContextBinding, WorkflowFreshnessPolicy,
    WorkflowLoweringCounters,
};
use worth_relational::facade::merge::{MergeExecutionRequest, MergeIntent};

use super::counters::{
    lowering_denial_counters, merge_lowering_success_counters, LoweringDenialClass,
};
use super::errors::{
    ensure_merge_workflow_family, WorkflowLoweringError, WorkflowLoweringFailureClass,
};
use super::terms::{
    MergeAuthorityTarget, MergeLoweringInput, MergeWorkflowIntent, WorkflowFreshnessBinding,
    WorkflowStalenessClass,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweredMergeWorkflowDeclaration {
    declaration: QueryWorkflowDeclaration,
    merge_intent: MergeWorkflowIntent,
    authority_target: MergeAuthorityTarget,
    merge_request: MergeExecutionRequest,
    freshness_binding: WorkflowFreshnessBinding,
    staleness_class: WorkflowStalenessClass,
    lowering_identity: WorthQueryEvidenceIdentity,
    counters: WorkflowLoweringCounters,
}

impl LoweredMergeWorkflowDeclaration {
    pub fn declaration(&self) -> &QueryWorkflowDeclaration {
        &self.declaration
    }

    pub fn merge_intent(&self) -> &MergeWorkflowIntent {
        &self.merge_intent
    }

    pub fn authority_target(&self) -> &MergeAuthorityTarget {
        &self.authority_target
    }

    pub fn merge_request(&self) -> &MergeExecutionRequest {
        &self.merge_request
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

pub(crate) fn lower_merge_workflow_declaration(
    declaration: &QueryWorkflowDeclaration,
    input: MergeLoweringInput,
) -> Result<LoweredMergeWorkflowDeclaration, WorkflowLoweringError> {
    ensure_merge_workflow_family(declaration)?;
    ensure_distinct_merge_branches(&input)?;

    let freshness_binding = merge_freshness_binding(declaration.binding());
    ensure_merge_freshness_policy(declaration.request().freshness_policy(), &freshness_binding)?;
    let staleness_class = merge_staleness_class(&freshness_binding);
    let merge_request = MergeExecutionRequest::new(
        input.target_branch().clone(),
        input.source_branch().clone(),
        MergeIntent::ReconcileIntoTarget,
    );
    let lowering_identity =
        merge_lowering_identity(declaration, &input, &freshness_binding, &staleness_class)
            .map_err(|cause| {
                WorkflowLoweringError::identity_derivation_denied(
                    cause,
                    staleness_class.clone(),
                    lowering_denial_counters(1, LoweringDenialClass::MergeDenied),
                )
            })?;

    Ok(LoweredMergeWorkflowDeclaration {
        declaration: declaration.clone(),
        merge_intent: input.intent().clone(),
        authority_target: MergeAuthorityTarget::PairwiseExecution,
        merge_request,
        freshness_binding,
        staleness_class,
        lowering_identity,
        counters: merge_lowering_success_counters(1),
    })
}

fn ensure_distinct_merge_branches(input: &MergeLoweringInput) -> Result<(), WorkflowLoweringError> {
    if input.target_branch() != input.source_branch() {
        return Ok(());
    }
    Err(WorkflowLoweringError::new(
        WorkflowLoweringFailureClass::InvalidMergeBranchPairing,
        "merge lowering requires distinct target and source branches",
        WorkflowStalenessClass::ExactBasisPreserved,
        lowering_denial_counters(1, LoweringDenialClass::MergeDenied),
    ))
}

fn ensure_merge_freshness_policy(
    freshness_policy: &WorkflowFreshnessPolicy,
    freshness_binding: &WorkflowFreshnessBinding,
) -> Result<(), WorkflowLoweringError> {
    if matches!(
        freshness_binding,
        WorkflowFreshnessBinding::PreviewSessionBound
            | WorkflowFreshnessBinding::PreviewPromotionBound
    ) && freshness_policy == &WorkflowFreshnessPolicy::ExactBasis
    {
        return Err(WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::StaleWorkflowDenied,
            "preview-based merge lowering cannot preserve exact basis without authoritative freshness revalidation",
            WorkflowStalenessClass::StaleDenied,
            lowering_denial_counters(1, LoweringDenialClass::StaleDenied),
        ));
    }
    Ok(())
}

fn merge_freshness_binding(binding: &WorkflowContextBinding) -> WorkflowFreshnessBinding {
    match binding.basis_family() {
        WorkflowBasisFamily::RuntimePreflight => WorkflowFreshnessBinding::RuntimeBasisExact,
        WorkflowBasisFamily::PreviewFoundation => WorkflowFreshnessBinding::PreviewSessionBound,
        WorkflowBasisFamily::PreviewPromotionComparison => {
            WorkflowFreshnessBinding::PreviewPromotionBound
        }
        WorkflowBasisFamily::CorrespondenceHistorical => {
            WorkflowFreshnessBinding::BridgeAuthorityRebindRequired
        }
    }
}

fn merge_staleness_class(freshness_binding: &WorkflowFreshnessBinding) -> WorkflowStalenessClass {
    match freshness_binding {
        WorkflowFreshnessBinding::RuntimeBasisExact
        | WorkflowFreshnessBinding::PreviewSessionBound
        | WorkflowFreshnessBinding::PreviewPromotionBound => {
            WorkflowStalenessClass::AuthorityValidationRequired
        }
        WorkflowFreshnessBinding::BridgeAuthorityRebindRequired => {
            WorkflowStalenessClass::ExplicitRebindRequired
        }
    }
}

fn merge_lowering_identity(
    declaration: &QueryWorkflowDeclaration,
    input: &MergeLoweringInput,
    freshness_binding: &WorkflowFreshnessBinding,
    staleness_class: &WorkflowStalenessClass,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
        .field_shape(WorthQueryEvidenceTag::new("lowering_kind"), "merge")
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("declaration"),
            declaration.report().declaration_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("merge_intent"),
            input.intent().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_branch"),
            &input.target_branch().0,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_branch"),
            &input.source_branch().0,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("freshness"),
            freshness_binding.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("staleness"),
            staleness_class.as_str(),
        )
        .seal()
}
