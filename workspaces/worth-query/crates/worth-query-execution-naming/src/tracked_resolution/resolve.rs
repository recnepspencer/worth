//! Tracked resolution of a held `WorthQueryName<K>` at the reader's basis.

use worth_query_declaration::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
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
use super::NameReadDependencies;
use super::{ExecutionNameReadDenial, ExecutionNameReader, ExecutionNameResolveDenial};
use crate::admission::{kind_verified_resolved_name, verify_resolved_subject, RequestedSubject};

type Resolved<H, K> = Result<ResolvedName<H>, ExecutionNameResolveDenial<K>>;

impl ExecutionNameReader<'_> {
    /// Resolves a held entity name at the reader's basis under `selection`.
    ///
    /// `Exact` on two or more live successors refuses `Ambiguous` with every
    /// candidate; it never picks a least member. Records the answer's lineage
    /// footprint, or the held digest when the name is still unknown.
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
        let requested = RequestedSubject {
            kind: PersistentNameSubjectKind::Entity,
            name: Entity::IDENTIFIER,
        };
        self.resolve_held(
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
        self.resolve_held(
            held,
            requested,
            selection,
            dependencies,
            |target| match target {
                PersistentNameTarget::Relation { relation, .. } => Some(*relation),
                _ => None,
            },
        )
    }

    fn resolve_held<K, H>(
        &self,
        held: &WorthQueryName<K>,
        requested: RequestedSubject<'_>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
        handle: impl Fn(&PersistentNameTarget) -> Option<H>,
    ) -> Resolved<H, K> {
        let branch = self.branch().clone();
        let lift = LiftAt {
            runtime: self.runtime,
            requested,
        };
        self.with_view(|view| {
            let Some(key) = view.recorded_name(requested.kind, held.as_bytes()) else {
                dependencies.record_awaited_birth(&branch, *held.as_bytes());
                return Err(ExecutionNameResolveDenial::Resolution(
                    NameResolutionDenial::UnknownName,
                ));
            };
            verify_resolved_subject(self.runtime, &view, key, requested)?;
            let (outcome, unrecorded_carrier) = match lower_selection(&view, selection) {
                LoweredSelection::Relational(lowered) => (view.resolve(key, &lowered), false),
                LoweredSelection::UnrecordedCarrier => (
                    view.resolve(key, &PersistentNameSelection::AllDescendants),
                    true,
                ),
            };
            dependencies.record_footprint(&branch, &outcome.footprint);
            match (outcome.result, unrecorded_carrier) {
                (Ok(PersistentNameResolution::Retired { at_commit }), _) => {
                    Ok(ResolvedName::Retired {
                        at_commit: at_commit.0,
                    })
                }
                (Ok(PersistentNameResolution::Exact(target)), true) => {
                    lift.no_match(&view, vec![target])
                }
                (Ok(PersistentNameResolution::Plural(targets)), true) => {
                    lift.no_match(&view, targets)
                }
                (Ok(PersistentNameResolution::Exact(target)), false) => {
                    Ok(ResolvedName::Exact(lift.handle(&view, &target, &handle)?))
                }
                (Ok(PersistentNameResolution::Plural(targets)), false) => {
                    let handles = targets
                        .iter()
                        .map(|target| lift.handle(&view, target, &handle))
                        .collect::<Result<_, _>>()?;
                    Ok(ResolvedName::Plural(handles))
                }
                (Err(denial), _) => {
                    if matches!(denial, PersistentNameResolutionDenial::UnknownName { .. }) {
                        dependencies.record_awaited_birth(&branch, *held.as_bytes());
                    }
                    Err(lift.denial(&view, denial))
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
    fn handle<K, H>(
        &self,
        view: &PersistentNameObservationView<'_>,
        target: &PersistentNameTarget,
        handle: &impl Fn(&PersistentNameTarget) -> Option<H>,
    ) -> Result<H, ExecutionNameResolveDenial<K>> {
        verify_resolved_subject(self.runtime, view, target.key(), self.requested)?;
        handle(target).ok_or(ExecutionNameResolveDenial::Read(
            ExecutionNameReadDenial::SubjectKindMismatch {
                expected: self.requested.kind,
                actual: target.key().subject_kind(),
            },
        ))
    }

    fn names<K>(
        &self,
        view: &PersistentNameObservationView<'_>,
        keys: impl IntoIterator<Item = PersistentNameKey>,
    ) -> Result<Vec<WorthQueryName<K>>, ExecutionNameResolveDenial<K>> {
        keys.into_iter()
            .map(|key| {
                kind_verified_resolved_name(self.runtime, view, key, self.requested)
                    .map_err(ExecutionNameResolveDenial::Read)
            })
            .collect()
    }

    fn no_match<K, H>(
        &self,
        view: &PersistentNameObservationView<'_>,
        live: Vec<PersistentNameTarget>,
    ) -> Result<ResolvedName<H>, ExecutionNameResolveDenial<K>> {
        let live_candidates = self.names(view, live.iter().map(PersistentNameTarget::key))?;
        Err(ExecutionNameResolveDenial::Resolution(
            NameResolutionDenial::NoMatchingCandidate { live_candidates },
        ))
    }

    fn denial<K>(
        &self,
        view: &PersistentNameObservationView<'_>,
        denial: PersistentNameResolutionDenial,
    ) -> ExecutionNameResolveDenial<K> {
        let lifted = match denial {
            PersistentNameResolutionDenial::Ambiguous { candidates } => self
                .names(view, candidates)
                .map(|candidates| NameResolutionDenial::Ambiguous { candidates }),
            PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates } => {
                self.names(view, live_candidates).map(|live_candidates| {
                    NameResolutionDenial::NoMatchingCandidate { live_candidates }
                })
            }
            PersistentNameResolutionDenial::BasisPruned { .. } => {
                Ok(NameResolutionDenial::BasisPruned)
            }
            PersistentNameResolutionDenial::UnknownName { .. } => {
                Ok(NameResolutionDenial::UnknownName)
            }
            PersistentNameResolutionDenial::EmptySelectionFilter => {
                return ExecutionNameResolveDenial::Read(
                    ExecutionNameReadDenial::EmptySelectionFilter,
                );
            }
        };
        match lifted {
            Ok(denial) => ExecutionNameResolveDenial::Resolution(denial),
            Err(read) => read,
        }
    }
}
