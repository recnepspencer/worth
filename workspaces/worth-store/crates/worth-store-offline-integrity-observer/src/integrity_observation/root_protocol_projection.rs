use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity, PhysicalByteRange,
};

use super::families::{root_manifest::ROOT_MANIFEST_BYTES, SelectorRole, ROOT_SELECTOR_BYTES};
use super::root_protocol_walk::{AddressedRootExpectation, RootEntry, SelectorEntry};
use super::{
    OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIndeterminatePhysicalReason, OfflineIntegrityOutcome, OfflinePhysicalBlastRadius,
    OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization,
};

const ROOTS_RELATIVE: &str = "families/records/roots";

pub(crate) fn selector_observations(
    selectors: Vec<SelectorEntry>,
) -> Vec<OfflineArtifactObservation> {
    selectors
        .into_iter()
        .map(|entry| {
            let family = match entry.role {
                SelectorRole::Current => PhysicalArtifactFamily::CurrentRootSelector,
                SelectorRole::Previous => PhysicalArtifactFamily::PreviousRootSelector,
            };
            let identity = entry.observed_identity.map_or_else(
                || selector_role_name(entry.role).to_owned(),
                |identity| format!("selector:{identity:016x}"),
            );
            add_duplicate_evidence(
                artifact_observation(
                    entry.relative,
                    family.into(),
                    identity,
                    PhysicalArtifactGeneration::NotEncoded,
                    ROOT_SELECTOR_BYTES,
                    entry.outcome,
                ),
                entry.physical_alias_of,
                entry.semantic_duplicate,
            )
        })
        .collect()
}

pub(crate) fn root_observations(
    addressed: &BTreeMap<u64, AddressedRootExpectation>,
    roots: Vec<RootEntry>,
    incomplete: Option<OfflineIndeterminatePhysicalReason>,
) -> Vec<OfflineArtifactObservation> {
    let present: BTreeSet<_> = roots
        .iter()
        .map(|entry| entry.expected_generation)
        .collect();
    let mut observations: Vec<_> = roots
        .into_iter()
        .map(|entry| {
            add_duplicate_evidence(
                root_observation(entry.relative, entry.expected_generation, entry.outcome),
                entry.physical_alias_of,
                entry.semantic_duplicate,
            )
        })
        .collect();
    for generation in addressed
        .keys()
        .filter(|generation| !present.contains(generation))
    {
        observations.push(root_observation(
            format!("{ROOTS_RELATIVE}/root-{generation:016x}.manifest"),
            *generation,
            if let Some(reason) = incomplete {
                OfflineIntegrityOutcome::Indeterminate(reason)
            } else {
                damage(
                    OfflinePhysicalDamageCause::MissingArtifact,
                    None,
                    None,
                    OfflinePhysicalBlastRadius::ReachableRootSubtree,
                )
            },
        ));
    }
    observations.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
    observations
}

fn add_duplicate_evidence(
    mut observation: OfflineArtifactObservation,
    physical_alias_of: Option<String>,
    semantic_duplicate: bool,
) -> OfflineArtifactObservation {
    if let Some(first_path) = physical_alias_of {
        observation = observation.with_duplicate(OfflineArtifactDuplicateEvidence::PhysicalAlias {
            first_path: first_path.into_boxed_str(),
        });
    }
    if semantic_duplicate {
        observation =
            observation.with_duplicate(OfflineArtifactDuplicateEvidence::SemanticIdentity);
    }
    observation
}

fn root_observation(
    relative: String,
    expected_generation: u64,
    outcome: OfflineIntegrityOutcome,
) -> OfflineArtifactObservation {
    artifact_observation(
        relative,
        PhysicalArtifactFamily::RootManifest.into(),
        // Identity names the inspected, parent-addressed scope even when the
        // decoded payload claims a different generation.
        format!("root:{expected_generation:016x}"),
        PhysicalArtifactGeneration::encoded(expected_generation)
            .unwrap_or(PhysicalArtifactGeneration::NotEncoded),
        ROOT_MANIFEST_BYTES,
        outcome,
    )
}

fn artifact_observation(
    relative: String,
    family: super::OfflineArtifactFamily,
    identity: String,
    generation: PhysicalArtifactGeneration,
    byte_length: usize,
    outcome: OfflineIntegrityOutcome,
) -> OfflineArtifactObservation {
    OfflineArtifactObservation::new(
        relative,
        family,
        PhysicalArtifactIdentity::new(identity).expect("bounded observer identity"),
        generation,
        (byte_length > 0).then(|| PhysicalByteRange::new(0, byte_length as u64).unwrap()),
        outcome,
    )
}

fn selector_role_name(role: SelectorRole) -> &'static str {
    match role {
        SelectorRole::Current => "current-selector",
        SelectorRole::Previous => "previous-selector",
    }
}

fn damage(
    cause: OfflinePhysicalDamageCause,
    range: Option<(u64, u64)>,
    field: Option<super::OfflinePhysicalFormatField>,
    blast: OfflinePhysicalBlastRadius,
) -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
        cause, range, field, blast,
    ))
}
