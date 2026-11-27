//! `CHK24` commit-index pages of one complete capture.
//!
//! Pages are built once the regions installed the selected roots, so the
//! inline heads match them. Prior prefix pages the destination already holds
//! are carried; every other page is written with its exact bytes.
use crate::durability::data::*;
use contracts::durability::data::native_checkpoint_capture_denial::{
    capture_stop, RelationalNativeCheckpointCaptureDenial as Denial,
};
use std::collections::BTreeSet;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry as Entry;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;

pub(super) fn install(
    draft: &mut RelationalCheckpointManifestDraft,
    rows: &[Entry],
    prior: Option<&worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let RelationalCheckpointArtifactPolicy::ExternalMembers {
        current_members, ..
    } = placement;
    let held = current_members.iter().copied().collect::<BTreeSet<_>>();
    let roots = draft.roots.iter().map(|root| root.commit_id).collect();
    use crate::durability::log::native_checkpoint_codec::commit_pages;
    let start = commit_pages::carry_start(prior.map(|prior| prior.descriptor().as_ref()));
    let rows = usize::try_from(start - 1)
        .ok()
        .and_then(|skip| rows.get(skip..))
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let (index, pages) = commit_pages::page_commit_rows(
        prior,
        rows,
        &roots,
        commit_pages::CommitPageCarry::Held(&held),
    )
    .map_err(Denial::Durability)?;
    for member in pages {
        let locator = member.locator();
        if members.get(&locator).is_none() {
            members
                .insert_cloned_native_for_request(&locator, &member, *request)
                .map_err(|cause| {
                    capture_stop(contracts::durability::data::recovery_stop::map_stop(cause))
                })?;
        }
    }
    draft.commits = index;
    Ok(())
}

/// Seals the frozen branch naming and group state into `CHK24` pages and adds
/// their members to `members`, carrying pages the destination already holds.
pub(super) fn install_naming(
    draft: &mut RelationalCheckpointManifestDraft,
    frozen: &worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::FrozenLineageNaming,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let RelationalCheckpointArtifactPolicy::ExternalMembers {
        current_members, ..
    } = placement;
    let held = current_members.iter().copied().collect::<BTreeSet<_>>();
    use crate::durability::log::native_checkpoint_codec::naming_pages;
    let required: Vec<_> = draft
        .branch_cells
        .iter()
        .filter(|cell| {
            matches!(
                cell.observation.target(),
                worth_foundational::FoundationalBranchTarget::Basis(_)
            )
        })
        .map(|cell| cell.branch_id.clone())
        .collect();
    let (naming, pages) = naming_pages::seal_naming_state(
        frozen,
        &required,
        naming_pages::NamingPageCarry::Held(&held),
    )
    .map_err(Denial::Durability)?;
    for member in pages {
        let locator = member.locator();
        if members.get(&locator).is_none() {
            members
                .insert_cloned_native_for_request(&locator, &member, *request)
                .map_err(|cause| {
                    capture_stop(contracts::durability::data::recovery_stop::map_stop(cause))
                })?;
        }
    }
    draft.naming = naming;
    Ok(())
}
