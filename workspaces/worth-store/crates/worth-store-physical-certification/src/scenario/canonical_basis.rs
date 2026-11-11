use std::collections::BTreeSet;

use worth_foundational::canonicalization_api::lower_lane::basis::{
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalizationRuleVersion,
};
use worth_foundational::canonicalization_api::lower_lane::digest::{
    admit_canonical_sequence_digest_derivation, derive_canonical_digest,
    CanonicalDigestAlgorithmId, CanonicalSingleSequenceDigestAlgorithmSlot,
};
use worth_foundational::InternedString;
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreAspectBoundaryFact, StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

use super::canonical_tokens::{
    actor_role_token, blob_harness_access_mode_token, blob_harness_actor_mix_token,
    blob_harness_chunk_size_class_token, blob_harness_failure_point_token,
    blob_harness_placement_class_token, blob_harness_security_scope_class_token,
    blob_harness_size_class_token, expectation_token, family_token, fault_token, intent_token,
    non_claim_token,
};
use super::definition::PhysicalSimulationScenarioDefinition;
use super::denial::PhysicalScenarioDefinitionDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalScenarioCanonicalBasis {
    ready: CanonicalBasisReadyArtifact,
}

impl PhysicalScenarioCanonicalBasis {
    pub(crate) fn from_definition(
        definition: &PhysicalSimulationScenarioDefinition,
    ) -> Result<Self, PhysicalScenarioDefinitionDenial> {
        let mut entries = vec![
            text_entry("scenario.family", family_token(definition.family())),
            text_entry("scenario.intent", intent_token(definition.intent())),
            text_entry(
                "scenario.schedule.yieldpoint",
                definition.schedule().production_boundary_yieldpoint(),
            ),
            text_entry("scenario.fault", fault_token(definition.fault().kind())),
            text_entry(
                "scenario.expectation",
                expectation_token(definition.expectation().kind()),
            ),
        ];
        entries.extend(canonical_fixture_entries(definition.fixtures())?);
        entries.extend(canonical_actor_entries(definition));
        entries.extend(canonical_non_claim_entries(definition));
        entries.extend(canonical_blob_harness_entries(definition));
        let version = scenario_canonicalization_version();
        match prepare_canonical_basis_sequence(version, SCENARIO_DOMAIN, entries) {
            TransitionOutcome::Success(ready) => Ok(Self { ready }),
            TransitionOutcome::Denied(denial) => {
                Err(PhysicalScenarioDefinitionDenial::ScenarioCanonicalBasisDenied(denial))
            }
            TransitionOutcome::Deferred(value) => match value {},
            TransitionOutcome::Stale(value) => match value {},
            TransitionOutcome::RebindRequired(value) => match value {},
            TransitionOutcome::Failed(value) => match value {},
        }
    }

    pub(crate) fn ready(self) -> CanonicalBasisReadyArtifact {
        self.ready
    }
}

const SCENARIO_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("store.physical.simulation.scenario");
const SCENARIO_FIELD_KIND: CanonicalBasisEntryKind =
    CanonicalBasisEntryKind::Future("store-physical-scenario-field");
const SCENARIO_FIXTURE_KIND: CanonicalBasisEntryKind =
    CanonicalBasisEntryKind::Future("store-physical-scenario-native-fixture");

fn canonical_fixture_entries(
    fixtures: &[StoreAspectBoundaryFact],
) -> Result<Vec<CanonicalBasisEntry>, PhysicalScenarioDefinitionDenial> {
    let mut entries = Vec::new();
    for (fixture_index, fixture) in fixtures.iter().enumerate() {
        let ready = prepare_fixture_basis(fixture)?;
        for (entry_index, entry) in ready.payload().entries().iter().enumerate() {
            entries.push(CanonicalBasisEntry::new(
                SCENARIO_DOMAIN,
                named_locus(format!(
                    "fixture.{fixture_index:04}.entry.{entry_index:04}.locus"
                )),
                SCENARIO_FIXTURE_KIND,
                CanonicalBasisValue::ExactText(locus_token(entry.locus()).into()),
            ));
            entries.push(CanonicalBasisEntry::new(
                SCENARIO_DOMAIN,
                named_locus(format!(
                    "fixture.{fixture_index:04}.entry.{entry_index:04}.value"
                )),
                entry.kind(),
                entry.value().clone(),
            ));
        }
    }
    Ok(entries)
}

pub(crate) fn fixture_canonical_sort_key(
    fixture: &StoreAspectBoundaryFact,
) -> Result<[u8; 32], PhysicalScenarioDefinitionDenial> {
    let ready = prepare_fixture_basis(fixture)?;
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        ready.payload().domain(),
        ready.payload().version().clone(),
    );
    match admit_canonical_sequence_digest_derivation(ready, slot) {
        TransitionOutcome::Success(ready) => Ok(*derive_canonical_digest(ready).value().bytes()),
        TransitionOutcome::Denied(denial) => {
            Err(PhysicalScenarioDefinitionDenial::ScenarioDigestDerivationDenied(denial))
        }
        TransitionOutcome::Deferred(value) => match value {},
        TransitionOutcome::Stale(value) => match value {},
        TransitionOutcome::RebindRequired(value) => match value {},
        TransitionOutcome::Failed(value) => match value {},
    }
}

fn prepare_fixture_basis(
    fixture: &StoreAspectBoundaryFact,
) -> Result<CanonicalBasisReadyArtifact, PhysicalScenarioDefinitionDenial> {
    let version = scenario_canonicalization_version();
    match StoreCanonicalBasisConstruction::for_family(StoreCanonicalBasisFamily::AspectBoundaryFact)
        .with_aspect_boundary_fact(fixture)
        .prepare(version)
    {
        TransitionOutcome::Success(ready) => Ok(ready),
        TransitionOutcome::Denied(denial) => {
            Err(PhysicalScenarioDefinitionDenial::FixtureCanonicalBasisDenied(denial))
        }
        TransitionOutcome::Deferred(value) => match value {},
        TransitionOutcome::Stale(value) => match value {},
        TransitionOutcome::RebindRequired(value) => match value {},
        TransitionOutcome::Failed(value) => match value {},
    }
}

fn canonical_actor_entries(
    definition: &PhysicalSimulationScenarioDefinition,
) -> Vec<CanonicalBasisEntry> {
    definition
        .actors()
        .iter()
        .enumerate()
        .flat_map(|(index, actor)| {
            [
                text_entry(
                    format!("actor.{index:04}.role"),
                    actor_role_token(actor.role()),
                ),
                text_entry(format!("actor.{index:04}.id"), actor.id()),
            ]
        })
        .collect()
}

fn canonical_non_claim_entries(
    definition: &PhysicalSimulationScenarioDefinition,
) -> Vec<CanonicalBasisEntry> {
    definition
        .expectation()
        .non_claims()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(index, non_claim)| {
            text_entry(format!("non_claim.{index:04}"), non_claim_token(non_claim))
        })
        .collect()
}

fn canonical_blob_harness_entries(
    definition: &PhysicalSimulationScenarioDefinition,
) -> Vec<CanonicalBasisEntry> {
    let Some(metadata) = definition.expectation().blob_harness_metadata() else {
        return Vec::new();
    };
    let topology = definition
        .expectation()
        .blob_harness_topology()
        .expect("blob harness metadata must carry topology");
    vec![
        text_entry(
            "blob_harness.size_class",
            blob_harness_size_class_token(metadata.size_class()),
        ),
        text_entry(
            "blob_harness.chunk_size_class",
            blob_harness_chunk_size_class_token(metadata.chunk_size_class()),
        ),
        text_entry(
            "blob_harness.placement_class",
            blob_harness_placement_class_token(metadata.placement_class()),
        ),
        text_entry(
            "blob_harness.security_scope_class",
            blob_harness_security_scope_class_token(metadata.security_scope_class()),
        ),
        text_entry(
            "blob_harness.access_mode",
            blob_harness_access_mode_token(metadata.access_mode()),
        ),
        text_entry(
            "blob_harness.failure_point",
            blob_harness_failure_point_token(metadata.failure_point()),
        ),
        text_entry(
            "blob_harness.actor_mix",
            blob_harness_actor_mix_token(metadata.actor_mix()),
        ),
        text_entry(
            "blob_harness.chunk_count",
            topology.chunk_count().to_string(),
        ),
        text_entry(
            "blob_harness.logical_bytes",
            topology.logical_bytes().to_string(),
        ),
        text_entry(
            "blob_harness.chunk_bytes",
            topology.chunk_bytes().to_string(),
        ),
    ]
}

fn text_entry(
    locus: impl Into<InternedString>,
    value: impl Into<InternedString>,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        SCENARIO_DOMAIN,
        CanonicalBasisLocus::Named(locus.into()),
        SCENARIO_FIELD_KIND,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

fn named_locus(locus: impl Into<InternedString>) -> CanonicalBasisLocus {
    CanonicalBasisLocus::Named(locus.into())
}

fn scenario_canonicalization_version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("store.physical.scenario.v1")
        .expect("scenario canonicalization version is static and valid")
}

fn locus_token(locus: &CanonicalBasisLocus) -> String {
    match locus {
        CanonicalBasisLocus::Root => "root".to_owned(),
        CanonicalBasisLocus::EntryOrdinal(ordinal) => format!("ordinal.{ordinal}"),
        CanonicalBasisLocus::Aspect(aspect) => format!("aspect.{}", aspect.as_str()),
        CanonicalBasisLocus::AspectField { aspect, path } => format!(
            "aspect-field.{}.{}",
            aspect.as_str(),
            path.fields()
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(".")
        ),
        CanonicalBasisLocus::Named(name) => interned_token(name),
    }
}

fn interned_token(value: &InternedString) -> String {
    match value {
        InternedString::Raw(raw) => raw.clone(),
        InternedString::Stored(raw) => raw.as_str().to_owned(),
        InternedString::Symbol(symbol) => format!("symbol.{}", symbol.0),
    }
}
