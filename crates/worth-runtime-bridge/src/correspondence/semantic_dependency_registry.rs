use std::collections::{BTreeMap, HashMap};

use crate::error::{BridgeBuildError, BridgeBuildErrorKind};

use super::runtime_world_admission::RuntimeWorldCorrespondenceInspectionLedger;
use super::{
    BridgeInstalledBindingKey, BridgeSemanticCorrespondenceRegistration,
    BridgeSemanticDependencyCandidate, BridgeSignalAspectTargetDeclaration,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct AdmittedSemanticDependencyRegistry {
    authoritative: Vec<BridgeSemanticCorrespondenceRegistration>,
    index: BTreeMap<String, usize>,
    by_authority: BTreeMap<String, usize>,
    currentness_index: InstalledBindingCurrentnessIndex,
    signal_graph_instance_id: Option<u64>,
}

/// Derived currentness authority for installed source bindings.
///
/// This index intentionally owns no reference to the authoritative
/// registration storage. Its direct lookup is the only Runtime World
/// currentness operation and records that operation immediately before the
/// `HashMap::get`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InstalledBindingCurrentnessIndex {
    generations: HashMap<BridgeInstalledBindingKey, u64>,
}

impl InstalledBindingCurrentnessIndex {
    fn insert_or_max(&mut self, candidate: &BridgeSemanticDependencyCandidate) {
        let key = candidate.installed_binding_key();
        let generation = candidate.source_installation_generation();
        self.generations
            .entry(key)
            .and_modify(|current| *current = (*current).max(generation))
            .or_insert(generation);
    }

    fn rebuilt_from(registrations: &[BridgeSemanticCorrespondenceRegistration]) -> Self {
        let mut index = Self::default();
        for registration in registrations {
            index.insert_or_max(&registration.dependency);
        }
        index
    }

    fn rebuild_has_exact_parity(
        &self,
        registrations: &[BridgeSemanticCorrespondenceRegistration],
    ) -> bool {
        self == &Self::rebuilt_from(registrations)
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.generations.clear();
    }

    pub(crate) fn lookup(
        &self,
        candidate: &BridgeSemanticDependencyCandidate,
        inspection: &RuntimeWorldCorrespondenceInspectionLedger,
    ) -> Option<u64> {
        inspection.record_binding_index_lookup();
        self.generations
            .get(&candidate.installed_binding_key())
            .copied()
    }
}

pub(crate) struct AdmittedSemanticDependencyExtension {
    registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
    new_registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
    updated_registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
    counters: SemanticDependencyExtensionCounters,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SemanticDependencyExtensionCounters {
    pub(crate) existing_key_lookups: usize,
    pub(crate) batch_key_lookups: usize,
}

#[derive(Debug)]
pub(crate) struct SemanticDependencyExtensionDenial {
    pub(crate) error: BridgeBuildError,
    pub(crate) counters: SemanticDependencyExtensionCounters,
}

impl AdmittedSemanticDependencyRegistry {
    pub(crate) fn freeze(
        mut registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
    ) -> Result<Self, BridgeBuildError> {
        registrations.sort_by_key(BridgeSemanticCorrespondenceRegistration::canonical_key);
        let mut index: BTreeMap<String, usize> = BTreeMap::new();
        let mut authoritative: Vec<BridgeSemanticCorrespondenceRegistration> = Vec::new();
        let mut by_authority: BTreeMap<String, usize> = BTreeMap::new();
        let mut currentness_index = InstalledBindingCurrentnessIndex::default();
        let mut signal_graph_instance_id = None;
        for registration in registrations {
            let registration_graph = registration.signal_graph_instance_id();
            if signal_graph_instance_id
                .is_some_and(|installed_graph| installed_graph != registration_graph)
            {
                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::MixedSemanticCorrespondenceSignalGraphs,
                    "One Bridge runtime cannot own semantic correspondences for multiple Signal graphs. Install an explicit bridge runtime per executable graph.",
                ));
            }
            signal_graph_instance_id = Some(registration_graph);
            let candidate = &registration.dependency;
            currentness_index.insert_or_max(candidate);
            let key = candidate.canonical_registration_key();
            let authority_key = candidate.authority_registration_key();
            if let Some(existing) = by_authority
                .get(&authority_key)
                .map(|position| &authoritative[*position])
            {
                if existing.dependency != registration.dependency {
                    return Err(BridgeBuildError::new(
                        BridgeBuildErrorKind::AmbiguousSemanticDependencyRegistration,
                        "One installed source dependency authority resolved to conflicting semantic meaning.",
                    ));
                }
            }
            if let Some(position) = index.get(&key).copied() {
                authoritative[position]
                    .extend_targets(&registration)
                    .map_err(|_| {
                        BridgeBuildError::new(
                            BridgeBuildErrorKind::AmbiguousSemanticDependencyRegistration,
                            "Semantic dependency registration key resolved to conflicting meaning.",
                        )
                    })?;
                continue;
            }
            let position = authoritative.len();
            index.insert(key, position);
            by_authority.insert(authority_key, position);
            authoritative.push(registration);
        }
        Ok(Self {
            authoritative,
            index,
            by_authority,
            currentness_index,
            signal_graph_instance_id,
        })
    }

    pub(crate) fn resolve(
        &self,
        candidate: &BridgeSemanticDependencyCandidate,
    ) -> Option<Vec<BridgeSignalAspectTargetDeclaration>> {
        self.index
            .get(&candidate.canonical_registration_key())
            .map(|position| &self.authoritative[*position])
            .filter(|installed| installed.dependency == *candidate)
            .map(|installed| installed.targets.clone())
    }

    pub(crate) fn currentness_index(&self) -> &InstalledBindingCurrentnessIndex {
        &self.currentness_index
    }

    pub(crate) fn rebuild_has_exact_parity(&self) -> bool {
        let currentness_parity = self
            .currentness_index
            .rebuild_has_exact_parity(&self.authoritative);
        Self::freeze(self.authoritative.clone()).is_ok_and(|rebuilt| {
            currentness_parity
                && rebuilt.index == self.index
                && rebuilt.by_authority == self.by_authority
                && rebuilt.authoritative == self.authoritative
                && rebuilt.signal_graph_instance_id == self.signal_graph_instance_id
        })
    }

    pub(crate) fn reconstruct_derived_indexes(&self) -> Result<Self, BridgeBuildError> {
        Self::freeze(self.authoritative.clone())
    }

    pub(crate) fn authoritative_count(&self) -> usize {
        self.authoritative.len()
    }

    pub(crate) fn signal_graph_instance_id(&self) -> Option<u64> {
        self.signal_graph_instance_id
    }

    #[cfg(test)]
    pub(crate) fn destroy_derived_indexes(&mut self) {
        self.index.clear();
        self.by_authority.clear();
        self.currentness_index.clear();
        self.signal_graph_instance_id = None;
    }

    pub(crate) fn admit_extension(
        &self,
        registrations: &[BridgeSemanticCorrespondenceRegistration],
    ) -> Result<AdmittedSemanticDependencyExtension, SemanticDependencyExtensionDenial> {
        let mut counters = SemanticDependencyExtensionCounters::default();
        let mut batch_by_key: BTreeMap<String, BridgeSemanticCorrespondenceRegistration> =
            BTreeMap::new();
        let mut batch_by_authority = BTreeMap::new();
        let mut new_registrations = Vec::new();
        let mut updated_registrations = Vec::new();
        for registration in registrations {
            require_compatible_signal_graph(self, registration, counters)?;
            let key = registration.dependency.canonical_registration_key();
            let authority_key = registration.dependency.authority_registration_key();
            counters.existing_key_lookups += 2;
            require_compatible_registration(
                self.by_authority
                    .get(&authority_key)
                    .map(|position| &self.authoritative[*position]),
                registration,
                counters,
            )?;
            if let Some(existing) = self
                .index
                .get(&key)
                .map(|position| &self.authoritative[*position])
            {
                require_compatible_registration(Some(existing), registration, counters)?;
                if existing.has_new_targets(registration) {
                    updated_registrations.push(registration.clone());
                }
                continue;
            }
            counters.batch_key_lookups += 2;
            require_compatible_registration(
                batch_by_authority.get(&authority_key),
                registration,
                counters,
            )?;
            if let Some(existing) = batch_by_key.get_mut(&key) {
                existing.extend_targets(registration).map_err(|_| {
                    extension_denial(
                        BridgeBuildErrorKind::AmbiguousSemanticDependencyRegistration,
                        "One semantic dependency extension resolved to conflicting meaning.",
                        counters,
                    )
                })?;
                continue;
            }
            batch_by_authority.insert(authority_key, registration.clone());
            batch_by_key.insert(key, registration.clone());
        }
        new_registrations.extend(batch_by_key.into_values());
        Ok(AdmittedSemanticDependencyExtension {
            registrations: registrations.to_vec(),
            new_registrations,
            updated_registrations,
            counters,
        })
    }
}

impl AdmittedSemanticDependencyExtension {
    pub(crate) fn registrations(&self) -> &[BridgeSemanticCorrespondenceRegistration] {
        &self.registrations
    }

    pub(crate) const fn counters(&self) -> SemanticDependencyExtensionCounters {
        self.counters
    }

    pub(crate) fn commit(self, registry: &mut AdmittedSemanticDependencyRegistry) -> usize {
        let committed = self.new_registrations.len() + self.updated_registrations.len();
        for registration in self.updated_registrations {
            let key = registration.dependency.canonical_registration_key();
            let position = *registry
                .index
                .get(&key)
                .expect("admitted semantic extension retains its authoritative position");
            registry.authoritative[position]
                .extend_targets(&registration)
                .expect("admitted semantic extension remains compatible at commit");
            registry
                .currentness_index
                .insert_or_max(&registration.dependency);
        }
        for registration in self.new_registrations {
            registry
                .currentness_index
                .insert_or_max(&registration.dependency);
            registry.authoritative.push(registration);
        }
        registry
            .authoritative
            .sort_by_key(BridgeSemanticCorrespondenceRegistration::canonical_key);
        rebuild_lookup_indexes(registry);
        if let Some(first) = self.registrations.first() {
            registry.signal_graph_instance_id = Some(first.signal_graph_instance_id());
        }
        committed
    }
}

fn rebuild_lookup_indexes(registry: &mut AdmittedSemanticDependencyRegistry) {
    registry.index.clear();
    registry.by_authority.clear();
    for (position, registration) in registry.authoritative.iter().enumerate() {
        registry.index.insert(
            registration.dependency.canonical_registration_key(),
            position,
        );
        registry.by_authority.insert(
            registration.dependency.authority_registration_key(),
            position,
        );
    }
}

fn require_compatible_signal_graph(
    registry: &AdmittedSemanticDependencyRegistry,
    registration: &BridgeSemanticCorrespondenceRegistration,
    counters: SemanticDependencyExtensionCounters,
) -> Result<(), SemanticDependencyExtensionDenial> {
    if registry
        .signal_graph_instance_id
        .is_some_and(|installed| installed != registration.signal_graph_instance_id())
    {
        return Err(extension_denial(
            BridgeBuildErrorKind::MixedSemanticCorrespondenceSignalGraphs,
            "One Bridge runtime cannot own semantic correspondences for multiple Signal graphs. Install an explicit bridge runtime per executable graph.",
            counters,
        ));
    }
    Ok(())
}

fn require_compatible_registration(
    existing: Option<&BridgeSemanticCorrespondenceRegistration>,
    candidate: &BridgeSemanticCorrespondenceRegistration,
    counters: SemanticDependencyExtensionCounters,
) -> Result<(), SemanticDependencyExtensionDenial> {
    if existing.is_some_and(|existing| existing.dependency != candidate.dependency) {
        return Err(extension_denial(
            BridgeBuildErrorKind::AmbiguousSemanticDependencyRegistration,
            "An installed semantic dependency registration conflicts with the admitted extension.",
            counters,
        ));
    }
    Ok(())
}

fn extension_denial(
    kind: BridgeBuildErrorKind,
    detail: &str,
    counters: SemanticDependencyExtensionCounters,
) -> SemanticDependencyExtensionDenial {
    SemanticDependencyExtensionDenial {
        error: BridgeBuildError::new(kind, detail),
        counters,
    }
}

#[cfg(test)]
#[path = "semantic_dependency_registry/tests.rs"]
mod tests;
