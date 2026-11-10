use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis,
};
use crate::domain_installation::operation_identity_basis::{
    canonical_operation_identity, lineage_outcome_material,
};
use crate::domain_installation::{
    WorthQueryCompletedWorkflowTrace, WorthQueryConditionalOutcomeClass,
    WorthQueryOperationLineageContract, WorthQueryWorkflowStageReceipt,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::report::{lineage_evidence, WorthQueryStageLineageDeclaration};
use super::{WorthQueryTraceLineageCounters, WorthQueryTraceLineageReport};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLineageBindingDenial {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    StaleInstallationGeneration,
    LineageAlreadyBound,
    LineageNotDeclared,
    EmptyDeclarationSet,
    DuplicateStageDeclaration,
    UnknownStage,
    PreserveContractMismatch,
    IdentityEvolutionIsNotAuthoritative,
    ConditionalStageDidNotEstablishFreshLineage,
    DependencyClosureUnavailable,
    ClosureEffectMismatch,
}

impl WorthQueryLineageBindingDenial {
    pub(crate) fn identity_derivation_denial(self) -> Option<IdentityDerivationDenial> {
        match self {
            Self::EvidenceIdentityDerivationDenied(cause) => Some(cause),
            _ => None,
        }
    }
}

type TraceLineageBindingResult<D, O, F, L> = Result<
    WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    (
        WorthQueryCompletedWorkflowTrace<D, O, F, L>,
        WorthQueryLineageBindingDenial,
    ),
>;

pub(crate) fn bind_execution_lineage<D, O, F, L: BasisOperationLane>(
    trace: WorthQueryCompletedWorkflowTrace<D, O, F, L>,
) -> TraceLineageBindingResult<D, O, F, L> {
    let declarations = trace
        .stage_receipts()
        .iter()
        .filter(|receipt| !receipt.lineage.is_empty())
        .map(|receipt| {
            WorthQueryStageLineageDeclaration::from_execution(
                receipt.stage_identity().to_owned(),
                receipt.lineage.clone(),
            )
        })
        .collect::<Vec<_>>();
    if trace.bound().definition().semantics().lineage
        == WorthQueryOperationLineageContract::NotRequired
        && declarations.is_empty()
    {
        return Ok(trace);
    }
    bind_trace_lineage(trace, declarations)
}

fn bind_trace_lineage<D, O, F, L: BasisOperationLane>(
    mut trace: WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    declarations: Vec<WorthQueryStageLineageDeclaration>,
) -> TraceLineageBindingResult<D, O, F, L> {
    let contract = match admit_trace_lineage_binding(&trace, &declarations) {
        Ok(contract) => contract,
        Err(denial) => return Err((trace, denial)),
    };
    let mut binding = match TraceLineageBinding::new(&trace, contract) {
        Ok(binding) => binding,
        Err(denial) => return Err((trace, denial)),
    };
    for declaration in declarations {
        if let Err(denial) = binding.bind_declaration(&declaration) {
            return Err((trace, denial));
        }
    }
    let (evidence, counters) = binding.finish();
    let identity = match lineage_report_identity(trace.identity(), &evidence) {
        Ok(identity) => identity,
        Err(cause) => {
            return Err((
                trace,
                WorthQueryLineageBindingDenial::EvidenceIdentityDerivationDenied(cause),
            ))
        }
    };
    let proof = mint_operation_phase_proof(
        identity.clone(),
        Some(trace.phase_proof().payload().identity()),
        operation_phase_basis(trace.phase_proof()).clone(),
    );
    trace.lineage = Some(WorthQueryTraceLineageReport {
        identity,
        trace_identity: trace.identity().to_owned(),
        evidence,
        counters,
        proof,
    });
    trace.refresh_semantic_identity_for_lineage();
    Ok(trace)
}

fn admit_trace_lineage_binding<D, O, F, L: BasisOperationLane>(
    trace: &WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    declarations: &[WorthQueryStageLineageDeclaration],
) -> Result<WorthQueryOperationLineageContract, WorthQueryLineageBindingDenial> {
    if !trace.bound().installation_is_current() {
        return Err(WorthQueryLineageBindingDenial::StaleInstallationGeneration);
    }
    if trace.lineage_report().is_some() {
        return Err(WorthQueryLineageBindingDenial::LineageAlreadyBound);
    }
    let contract = trace.bound().definition().semantics().lineage;
    if matches!(contract, WorthQueryOperationLineageContract::NotRequired) {
        return Err(WorthQueryLineageBindingDenial::LineageNotDeclared);
    }
    if declarations.is_empty() {
        return Err(WorthQueryLineageBindingDenial::EmptyDeclarationSet);
    }
    Ok(contract)
}

struct TraceLineageBinding<'a> {
    contract: WorthQueryOperationLineageContract,
    dependency_closure:
        &'a crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    stage_index: std::collections::BTreeMap<&'a str, &'a WorthQueryWorkflowStageReceipt>,
    seen_stages: std::collections::BTreeSet<String>,
    counters: WorthQueryTraceLineageCounters,
    evidence: Vec<super::WorthQueryTraceLineageEvidence>,
}

impl<'a> TraceLineageBinding<'a> {
    fn new<D, O, F, L: BasisOperationLane>(
        trace: &'a WorthQueryCompletedWorkflowTrace<D, O, F, L>,
        contract: WorthQueryOperationLineageContract,
    ) -> Result<Self, WorthQueryLineageBindingDenial> {
        let dependency_closure = trace
            .semantic_aspect_dependency_closure()
            .ok_or(WorthQueryLineageBindingDenial::DependencyClosureUnavailable)?;
        let stage_index = trace
            .stage_receipts()
            .iter()
            .map(|receipt| (receipt.stage_identity(), receipt))
            .collect::<std::collections::BTreeMap<_, _>>();
        let counters = WorthQueryTraceLineageCounters {
            indexed_trace_stages: stage_index.len(),
            indexed_effect_receipts: stage_index
                .values()
                .map(|receipt| receipt.effect_evidence().len())
                .sum(),
            ..Default::default()
        };
        Ok(Self {
            contract,
            dependency_closure,
            stage_index,
            seen_stages: std::collections::BTreeSet::new(),
            counters,
            evidence: Vec::new(),
        })
    }

    fn bind_declaration(
        &mut self,
        declaration: &WorthQueryStageLineageDeclaration,
    ) -> Result<(), WorthQueryLineageBindingDenial> {
        if !self
            .seen_stages
            .insert(declaration.stage_identity().to_owned())
        {
            return Err(WorthQueryLineageBindingDenial::DuplicateStageDeclaration);
        }
        self.counters.stage_lookups += 1;
        let receipt = self
            .stage_index
            .get(declaration.stage_identity())
            .copied()
            .ok_or(WorthQueryLineageBindingDenial::UnknownStage)?;
        self.admit_conditional_lineage(receipt)?;
        for outcome in declaration.outcomes() {
            self.bind_outcome(receipt, outcome)?;
        }
        Ok(())
    }

    fn admit_conditional_lineage(
        &mut self,
        receipt: &WorthQueryWorkflowStageReceipt,
    ) -> Result<(), WorthQueryLineageBindingDenial> {
        if receipt.conditional_provenance().is_empty() {
            return Ok(());
        }
        self.counters.conditional_path_checks += 1;
        receipt
            .conditional_provenance()
            .iter()
            .any(|item| item.class() == WorthQueryConditionalOutcomeClass::ComputedChanged)
            .then_some(())
            .ok_or(WorthQueryLineageBindingDenial::ConditionalStageDidNotEstablishFreshLineage)
    }

    fn bind_outcome(
        &mut self,
        receipt: &WorthQueryWorkflowStageReceipt,
        outcome: &crate::identity_evolution::InstalledIdentityEvolutionOutcome,
    ) -> Result<(), WorthQueryLineageBindingDenial> {
        self.counters.outcome_contract_checks += 1;
        self.counters.outcome_width += outcome.width();
        if self.contract == WorthQueryOperationLineageContract::Preserve
            && outcome.kind()
                != crate::identity_evolution::InstalledIdentityEvolutionKind::PreservedIdentity
        {
            return Err(WorthQueryLineageBindingDenial::PreserveContractMismatch);
        }
        let foundational_lineage = outcome
            .foundational_attested_lineage()
            .ok_or(WorthQueryLineageBindingDenial::IdentityEvolutionIsNotAuthoritative)?;
        let effect_receipts = outcome
            .establishing_effect_receipt_identity()
            .map(|identity| vec![identity.to_owned()])
            .unwrap_or_default();
        if effect_receipts.iter().any(|identity| {
            !self
                .dependency_closure
                .contains_workflow_effect_receipt(identity)
        }) {
            return Err(WorthQueryLineageBindingDenial::ClosureEffectMismatch);
        }
        self.counters.effect_receipt_attachments += effect_receipts.len();
        self.evidence.push(lineage_evidence(
            receipt.stage_identity().to_owned(),
            receipt.identity().to_owned(),
            effect_receipts,
            outcome.clone(),
            foundational_lineage,
        ));
        Ok(())
    }

    fn finish(
        self,
    ) -> (
        Vec<super::WorthQueryTraceLineageEvidence>,
        WorthQueryTraceLineageCounters,
    ) {
        (self.evidence, self.counters)
    }
}

fn lineage_report_identity(
    trace_identity: &str,
    evidence: &[super::WorthQueryTraceLineageEvidence],
) -> Result<String, IdentityDerivationDenial> {
    canonical_operation_identity(
        "trace-lineage-report-v2",
        vec![
            ("lineage.trace", trace_identity.to_owned()),
            (
                "lineage.evidence",
                crate::domain_installation::operation_identity_basis::canonical_indexed_operation_material(
                    "lineage.evidence",
                    evidence.iter().map(|item| {
                        crate::domain_installation::operation_identity_basis::canonical_operation_material(vec![
                            ("lineage.stage", item.stage_identity().to_owned()),
                            ("lineage.stage_receipt", item.stage_receipt_identity().to_owned()),
                            (
                                "lineage.effect_receipts",
                                crate::domain_installation::operation_identity_basis::canonical_indexed_operation_material(
                                    "lineage.effect_receipt",
                                    item.effect_receipt_identities().iter().cloned(),
                                ),
                            ),
                            ("lineage.outcome", lineage_outcome_material(item.outcome())),
                        ])
                    }),
                ),
            ),
        ],
    )
}
