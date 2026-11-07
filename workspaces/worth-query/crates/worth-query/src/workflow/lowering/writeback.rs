use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::identity::bridge_request_kind_label;
use crate::workflow::{
    QueryWorkflowDeclaration, WorkflowBasisFamily, WorkflowContextBinding, WorkflowFreshnessPolicy,
    WorkflowLoweringCounters,
};
use worth_runtime_bridge::facade::{
    BridgeRequestKind, BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity,
    BridgeWritebackEffectClass, BridgeWritebackFamilyKind, BridgeWritebackIdempotenceClass,
    BridgeWritebackStrategyClass,
};

use super::counters::{
    lowering_denial_counters, writeback_lowering_success_counters, LoweringDenialClass,
};
use super::errors::{
    ensure_writeback_workflow_family, WorkflowLoweringError, WorkflowLoweringFailureClass,
};
use super::terms::{
    WorkflowFreshnessBinding, WorkflowStalenessClass, WritebackDeclarationFamily,
    WritebackLoweringInput,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WritebackCausalityBinding {
    binding_identity: WorthQueryEvidenceIdentity,
    basis_family: WorkflowBasisFamily,
    request_kind: BridgeRequestKind,
    causality_identity: WorthQueryEvidenceIdentity,
    basis_identity: WorthQueryEvidenceIdentity,
}

impl WritebackCausalityBinding {
    pub fn binding_for_reporting(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub fn basis_family(&self) -> &WorkflowBasisFamily {
        &self.basis_family
    }

    pub fn basis_for_reporting(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn request_kind(&self) -> BridgeRequestKind {
        self.request_kind
    }

    pub fn causality_for_reporting(&self) -> &str {
        self.causality_identity.as_str()
    }

    pub fn causality_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.causality_identity
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryWritebackDeclaration {
    declaration: QueryWorkflowDeclaration,
    family: WritebackDeclarationFamily,
    causality_binding: WritebackCausalityBinding,
    bridge_declaration: BridgeWritebackDeclaration,
    freshness_binding: WorkflowFreshnessBinding,
    staleness_class: WorkflowStalenessClass,
    lowering_identity: WorthQueryEvidenceIdentity,
    counters: WorkflowLoweringCounters,
}

impl QueryWritebackDeclaration {
    pub fn declaration(&self) -> &QueryWorkflowDeclaration {
        &self.declaration
    }

    pub fn family(&self) -> &WritebackDeclarationFamily {
        &self.family
    }

    pub fn causality_binding(&self) -> &WritebackCausalityBinding {
        &self.causality_binding
    }

    pub fn bridge_declaration(&self) -> &BridgeWritebackDeclaration {
        &self.bridge_declaration
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

pub(crate) fn lower_query_writeback_declaration(
    declaration: &QueryWorkflowDeclaration,
    input: WritebackLoweringInput,
) -> Result<QueryWritebackDeclaration, WorkflowLoweringError> {
    ensure_writeback_workflow_family(declaration)?;
    let request_kind = writeback_request_kind(
        declaration.binding(),
        declaration.request().freshness_policy(),
    )?;
    let binding_identity = declaration.binding().binding_identity().clone();
    let identity_denial = |cause| {
        WorkflowLoweringError::identity_derivation_denied(
            cause,
            WorkflowStalenessClass::AuthorityValidationRequired,
            lowering_denial_counters(1, LoweringDenialClass::WritebackDenied),
        )
    };
    let causality_identity =
        writeback_causality_identity(declaration, request_kind).map_err(identity_denial)?;
    let basis_identity = writeback_basis_identity(declaration).map_err(identity_denial)?;
    let bridge_declaration =
        writeback_bridge_declaration(declaration, input.family(), request_kind)
            .map_err(identity_denial)?;
    let lowering_identity = writeback_lowering_identity(
        declaration,
        input.family(),
        &bridge_declaration,
        &causality_identity,
    )
    .map_err(identity_denial)?;

    Ok(QueryWritebackDeclaration {
        declaration: declaration.clone(),
        family: input.family().clone(),
        causality_binding: WritebackCausalityBinding {
            binding_identity,
            basis_family: declaration.binding().basis_family().clone(),
            request_kind,
            causality_identity,
            basis_identity,
        },
        bridge_declaration,
        freshness_binding: WorkflowFreshnessBinding::RuntimeBasisExact,
        staleness_class: WorkflowStalenessClass::AuthorityValidationRequired,
        lowering_identity,
        counters: writeback_lowering_success_counters(1),
    })
}

fn writeback_request_kind(
    binding: &WorkflowContextBinding,
    freshness_policy: &WorkflowFreshnessPolicy,
) -> Result<BridgeRequestKind, WorkflowLoweringError> {
    match binding.basis_family() {
        WorkflowBasisFamily::RuntimePreflight => Ok(BridgeRequestKind::Authoritative),
        WorkflowBasisFamily::PreviewFoundation
        | WorkflowBasisFamily::PreviewPromotionComparison => {
            let (failure_class, message, staleness_class, denial_class) =
                writeback_preview_freshness_denial(freshness_policy);
            Err(WorkflowLoweringError::new(
                failure_class,
                message,
                staleness_class,
                lowering_denial_counters(1, denial_class),
            ))
        }
        WorkflowBasisFamily::CorrespondenceHistorical => Err(WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::UnsupportedWritebackCausality,
            "correspondence/historical workflow contexts cannot mint bridge writeback causality",
            WorkflowStalenessClass::ExplicitRebindRequired,
            lowering_denial_counters(1, LoweringDenialClass::AmbientBasisFallback),
        )),
    }
}

fn writeback_preview_freshness_denial(
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
            "query-triggered writeback cannot preserve exact basis from preview workflow contexts without authoritative revalidation",
            WorkflowStalenessClass::StaleDenied,
            LoweringDenialClass::StaleDenied,
        )
    } else {
        (
            WorkflowLoweringFailureClass::ExplicitRebindRequired,
            "query-triggered writeback requires authoritative rebind before bridge lowering",
            WorkflowStalenessClass::ExplicitRebindRequired,
            LoweringDenialClass::WritebackExplicitRebind,
        )
    }
}

fn writeback_bridge_declaration(
    declaration: &QueryWorkflowDeclaration,
    family: &WritebackDeclarationFamily,
    request_kind: BridgeRequestKind,
) -> BridgeWritebackDeclaration {
    let (family_kind, effect_class, strategy_class, idempotence_class) =
        writeback_bridge_family_terms(family);
    let bridge_declaration_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_writeback_bridge_declaration_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                declaration.report().declaration_identity(),
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens");
    BridgeWritebackDeclaration::writeback_capable(
        BridgeWritebackDeclarationIdentity::from_bridge_evidence(
            &bridge_declaration_identity.bridge_external_identity_evidence(),
        ),
        request_kind,
        family_kind,
        effect_class,
        strategy_class,
        idempotence_class,
    )
}

fn writeback_bridge_family_terms(
    family: &WritebackDeclarationFamily,
) -> (
    BridgeWritebackFamilyKind,
    BridgeWritebackEffectClass,
    BridgeWritebackStrategyClass,
    BridgeWritebackIdempotenceClass,
) {
    match family {
        WritebackDeclarationFamily::ProjectedStateDiff => (
            BridgeWritebackFamilyKind::ProjectedStateDiff,
            BridgeWritebackEffectClass::ProjectedStateDiff,
            BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
            BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
        ),
    }
}

fn writeback_causality_identity(
    declaration: &QueryWorkflowDeclaration,
    request_kind: BridgeRequestKind,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), "writeback_causality")
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("binding"),
        declaration.binding().binding_identity(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("basis_family"),
        declaration.binding().basis_family().as_str(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("basis"),
        declaration.binding().basis_identity(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("request_kind"),
        bridge_request_kind_label(request_kind),
    )
    .seal()?)
}

fn writeback_basis_identity(declaration: &QueryWorkflowDeclaration) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), "writeback_basis")
    .field_shape(
        WorthQueryEvidenceTag::new("basis_family"),
        declaration.binding().basis_family().as_str(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("basis"),
        declaration.binding().basis_identity(),
    )
    .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

fn writeback_lowering_identity(
    declaration: &QueryWorkflowDeclaration,
    family: &WritebackDeclarationFamily,
    bridge_declaration: &BridgeWritebackDeclaration,
    causality_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), "writeback_lowering")
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("declaration"),
        declaration.report().declaration_identity(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("writeback_family"),
        family.as_str(),
    )
    .field_bridge_authority_identity(
        WorthQueryEvidenceTag::new("bridge_declaration"),
        &bridge_declaration
            .declaration_identity()
            .bridge_trust_boundary(),
    )
    .field_evidence_identity(WorthQueryEvidenceTag::new("causality"), causality_identity)
    .seal()?)
}
