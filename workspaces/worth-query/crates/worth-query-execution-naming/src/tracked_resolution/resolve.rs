//! Tracked resolution of a held `WorthQueryName<K>` at the reader's basis.

use worth_query_declaration::facade::application_naming::{
    NameCandidate, NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameSelection, PersistentNameSubjectKind,
    PersistentNameTarget,
};

use super::selection_lowering::{lower_selection, LoweredSelection};
use super::{
    ExecutionNameReadDenial, ExecutionNameReader, ExecutionNameResolveDenial, NameReadDependencies,
    NameResolutionCandidates,
};
use crate::admission::{kind_verified_resolved_name, verify_resolved_subject, RequestedSubject};

type Resolved<H, K> = Result<ResolvedName<H>, ExecutionNameResolveDenial<K>>;
type Candidates<K, H> = Result<NameResolutionCandidates<K, H>, ExecutionNameReadDenial>;

/// A resolution over an owner read that cannot fail.
fn infallible<T>(resolved: Result<T, std::convert::Infallible>) -> T {
    resolved.unwrap_or_else(|never| match never {})
}

impl ExecutionNameReader<'_> {
    /// Resolves a held entity name at the reader's basis under `selection`.
    ///
    /// `Exact` on two or more live successors refuses `Ambiguous` with every
    /// candidate; it never picks a least member. This reader reads no owners,
    /// so candidates carry none ([`Self::entity_candidates`] reports them).
    /// Records the answer's lineage footprint, or the held digest when the
    /// name is still unknown.
    ///
    /// Complexity: O(lineage path from `held` to its frontier), never
    /// O(names) or O(commits).
    pub fn resolve_entity<Schema, Entity>(
        &self,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
    ) -> Resolved<EntityId, Entity>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let candidates = self.entity_candidates::<Schema, Entity>(held, selection, dependencies)?;
        Ok(infallible(candidates.resolve(|_| Ok(Vec::new())))?)
    }

    /// Resolves a held entity name among the live records `owner` holds.
    ///
    /// `owners_of` reads the owners a candidate's declared owner chain
    /// reaches; those reads belong to the caller, who tracks them. An owner
    /// holding no matching live record refuses `NoCandidateInOwner`.
    ///
    /// Complexity: O(lineage path) plus one `owners_of` read per matched
    /// live record.
    pub fn resolve_entity_in<Schema, Entity, Owner: PartialEq>(
        &self,
        owner: &Owner,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
        mut owners_of: impl FnMut(EntityId) -> Vec<Owner>,
        dependencies: &mut NameReadDependencies,
    ) -> Result<ResolvedName<EntityId>, ExecutionNameResolveDenial<Entity, Owner>>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let candidates = self.entity_candidates::<Schema, Entity>(held, selection, dependencies)?;
        Ok(infallible(
            candidates.resolve_in(owner, |entity| Ok(owners_of(*entity))),
        )?)
    }

    /// The live entities `held` matches under `selection`, before any owner
    /// restriction. Settle them with
    /// [`NameResolutionCandidates::resolve`] or
    /// [`NameResolutionCandidates::resolve_in`] once the reader is released,
    /// so owner reads never run under the name index.
    ///
    /// Complexity: O(lineage path from `held` to its frontier).
    pub fn entity_candidates<Schema, Entity>(
        &self,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
    ) -> Candidates<Entity, EntityId>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let requested = RequestedSubject {
            kind: PersistentNameSubjectKind::Entity,
            name: Entity::IDENTIFIER,
        };
        self.held_candidates(
            held,
            requested,
            selection,
            dependencies,
            |target| match target {
                PersistentNameTarget::Entity { entity, .. } => Some(*entity),
                _ => None,
            },
        )
    }

    /// Resolves a held relation name at the reader's basis under `selection`.
    ///
    /// Complexity: O(lineage path from `held` to its frontier).
    pub fn resolve_relation<Relation>(
        &self,
        held: &WorthQueryName<Relation>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
    ) -> Resolved<RelationId, Relation>
    where
        Relation: fragment::Relation,
    {
        let requested = RequestedSubject {
            kind: PersistentNameSubjectKind::Relation,
            name: Relation::NAME,
        };
        let candidates = self.held_candidates(
            held,
            requested,
            selection,
            dependencies,
            |target| match target {
                PersistentNameTarget::Relation { relation, .. } => Some(*relation),
                _ => None,
            },
        )?;
        Ok(infallible(candidates.resolve(|_| Ok(Vec::new())))?)
    }

    pub(super) fn held_candidates<K, H>(
        &self,
        held: &WorthQueryName<K>,
        requested: RequestedSubject<'_>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
        handle: impl Fn(&PersistentNameTarget) -> Option<H>,
    ) -> Candidates<K, H> {
        let branch = self.branch().clone();
        let lift = LiftAt {
            runtime: self.runtime,
            requested,
        };
        let selects_all = matches!(selection, NameSelection::AllDescendants);
        self.with_view(|view| {
            let Some(key) = view.recorded_name(requested.kind, held.as_bytes()) else {
                dependencies.record_awaited_birth(&branch, *held.as_bytes());
                return Ok(NameResolutionCandidates::refused(
                    NameResolutionDenial::UnknownName,
                ));
            };
            verify_resolved_subject(self.runtime, &view, key, requested)?;
            let (outcome, unrecorded_carrier) = match lower_selection(&view, selection) {
                LoweredSelection::Relational(lowered) => (view.resolve(key, &lowered), false),
                LoweredSelection::UnrecordedCarriers(unborn) => {
                    for carrier in unborn {
                        dependencies.record_awaited_birth(&branch, carrier);
                    }
                    (
                        view.resolve(key, &PersistentNameSelection::AllDescendants),
                        true,
                    )
                }
            };
            dependencies.record_footprint(&branch, &outcome.footprint);
            match (outcome.result, unrecorded_carrier) {
                (Ok(PersistentNameResolution::Retired { at_commit }), _) => {
                    Ok(NameResolutionCandidates::retired(at_commit.0))
                }
                (Ok(PersistentNameResolution::Exact(target)), true) => {
                    lift.no_match(&view, vec![target])
                }
                (Ok(PersistentNameResolution::Plural(targets)), true) => {
                    lift.no_match(&view, targets)
                }
                (Ok(PersistentNameResolution::Exact(target)), false) => {
                    lift.matched(&view, vec![target], selects_all, &handle)
                }
                (Ok(PersistentNameResolution::Plural(targets)), false) => {
                    lift.matched(&view, targets, selects_all, &handle)
                }
                (Err(PersistentNameResolutionDenial::Ambiguous { candidates }), _) => {
                    // Every ambiguous candidate is a live footprint record; if
                    // one were not, the ambiguity still refuses, by name alone.
                    let live = &outcome.footprint.live;
                    let targets = candidates
                        .iter()
                        .filter_map(|key| live.iter().find(|target| target.key() == *key))
                        .copied()
                        .collect::<Vec<_>>();
                    if targets.len() == candidates.len() {
                        lift.matched(&view, targets, false, &handle)
                    } else {
                        let denial = PersistentNameResolutionDenial::Ambiguous { candidates };
                        lift.denial(&view, denial)
                    }
                }
                (Err(denial), _) => {
                    if matches!(denial, PersistentNameResolutionDenial::UnknownName { .. }) {
                        dependencies.record_awaited_birth(&branch, *held.as_bytes());
                    }
                    lift.denial(&view, denial)
                }
            }
        })?
    }
}

/// Lifts Relational answers into kind-typed results, verifying every name.
struct LiftAt<'read, 'kind> {
    runtime: &'read worth_relational::facade::runtime::RelationalRuntime,
    requested: RequestedSubject<'kind>,
}

impl LiftAt<'_, '_> {
    /// The matched live targets as kind-verified names with their handles.
    fn matched<K, H>(
        &self,
        view: &PersistentNameObservationView<'_>,
        targets: Vec<PersistentNameTarget>,
        selects_all: bool,
        handle: &impl Fn(&PersistentNameTarget) -> Option<H>,
    ) -> Candidates<K, H> {
        let matched = targets
            .iter()
            .map(|target| {
                let name =
                    kind_verified_resolved_name(self.runtime, view, target.key(), self.requested)?;
                let handle =
                    handle(target).ok_or(ExecutionNameReadDenial::SubjectKindMismatch {
                        expected: self.requested.kind,
                        actual: target.key().subject_kind(),
                    })?;
                Ok((name, handle))
            })
            .collect::<Result<Vec<_>, ExecutionNameReadDenial>>()?;
        Ok(NameResolutionCandidates::matched(matched, selects_all))
    }

    fn names<K>(
        &self,
        view: &PersistentNameObservationView<'_>,
        keys: impl IntoIterator<Item = PersistentNameKey>,
    ) -> Result<Vec<WorthQueryName<K>>, ExecutionNameReadDenial> {
        keys.into_iter()
            .map(|key| kind_verified_resolved_name(self.runtime, view, key, self.requested))
            .collect()
    }

    fn no_match<K, H>(
        &self,
        view: &PersistentNameObservationView<'_>,
        live: Vec<PersistentNameTarget>,
    ) -> Candidates<K, H> {
        let live_candidates = self.names(view, live.iter().map(PersistentNameTarget::key))?;
        Ok(NameResolutionCandidates::refused(
            NameResolutionDenial::NoMatchingCandidate { live_candidates },
        ))
    }

    /// Relational's refusals as typed outcomes that leave nothing to restrict.
    fn denial<K, H>(
        &self,
        view: &PersistentNameObservationView<'_>,
        denial: PersistentNameResolutionDenial,
    ) -> Candidates<K, H> {
        let lifted = match denial {
            PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates } => {
                NameResolutionDenial::NoMatchingCandidate {
                    live_candidates: self.names(view, live_candidates)?,
                }
            }
            PersistentNameResolutionDenial::BasisPruned { .. } => NameResolutionDenial::BasisPruned,
            PersistentNameResolutionDenial::UnknownName { .. } => NameResolutionDenial::UnknownName,
            PersistentNameResolutionDenial::Ambiguous { candidates } => {
                NameResolutionDenial::Ambiguous {
                    candidates: self
                        .names(view, candidates)?
                        .into_iter()
                        .map(|name| NameCandidate::new(name, Vec::new()))
                        .collect(),
                }
            }
            PersistentNameResolutionDenial::EmptySelectionFilter => {
                return Err(ExecutionNameReadDenial::EmptySelectionFilter);
            }
        };
        Ok(NameResolutionCandidates::refused(lifted))
    }
}
