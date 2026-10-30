use std::collections::{BTreeMap, BTreeSet};

use crate::capability::{
    CapabilityDiagnosticCode, CapabilityDiagnosticRichness, CapabilityRegistrationDiagnostic,
    CapabilitySupportKind, RegisteredCapabilitySet, THEME_TOKEN_FAMILY_NAME,
};

use super::registration_candidate::RegistrationCandidate;
use super::registration_validation_report::{
    AcceptedRegistrationKey, RegistrationValidationReport,
};

type CandidateKey = (&'static str, String);

pub(crate) fn validate_registration_candidates(
    candidates: &[RegistrationCandidate],
    richness: CapabilityDiagnosticRichness,
) -> RegistrationValidationReport {
    let duplicate_keys = duplicate_candidate_keys(candidates);
    let cycle_keys = theme_token_alias_cycle_keys(candidates, &duplicate_keys);
    let mut diagnostics =
        collect_registration_diagnostics(candidates, &duplicate_keys, &cycle_keys, richness);
    diagnostics.sort_by_key(CapabilityRegistrationDiagnostic::ordering_key);
    let accepted_registration_keys =
        accepted_candidate_keys(candidates, &duplicate_keys, &cycle_keys);

    RegistrationValidationReport::new(
        accepted_capabilities_from_keys(&accepted_registration_keys),
        accepted_registration_keys,
        diagnostics,
    )
}

fn duplicate_candidate_keys(candidates: &[RegistrationCandidate]) -> BTreeSet<CandidateKey> {
    let mut counts = BTreeMap::<CandidateKey, usize>::new();
    for candidate in candidates {
        *counts.entry(candidate_key(candidate)).or_default() += 1;
    }

    counts
        .into_iter()
        .filter_map(|(candidate_key, count)| (count > 1).then_some(candidate_key))
        .collect()
}

fn collect_registration_diagnostics(
    candidates: &[RegistrationCandidate],
    duplicate_keys: &BTreeSet<CandidateKey>,
    cycle_keys: &BTreeSet<CandidateKey>,
    richness: CapabilityDiagnosticRichness,
) -> Vec<CapabilityRegistrationDiagnostic> {
    let resolvable_keys = resolvable_dependency_keys(candidates, duplicate_keys);
    let mut diagnostics = Vec::new();

    for candidate in candidates {
        collect_duplicate_id_diagnostic(candidate, duplicate_keys, richness, &mut diagnostics);
        collect_support_posture_diagnostic(candidate, richness, &mut diagnostics);
        collect_descriptor_diagnostics(candidate, richness, &mut diagnostics);
        collect_dependency_diagnostics(candidate, &resolvable_keys, richness, &mut diagnostics);
        collect_theme_token_alias_cycle_diagnostic(
            candidate,
            cycle_keys,
            richness,
            &mut diagnostics,
        );
    }

    diagnostics
}

fn collect_theme_token_alias_cycle_diagnostic(
    candidate: &RegistrationCandidate,
    cycle_keys: &BTreeSet<CandidateKey>,
    richness: CapabilityDiagnosticRichness,
    diagnostics: &mut Vec<CapabilityRegistrationDiagnostic>,
) {
    if cycle_keys.contains(&candidate_key(candidate)) {
        diagnostics.push(diagnostic_for_candidate(
            CapabilityDiagnosticCode::ThemeTokenAliasCycle,
            candidate,
            None,
            None,
            rich_detail(richness, "theme token aliases must form an acyclic graph"),
        ));
    }
}

fn collect_duplicate_id_diagnostic(
    candidate: &RegistrationCandidate,
    duplicate_keys: &BTreeSet<CandidateKey>,
    richness: CapabilityDiagnosticRichness,
    diagnostics: &mut Vec<CapabilityRegistrationDiagnostic>,
) {
    if duplicate_keys.contains(&candidate_key(candidate)) {
        diagnostics.push(diagnostic_for_candidate(
            CapabilityDiagnosticCode::DuplicateCapabilityId,
            candidate,
            None,
            None,
            rich_detail(richness, "duplicate capability identity"),
        ));
    }
}

fn collect_support_posture_diagnostic(
    candidate: &RegistrationCandidate,
    richness: CapabilityDiagnosticRichness,
    diagnostics: &mut Vec<CapabilityRegistrationDiagnostic>,
) {
    if candidate.support_kind() == CapabilitySupportKind::Unsupported {
        diagnostics.push(diagnostic_for_candidate(
            CapabilityDiagnosticCode::UnsupportedPostureReference,
            candidate,
            None,
            None,
            rich_detail(
                richness,
                "unsupported posture cannot become accepted registration",
            ),
        ));
    }
}

fn collect_descriptor_diagnostics(
    candidate: &RegistrationCandidate,
    richness: CapabilityDiagnosticRichness,
    diagnostics: &mut Vec<CapabilityRegistrationDiagnostic>,
) {
    for descriptor_diagnostic in candidate.descriptor_diagnostics() {
        diagnostics.push(diagnostic_for_candidate(
            descriptor_diagnostic.code(),
            candidate,
            None,
            None,
            rich_detail(richness, descriptor_diagnostic.detail()),
        ));
    }
}

fn collect_dependency_diagnostics(
    candidate: &RegistrationCandidate,
    resolvable_keys: &BTreeSet<CandidateKey>,
    richness: CapabilityDiagnosticRichness,
    diagnostics: &mut Vec<CapabilityRegistrationDiagnostic>,
) {
    for dependency in candidate.dependencies() {
        collect_dependency_diagnostic(
            candidate,
            dependency,
            resolvable_keys,
            richness,
            diagnostics,
        );
    }
}

fn collect_dependency_diagnostic(
    candidate: &RegistrationCandidate,
    dependency: &super::registration_candidate::RegistrationDependency,
    resolvable_keys: &BTreeSet<CandidateKey>,
    richness: CapabilityDiagnosticRichness,
    diagnostics: &mut Vec<CapabilityRegistrationDiagnostic>,
) {
    if dependency.expected_family_name() != dependency.actual_family_name() {
        diagnostics.push(diagnostic_for_candidate(
            CapabilityDiagnosticCode::FamilyMismatch,
            candidate,
            Some(dependency.actual_family_name()),
            Some(dependency.identity_text()),
            rich_detail(
                richness,
                "dependency reference used the wrong capability family",
            ),
        ));
        return;
    }

    if !resolvable_keys.contains(&(
        dependency.expected_family_name(),
        dependency.identity_text().to_owned(),
    )) {
        diagnostics.push(diagnostic_for_candidate(
            CapabilityDiagnosticCode::MissingDependency,
            candidate,
            Some(dependency.expected_family_name()),
            Some(dependency.identity_text()),
            rich_detail(
                richness,
                "dependency reference did not resolve to a registered capability",
            ),
        ));
    }
}

fn accepted_capabilities_from_keys(
    accepted_registration_keys: &BTreeSet<AcceptedRegistrationKey>,
) -> RegisteredCapabilitySet {
    let accepted_families = accepted_registration_keys
        .iter()
        .map(|(family_name, _)| *family_name)
        .collect::<BTreeSet<_>>();

    RegisteredCapabilitySet::from_counts(accepted_families.len(), accepted_registration_keys.len())
}

fn accepted_candidate_keys(
    candidates: &[RegistrationCandidate],
    duplicate_keys: &BTreeSet<CandidateKey>,
    cycle_keys: &BTreeSet<CandidateKey>,
) -> BTreeSet<AcceptedRegistrationKey> {
    let mut resolvable_keys = resolvable_dependency_keys(candidates, duplicate_keys);
    resolvable_keys.retain(|key| !cycle_keys.contains(key));
    let mut accepted_registration_keys = BTreeSet::new();

    for candidate in candidates {
        if candidate_is_accepted(candidate, duplicate_keys, cycle_keys, &resolvable_keys) {
            accepted_registration_keys.insert(candidate_key(candidate));
        }
    }

    accepted_registration_keys
}

fn candidate_is_accepted(
    candidate: &RegistrationCandidate,
    duplicate_keys: &BTreeSet<CandidateKey>,
    cycle_keys: &BTreeSet<CandidateKey>,
    resolvable_keys: &BTreeSet<CandidateKey>,
) -> bool {
    candidate.support_kind() == CapabilitySupportKind::Admitted
        && !duplicate_keys.contains(&candidate_key(candidate))
        && !cycle_keys.contains(&candidate_key(candidate))
        && candidate.descriptor_diagnostics().is_empty()
        && candidate_dependency_is_satisfied(candidate, resolvable_keys)
}

fn candidate_dependency_is_satisfied(
    candidate: &RegistrationCandidate,
    resolvable_keys: &BTreeSet<CandidateKey>,
) -> bool {
    candidate.dependencies().iter().all(|dependency| {
        dependency.expected_family_name() == dependency.actual_family_name()
            && resolvable_keys.contains(&(
                dependency.expected_family_name(),
                dependency.identity_text().to_owned(),
            ))
    })
}

fn resolvable_dependency_keys(
    candidates: &[RegistrationCandidate],
    duplicate_keys: &BTreeSet<CandidateKey>,
) -> BTreeSet<CandidateKey> {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.support_kind() == CapabilitySupportKind::Admitted
                && !duplicate_keys.contains(&candidate_key(candidate))
                && candidate.descriptor_diagnostics().is_empty()
        })
        .map(candidate_key)
        .collect()
}

fn candidate_key(candidate: &RegistrationCandidate) -> CandidateKey {
    (
        candidate.family_name(),
        candidate.identity_text().to_owned(),
    )
}

fn theme_token_alias_cycle_keys(
    candidates: &[RegistrationCandidate],
    duplicate_keys: &BTreeSet<CandidateKey>,
) -> BTreeSet<CandidateKey> {
    let resolvable_keys = resolvable_dependency_keys(candidates, duplicate_keys);
    candidates
        .iter()
        .filter(|candidate| candidate.family_name() == THEME_TOKEN_FAMILY_NAME)
        .filter(|candidate| resolvable_keys.contains(&candidate_key(candidate)))
        .filter(|candidate| {
            theme_token_alias_reaches_itself(candidate, candidates, &resolvable_keys)
        })
        .map(candidate_key)
        .collect()
}

fn theme_token_alias_reaches_itself(
    candidate: &RegistrationCandidate,
    candidates: &[RegistrationCandidate],
    resolvable_keys: &BTreeSet<CandidateKey>,
) -> bool {
    let start_key = candidate_key(candidate);
    let mut visited = BTreeSet::new();
    let mut frontier = theme_token_alias_targets(candidate, resolvable_keys);

    while let Some(next_key) = frontier.pop() {
        if next_key == start_key {
            return true;
        }
        if visited.insert(next_key.clone())
            && let Some(next_candidate) = find_candidate(candidates, &next_key)
        {
            frontier.extend(theme_token_alias_targets(next_candidate, resolvable_keys));
        }
    }
    false
}

fn theme_token_alias_targets(
    candidate: &RegistrationCandidate,
    resolvable_keys: &BTreeSet<CandidateKey>,
) -> Vec<CandidateKey> {
    candidate
        .dependencies()
        .iter()
        .filter(|dependency| {
            dependency.expected_family_name() == THEME_TOKEN_FAMILY_NAME
                && dependency.actual_family_name() == THEME_TOKEN_FAMILY_NAME
        })
        .map(|dependency| {
            (
                dependency.expected_family_name(),
                dependency.identity_text().to_owned(),
            )
        })
        .filter(|key| resolvable_keys.contains(key))
        .collect()
}

fn find_candidate<'a>(
    candidates: &'a [RegistrationCandidate],
    key: &CandidateKey,
) -> Option<&'a RegistrationCandidate> {
    candidates
        .iter()
        .find(|candidate| &candidate_key(candidate) == key)
}

fn diagnostic_for_candidate(
    code: CapabilityDiagnosticCode,
    candidate: &RegistrationCandidate,
    related_family_name: Option<&'static str>,
    related_identity_text: Option<&str>,
    detail: Option<String>,
) -> CapabilityRegistrationDiagnostic {
    CapabilityRegistrationDiagnostic::error(
        code,
        Some(candidate.family_name()),
        Some(candidate.identity_text()),
        related_family_name,
        related_identity_text,
        detail,
    )
}

fn rich_detail(richness: CapabilityDiagnosticRichness, detail: &str) -> Option<String> {
    match richness {
        CapabilityDiagnosticRichness::Minimal => None,
        CapabilityDiagnosticRichness::Rich => Some(detail.to_owned()),
    }
}
