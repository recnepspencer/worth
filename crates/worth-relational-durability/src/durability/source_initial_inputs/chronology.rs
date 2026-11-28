//! Validate normalized original roles against the unchanged selected directory.
use super::{image::*, normalized::*, RelationalInitialInputsRefusal as Refusal};
use crate::durability::data::RelationalCheckpointManifest;
use contracts::branch::version::RelationalBranchVersionOwnerPort as _;
use contracts::history::data::coordinates::BranchId;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture as Lifecycle;
use worth_relational_branch::internal_port::branch::transition::record::RelationalBranchTransition as Transition;
use worth_relational_contracts::internal_port as contracts;

pub mod axes;
use axes::{Axes, ObservationKey};
pub mod transitions;
pub mod world_sources;
use world_sources::WorldSources;

struct Fold {
    current: ExecutionOrderedMap<BranchId, Axes>,
    observations: ExecutionOrderedMap<ObservationKey, Axes>,
    canonical: ExecutionOrderedMap<contracts::history::data::coordinates::CommitId, Axes>,
}
struct CommitPageStream<'a> {
    index: &'a worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex,
    page_ordinal: usize,
    row_offset: usize,
    page: Option<worth_relational_branch::internal_port::history::commit::paged_index::page::ResidentCommitPage>,
}
impl<'a> CommitPageStream<'a> {
    fn new(
        index: &'a worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex,
    ) -> Self {
        Self {
            index,
            page_ordinal: 0,
            row_offset: 0,
            page: None,
        }
    }
    fn next(
        &mut self,
    ) -> Result<Option<std::sync::Arc<worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry>>, Refusal>
{
        loop {
            if let Some(page) = &self.page {
                if let Some(row) = page.page.rows.get(self.row_offset) {
                    self.row_offset += 1;
                    return Ok(Some(std::sync::Arc::new(row.clone())));
                }
                // Drop the finished page before loading the next so at most one
                // decoded commit-index page exists in memory at any moment.
                self.page = None;
                self.page_ordinal += 1;
                self.row_offset = 0;
            }
            if self.page_ordinal >= self.index.descriptor().pages.len() {
                return Ok(None);
            }
            self.page = Some(self.index.read_page_uncached(self.page_ordinal).map_err(
                |cause| {
                    Refusal::Native(crate::durability::data::commit_index_access::stop(0, cause))
                },
            )?);
        }
    }
}
pub(super) fn validate(
    initial: &InitialInputsImage,
    normalized: &NormalizedOriginalInputs,
    manifest: &RelationalCheckpointManifest,
    commit_index: &worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex,
    commitments: &super::input_commitments::OriginalInputCommitments,
    schemas: &[crate::durability::data::DurableBranchRootSchemaImage],
    world: Option<&crate::durability::home_world_catalog::RelationalWorldSourceCatalog>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let Some(InitialInputEntry::Birth(birth)) = initial.entries.first() else {
        return Err(Refusal::Malformed);
    };
    let mut fold = Fold {
        current: Default::default(),
        observations: Default::default(),
        canonical: Default::default(),
    };
    fold.record(&initial.main, Axes::empty(birth), request)?;
    let mut sources = WorldSources::prepare(world, schemas, request)?;
    let mut commits = CommitPageStream::new(commit_index);
    let mut transition = 0usize;
    let mut installed = false;
    for position in 0..=normalized.canonical.len() {
        fold.install(initial, position, transition, &mut installed, request)?;
        while let Some(entry) = normalized.transitions.get(transition) {
            if entry.follows.0 != position as u64 {
                break;
            }
            sources.before_transition(&mut fold, entry, request)?;
            fold.transition(&initial.main, entry, request)?;
            transition += 1;
            fold.install(initial, position, transition, &mut installed, request)?;
        }
        let Some(record) = normalized.canonical.get(position) else {
            break;
        };
        let index = commits.next()?.ok_or(Refusal::Malformed)?;
        commitments.require_record(record, &index, request)?;
        let branch = index.commit.branch_id.clone();
        let mut state = sources.before_canonical(&mut fold, &index, request)?;
        let before = state.clone();
        if state.removed
            || state.lifecycle != Lifecycle::Live
            || state.schema != record.prestate_schema
        {
            return Err(Refusal::Malformed);
        }
        state.generation = state
            .generation
            .checked_advance()
            .map_err(|_| Refusal::Malformed)?;
        state.truth = state.truth.checked_advance().ok_or(Refusal::Malformed)?;
        state.commit = Some(index.clone());
        state.schema = record.result_schema;
        sources.retain(&index, before, state.clone(), request)?;
        if world.is_some() {
            fold.canonical
                .insert_cloned_native_for_request(&index.commit.commit_id, &state, *request)
                .map_err(map)?;
        }
        fold.record(&branch, state, request)?;
    }
    if transition != normalized.transitions.len() || installed != (initial.entries.len() == 2) {
        return Err(Refusal::Malformed);
    }
    sources.finish(&mut fold, &manifest.branch_cells, request)?;
    if manifest.branch_cells.len()
        != fold
            .current
            .iter()
            .filter(|(_, state)| !state.removed)
            .count()
        || manifest.branch_lifecycle.len() != manifest.branch_cells.len()
    {
        return Err(Refusal::Malformed);
    }
    for (cell, lifecycle) in manifest.branch_cells.iter().zip(&manifest.branch_lifecycle) {
        let state = fold
            .current
            .get(&cell.branch_id)
            .ok_or(Refusal::Malformed)?;
        if state.removed
            || state.generation != cell.observation.generation()
            || state.truth != cell.truth_version
            || !state.matches_target(cell.observation.target())
            || lifecycle.branch_id != cell.branch_id
            || lifecycle.posture != state.lifecycle
        {
            return Err(Refusal::Malformed);
        }
        let cell_fork_origin = match (&cell.fork_source_branch_id, &cell.fork_provenance) {
            (None, None) => None,
            (Some(source), Some(prov)) => match prov.target() {
                worth_foundational::FoundationalBranchTarget::Basis(target) => {
                    Some((source.clone(), target.selected_commit_id()))
                }
                worth_foundational::FoundationalBranchTarget::Empty => {
                    return Err(Refusal::Malformed);
                }
            },
            _ => return Err(Refusal::Malformed),
        };
        if state.fork_origin != cell_fork_origin {
            return Err(Refusal::Malformed);
        }
    }
    if manifest.retired_branch_names.len()
        != fold
            .current
            .iter()
            .filter(|(_, state)| state.lifecycle == Lifecycle::Deleting)
            .count()
        || manifest.retired_branch_names.iter().any(|branch| {
            fold.current
                .get(branch)
                .is_none_or(|state| state.lifecycle != Lifecycle::Deleting)
        })
    {
        return Err(Refusal::Malformed);
    }
    let mut required_retired: std::collections::BTreeMap<BranchId, (BranchId, u64)> =
        std::collections::BTreeMap::new();
    let mut visited: std::collections::BTreeSet<BranchId> = std::collections::BTreeSet::new();
    for cell in &manifest.branch_cells {
        if !matches!(
            cell.observation.target(),
            worth_foundational::FoundationalBranchTarget::Basis(_)
        ) {
            continue;
        }
        use std::ops::ControlFlow::{Break, Continue};
        let Ok(_) = worth_relational_branch::internal_port::history::access::ancestry_walk::walk_acyclic_chain(
            cell.branch_id.clone(),
            &mut visited,
            || Ok::<_, std::convert::Infallible>(()),
            |cur| {
                let Some(state) = fold.current.get(&cur) else {
                    return Ok(Break(()));
                };
                let Some((parent, fork_commit)) = &state.fork_origin else {
                    return Ok(Break(()));
                };
                if state.removed {
                    required_retired.insert(cur, (parent.clone(), *fork_commit));
                }
                Ok(Continue(parent.clone()))
            },
        );
    }
    for edge in &manifest.naming.retired_forks {
        if required_retired.remove(&edge.branch) != Some((edge.parent.clone(), edge.fork_commit)) {
            return Err(Refusal::Malformed);
        }
    }
    if !required_retired.is_empty() {
        return Err(Refusal::Malformed);
    }
    for root in &manifest.roots {
        for descriptor in &root.recorded_components {
            fold.require_component(descriptor, root.schema_carrier_digest)?;
        }
    }
    for empty in &manifest.retained_empty_components {
        fold.require_component(&empty.descriptor, empty.schema_carrier_digest)?;
    }
    Ok(())
}
impl Fold {
    fn record(
        &mut self,
        branch: &BranchId,
        state: Axes,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        self.observations
            .insert_cloned_native_for_request(
                &ObservationKey::of_axes(branch, &state),
                &state,
                *request,
            )
            .map_err(map)?;
        self.current
            .insert_cloned_native_for_request(branch, &state, *request)
            .map_err(map)
    }
    fn install(
        &mut self,
        initial: &InitialInputsImage,
        position: usize,
        transition: usize,
        installed: &mut bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let Some(InitialInputEntry::Installation {
            before,
            after,
            follows_transition,
        }) = initial.entries.get(1)
        else {
            return Ok(());
        };
        if *installed || follows_transition.0 != transition as u64 {
            return Ok(());
        }
        let state = self.current.get(&initial.main).ok_or(Refusal::Malformed)?;
        if position != 0 || state.removed || !state.matches_empty(before) {
            return Err(Refusal::Malformed);
        }
        self.record(&initial.main, Axes::empty(after), request)?;
        *installed = true;
        Ok(())
    }
    fn require_component(
        &self,
        descriptor: &worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor,
        schema: [u8; 32],
    ) -> Result<(), Refusal> {
        let state = self
            .observations
            .get(&ObservationKey::of_descriptor(descriptor))
            .ok_or(Refusal::Malformed)?;
        if state.truth != descriptor.truth_version()
            || state.schema != schema
            || !state.matches_target(descriptor.reference().target())
        {
            return Err(Refusal::Malformed);
        }
        Ok(())
    }
}
fn map(cause: worth_execution::ExecutionOrderedMapDenial) -> Refusal {
    Refusal::Native(contracts::durability::data::recovery_stop::map_stop(cause))
}
