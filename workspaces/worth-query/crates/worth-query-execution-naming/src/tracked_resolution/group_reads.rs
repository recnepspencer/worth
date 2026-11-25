//! Tracked persistent group reads at the reader's basis.
//!
//! A group kind `G` names its installed group scheme by its declared group
//! name. `group_name_of` answers which live group of `G` holds a member name;
//! `resolve_group` follows a held group name to the groups that carry its
//! meaning now, with entity and relation resolution's exact outcome rules:
//! `Exact` never picks among several live successors, and a retired group
//! answers `Retired`. Each answer records the groups it rested on, so a later
//! commit's group lineage stales exactly the reads it changed.

use worth_query_declaration::facade::application_naming::{
    ApplicationGroupKind, NameSelection, ResolvedName, WorthQueryName,
};
use worth_relational::facade::lineage::{PersistentNameKey, PersistentNameSubjectKind};

use super::{
    ExecutionNameReadDenial, ExecutionNameReader, ExecutionNameResolveDenial, NameReadDependencies,
    NameResolutionCandidates,
};
use crate::admission::{kind_verified_resolved_name, resolved_group_handle, RequestedSubject};

type Resolved<G> = Result<ResolvedName<WorthQueryName<G>>, ExecutionNameResolveDenial<G>>;

impl ExecutionNameReader<'_> {
    /// The live group of kind `G` that holds `member` at the reader's basis,
    /// or `None` when no group of `G` holds it there.
    ///
    /// Records the lookup and its answer, so a later commit that regroups
    /// `member`, or changes the answered group, stales the read.
    ///
    /// Complexity: O(1) expected probes plus O(log t) in the member's own
    /// group tenure.
    pub fn group_name_of<G: ApplicationGroupKind, M>(
        &self,
        member: &WorthQueryName<M>,
        dependencies: &mut NameReadDependencies,
    ) -> Result<Option<WorthQueryName<G>>, ExecutionNameReadDenial> {
        let scheme = self.group_scheme::<G>()?;
        let branch = self.branch().clone();
        let requested = group_subject::<G>();
        let member_digest = *member.as_bytes();
        let answer = self.with_view(|view| {
            let Some(PersistentNameKey::Entity(identity)) =
                view.recorded_name(PersistentNameSubjectKind::Entity, &member_digest)
            else {
                return Ok(None);
            };
            match view.group_of_member(&scheme, &identity) {
                Some(key) => {
                    kind_verified_resolved_name::<G>(self.runtime, &view, key, requested).map(Some)
                }
                None => Ok(None),
            }
        })??;
        dependencies.record_group_member(
            &branch,
            &scheme,
            member_digest,
            answer.map(|group| *group.as_bytes()),
        );
        Ok(answer)
    }

    /// Resolves a held group name of kind `G` at the reader's basis under
    /// `selection`: the live groups that carry its meaning, as their names.
    ///
    /// `Exact` on two or more live successors refuses `Ambiguous` with every
    /// candidate; a group whose every successor retired answers `Retired`.
    ///
    /// Complexity: O(group lineage path from `held` to its frontier).
    pub fn resolve_group<G: ApplicationGroupKind>(
        &self,
        held: &WorthQueryName<G>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
    ) -> Resolved<G> {
        let candidates = self.group_candidates(held, selection, dependencies)?;
        let resolved = candidates.resolve(|_| Ok::<_, std::convert::Infallible>(Vec::new()));
        Ok(resolved.unwrap_or_else(|never| match never {})?)
    }

    /// The live groups `held` matches under `selection`.
    ///
    /// Complexity: O(group lineage path from `held` to its frontier).
    pub fn group_candidates<G: ApplicationGroupKind>(
        &self,
        held: &WorthQueryName<G>,
        selection: &NameSelection,
        dependencies: &mut NameReadDependencies,
    ) -> Result<NameResolutionCandidates<G, WorthQueryName<G>>, ExecutionNameReadDenial> {
        self.group_scheme::<G>()?;
        let branch = self.branch().clone();
        let candidates = self.held_candidates(
            held,
            group_subject::<G>(),
            selection,
            dependencies,
            resolved_group_handle::<G>,
        )?;
        let mut rested_on = vec![*held.as_bytes()];
        rested_on.extend(candidates.handles().map(|group| *group.as_bytes()));
        dependencies.record_groups(&branch, rested_on);
        Ok(candidates)
    }

    /// The installed group scheme governing `G`.
    fn group_scheme<G: ApplicationGroupKind>(&self) -> Result<String, ExecutionNameReadDenial> {
        self.runtime
            .persistent_name_scheme_id(PersistentNameSubjectKind::Group, G::NAME)
            .ok_or(ExecutionNameReadDenial::UnknownScheme { scheme_id: None })
    }
}

const fn group_subject<G: ApplicationGroupKind>() -> RequestedSubject<'static> {
    RequestedSubject {
        kind: PersistentNameSubjectKind::Group,
        name: G::NAME,
    }
}
