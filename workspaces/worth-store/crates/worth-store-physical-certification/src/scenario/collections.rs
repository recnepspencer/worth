use std::collections::BTreeSet;

use worth_store_aspect_native::StoreAspectBoundaryFact;

use super::canonical_basis::fixture_canonical_sort_key;
use super::denial::PhysicalScenarioDefinitionDenial;
use super::vocabulary::PhysicalScenarioActor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalScenarioFixtureSet {
    fixtures: Vec<StoreAspectBoundaryFact>,
}

impl PhysicalScenarioFixtureSet {
    pub(crate) fn from_fixtures(
        fixtures: Vec<StoreAspectBoundaryFact>,
    ) -> Result<Self, PhysicalScenarioDefinitionDenial> {
        if fixtures.is_empty() {
            return Err(PhysicalScenarioDefinitionDenial::MissingAspectNativeFixture);
        }
        let mut fixtures = canonical_fixture_entries(fixtures)?;
        fixtures.sort_by_key(|fixture| fixture.sort_key);
        require_unique_fixture_sort_keys(&fixtures)?;
        let fixtures = fixtures
            .into_iter()
            .map(|fixture| fixture.boundary_fact)
            .collect();
        Ok(Self { fixtures })
    }

    pub fn fixtures(&self) -> &[StoreAspectBoundaryFact] {
        &self.fixtures
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalScenarioActorSet {
    actors: Vec<PhysicalScenarioActor>,
}

impl PhysicalScenarioActorSet {
    pub(crate) fn from_actors(
        mut actors: Vec<PhysicalScenarioActor>,
    ) -> Result<Self, PhysicalScenarioDefinitionDenial> {
        if actors.is_empty() {
            return Err(PhysicalScenarioDefinitionDenial::MissingActor);
        }
        require_named_actor_ids(&actors)?;
        require_unique_actor_ids(&actors)?;
        actors.sort();
        Ok(Self { actors })
    }

    pub fn actors(&self) -> &[PhysicalScenarioActor] {
        &self.actors
    }
}

struct CanonicalFixtureEntry {
    sort_key: [u8; 32],
    boundary_fact: StoreAspectBoundaryFact,
}

fn canonical_fixture_entries(
    fixtures: Vec<StoreAspectBoundaryFact>,
) -> Result<Vec<CanonicalFixtureEntry>, PhysicalScenarioDefinitionDenial> {
    fixtures
        .into_iter()
        .map(|boundary_fact| {
            let sort_key = fixture_canonical_sort_key(&boundary_fact)?;
            Ok(CanonicalFixtureEntry {
                sort_key,
                boundary_fact,
            })
        })
        .collect()
}

fn require_unique_fixture_sort_keys(
    fixtures: &[CanonicalFixtureEntry],
) -> Result<(), PhysicalScenarioDefinitionDenial> {
    let mut sort_keys = BTreeSet::new();
    for fixture in fixtures {
        if !sort_keys.insert(fixture.sort_key) {
            return Err(PhysicalScenarioDefinitionDenial::DuplicateAspectNativeFixture);
        }
    }
    Ok(())
}

fn require_named_actor_ids(
    actors: &[PhysicalScenarioActor],
) -> Result<(), PhysicalScenarioDefinitionDenial> {
    for actor in actors {
        if actor.id().trim().is_empty() {
            return Err(PhysicalScenarioDefinitionDenial::UnnamedActorId);
        }
    }
    Ok(())
}

fn require_unique_actor_ids(
    actors: &[PhysicalScenarioActor],
) -> Result<(), PhysicalScenarioDefinitionDenial> {
    let mut actor_ids = BTreeSet::new();
    for actor in actors {
        if !actor_ids.insert(actor.id()) {
            return Err(PhysicalScenarioDefinitionDenial::DuplicateActorId);
        }
    }
    Ok(())
}
