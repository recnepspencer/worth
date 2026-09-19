use crate::runtime::replacement::candidate::{
    file_authored_replacement_candidate, rust_authored_replacement_candidate,
};
use crate::runtime::source_ingress::{
    prepare_semantic_handoff, WorthUiCandidateComposition, WorthUiSourceIngressDenial,
    WorthUiSourceIngressDenialReason,
};
use crate::runtime::WorthUiReplacementCause;

use super::{
    UiSourceCompilationDenialReceipt, UiSourceRebindAttemptBasis, UiSourceRebindAttemptDenial,
    UiSourceRebindAttemptDenialReceipt, UiSourceRebindAttemptOutcome,
};

pub(super) fn attempt(
    settled: super::super::WorthUiSettledSourceSnapshot,
    snapshot: &crate::capability::CapabilitySnapshot,
) -> UiSourceRebindAttemptOutcome {
    let (provider, revision, ordering, counters) = settled.into_parts();
    let retained_observation_bytes = retained_source_bytes(&provider);
    let basis = UiSourceRebindAttemptBasis::seal(revision, ordering, counters, snapshot.digest());
    if !basis
        .ordering_receipt()
        .matches_revision(basis.source_revision())
    {
        return denied_source(
            WorthUiSourceIngressDenialReason::OrderingReceiptDrift,
            basis,
        );
    }
    if let Some(reason) = ambiguous_material(&provider) {
        return denied_source(reason, basis);
    }
    let composition = if let Some(input) = provider.rust_authored_inputs().first() {
        compile_rust(input, snapshot, &basis)
    } else if !provider.source_modules().is_empty() {
        compile_file(&provider, snapshot, &basis)
    } else {
        return denied_source(WorthUiSourceIngressDenialReason::NoCandidateMaterial, basis);
    };
    match composition {
        Ok(composition) => {
            let (revision, ordering, mut counters) = basis.into_source_parts();
            counters.emit_candidate_submission();
            UiSourceRebindAttemptOutcome::Candidate(Box::new(
                super::super::WorthUiWatchedCandidateSubmission::from_source_attempt(
                    composition,
                    revision,
                    ordering,
                    counters,
                    retained_observation_bytes,
                ),
            ))
        }
        Err(outcome) => outcome,
    }
}

fn compile_file(
    provider: &super::super::WorthUiSourceProvider,
    snapshot: &crate::capability::CapabilitySnapshot,
    basis: &UiSourceRebindAttemptBasis,
) -> Result<WorthUiCandidateComposition, UiSourceRebindAttemptOutcome> {
    let mut input = worth_ui_dsl::WorthUiAuthoredSourceInput::rooted_at(provider.workspace_root());
    for module in provider.source_modules() {
        input = input.with_module(module.relative_path(), module.source_text());
    }
    let sealed = worth_ui_dsl::WorthUiDslCompiler::compile_source(input).map_err(|report| {
        UiSourceRebindAttemptOutcome::CompilationDenied(Box::new(
            UiSourceCompilationDenialReceipt::new(basis.clone(), report),
        ))
    })?;
    let primary = sealed.module_ids().first().cloned().ok_or_else(|| {
        denied_source(
            WorthUiSourceIngressDenialReason::NoCandidateMaterial,
            basis.clone(),
        )
    })?;
    let material = prepare_semantic_handoff(sealed, snapshot).map_err(|denial| {
        denied(
            UiSourceRebindAttemptDenial::RuntimePreparation(denial),
            basis.clone(),
        )
    })?;
    let (artifact, declaration, handoff) = material.into_parts();
    let candidate = file_authored_replacement_candidate(
        artifact,
        handoff.successor_snapshot_digest(),
        WorthUiReplacementCause::file_source_change(
            primary,
            basis.source_revision().final_package_digest(),
        ),
    )
    .map_err(|denial| {
        denied(
            UiSourceRebindAttemptDenial::Candidate(denial),
            basis.clone(),
        )
    })?;
    Ok(WorthUiCandidateComposition::file_authored(
        candidate,
        declaration,
        handoff,
    ))
}

fn compile_rust(
    input: &worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
    snapshot: &crate::capability::CapabilitySnapshot,
    basis: &UiSourceRebindAttemptBasis,
) -> Result<WorthUiCandidateComposition, UiSourceRebindAttemptOutcome> {
    let sealed =
        worth_ui_dsl::WorthUiDslCompiler::compile_rust_authored(input).map_err(|report| {
            UiSourceRebindAttemptOutcome::CompilationDenied(Box::new(
                UiSourceCompilationDenialReceipt::new(basis.clone(), report),
            ))
        })?;
    let material = prepare_semantic_handoff(sealed, snapshot).map_err(|denial| {
        denied(
            UiSourceRebindAttemptDenial::RuntimePreparation(denial),
            basis.clone(),
        )
    })?;
    let (artifact, declaration, handoff) = material.into_parts();
    let candidate = rust_authored_replacement_candidate(
        artifact,
        handoff.successor_snapshot_digest(),
        WorthUiReplacementCause::rust_authored_input_change(
            basis.source_revision().final_package_digest(),
        ),
    )
    .map_err(|denial| {
        denied(
            UiSourceRebindAttemptDenial::Candidate(denial),
            basis.clone(),
        )
    })?;
    Ok(WorthUiCandidateComposition::rust_authored(
        candidate,
        declaration,
        handoff,
    ))
}

fn ambiguous_material(
    provider: &super::super::WorthUiSourceProvider,
) -> Option<WorthUiSourceIngressDenialReason> {
    if !provider.source_modules().is_empty() && !provider.rust_authored_inputs().is_empty() {
        Some(WorthUiSourceIngressDenialReason::MixedCandidateMaterial)
    } else if provider.rust_authored_inputs().len() > 1 {
        Some(WorthUiSourceIngressDenialReason::MultipleRustAuthoredInputs)
    } else {
        None
    }
}

fn denied_source(
    reason: WorthUiSourceIngressDenialReason,
    basis: UiSourceRebindAttemptBasis,
) -> UiSourceRebindAttemptOutcome {
    denied(
        UiSourceRebindAttemptDenial::SourceIngress(WorthUiSourceIngressDenial::new(reason)),
        basis,
    )
}

fn denied(
    denial: UiSourceRebindAttemptDenial,
    basis: UiSourceRebindAttemptBasis,
) -> UiSourceRebindAttemptOutcome {
    UiSourceRebindAttemptOutcome::Denied(Box::new(UiSourceRebindAttemptDenialReceipt::new(
        basis, denial,
    )))
}

fn retained_source_bytes(provider: &super::super::WorthUiSourceProvider) -> usize {
    let module_bytes = provider
        .source_modules()
        .iter()
        .map(|module| module.source_text().len())
        .sum::<usize>();
    module_bytes
        .saturating_add(std::mem::size_of_val(provider))
        .saturating_add(
            provider
                .rust_authored_inputs()
                .iter()
                .map(std::mem::size_of_val)
                .sum::<usize>(),
        )
}
