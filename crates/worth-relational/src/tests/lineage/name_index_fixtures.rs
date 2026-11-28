//! Shared helpers for the derived name index courts.

use crate::facade::history::{BranchId, CommitId};
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameKey,
    PersistentNameResolution, PersistentNameResolutionDenial, PersistentNameResolutionMetrics,
    PersistentNameSelection, PersistentNameTarget,
};

use super::name_group_world::{member, GroupWorld};
use super::name_world::{side_of, Keep, LineageWorld, Name, Subject};

pub(super) type Answer = Result<PersistentNameResolution, PersistentNameResolutionDenial>;

pub(super) fn main_branch() -> BranchId {
    BranchId("main".to_owned())
}

pub(super) fn rebuild(log: &[PersistentNameCommittedBatch]) -> PersistentNameBranchIndex {
    PersistentNameBranchIndex::rebuild_from_log(main_branch(), None, log)
        .expect("canonical committed log rebuilds the index")
}

pub(super) fn resolve(
    index: &PersistentNameBranchIndex,
    held: PersistentNameKey,
    basis: u64,
    selection: &PersistentNameSelection,
) -> Answer {
    index.resolve(held, CommitId(basis), selection).result
}

pub(super) fn metrics(
    index: &PersistentNameBranchIndex,
    held: PersistentNameKey,
    basis: u64,
) -> PersistentNameResolutionMetrics {
    index
        .resolve(held, CommitId(basis), &PersistentNameSelection::Exact)
        .metrics
}

pub(super) fn exact() -> PersistentNameSelection {
    PersistentNameSelection::Exact
}

pub(super) fn all() -> PersistentNameSelection {
    PersistentNameSelection::AllDescendants
}

pub(super) fn bounded(carriers: &[Name]) -> PersistentNameSelection {
    PersistentNameSelection::BoundedByCarriers {
        carriers: carriers.iter().copied().collect(),
    }
}

pub(super) fn side(label: &str) -> PersistentNameSelection {
    PersistentNameSelection::KeepSide {
        discriminator: side_of(label),
    }
}

/// The key an `Exact` answer resolved to.
pub(super) fn exact_key(answer: &Answer) -> PersistentNameKey {
    match answer {
        Ok(PersistentNameResolution::Exact(target)) => target.key(),
        other => panic!("expected an exact answer, found {other:?}"),
    }
}

pub(super) fn plural_keys(answer: &Answer) -> Vec<PersistentNameKey> {
    match answer {
        Ok(PersistentNameResolution::Plural(targets)) => {
            targets.iter().map(PersistentNameTarget::key).collect()
        }
        other => panic!("expected a plural answer, found {other:?}"),
    }
}

pub(super) fn ambiguous(mut candidates: Vec<PersistentNameKey>) -> Answer {
    candidates.sort_unstable();
    Err(PersistentNameResolutionDenial::Ambiguous { candidates })
}

pub(super) fn no_match(mut live_candidates: Vec<PersistentNameKey>) -> Answer {
    live_candidates.sort_unstable();
    Err(PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates })
}

pub(super) fn retired(at: u64) -> Answer {
    Ok(PersistentNameResolution::Retired {
        at_commit: CommitId(at),
    })
}

pub(super) fn unknown(name: PersistentNameKey) -> Answer {
    Err(PersistentNameResolutionDenial::UnknownName { name })
}

pub(super) fn probes(
    index_probes: usize,
    event_visits: usize,
    edge_visits: usize,
) -> PersistentNameResolutionMetrics {
    PersistentNameResolutionMetrics {
        index_probes,
        edge_visits,
        event_visits,
    }
}

/// A branch mixing entity, relation, and group naming in shared commits.
pub(super) fn mixed_history() -> (
    Vec<PersistentNameCommittedBatch>,
    PersistentNameBranchIndex,
    Vec<Name>,
) {
    let mut world = LineageWorld::new();
    let mut groups = GroupWorld::new();
    let mut incremental = PersistentNameBranchIndex::new(main_branch(), None);
    let mut log = Vec::new();
    let mut push = |batch: PersistentNameCommittedBatch, log: &mut Vec<_>| {
        incremental.apply(&batch).expect("committed batch ingests");
        log.push(batch);
    };
    push(world.log()[0].clone(), &mut log);
    let face = world.create(Subject::Entity, "face");
    let seam = world.create(Subject::Relation, "seam");
    let cutter = world.create(Subject::Entity, "cutter");
    let batch = world.commit();
    let born = groups.step(&[&[1, 2, 3, 4]]).unwrap();
    push(batch.with_published_group_transitions(born), &mut log);
    let cutter_name = world.name_of(&cutter);
    let face_pieces = world.split(world.name_of(&face), &["left", "right"]);
    world.bind_carriers(&face_pieces[0], &[cutter_name]);
    let seam_pieces = world.split(world.name_of(&seam), &["top", "bottom"]);
    let batch = world.commit();
    let split = groups.step(&[&[2], &[3, 4]]).unwrap();
    push(batch.with_published_group_transitions(split), &mut log);
    world.retire(world.name_of(&face_pieces[1]));
    world.merge(
        &[
            world.name_of(&seam_pieces[0]),
            world.name_of(&seam_pieces[1]),
        ],
        Keep::Recreated,
    );
    let batch = world.commit();
    let revived = groups.step(&[&[1], &[2, 3, 4]]).unwrap();
    push(batch.with_published_group_transitions(revived), &mut log);
    world.merge(
        &[world.name_of(&face_pieces[0]), cutter_name],
        Keep::MintNew,
    );
    let batch = world.commit();
    push(batch, &mut log);
    (log, incremental, vec![cutter_name, member(2), member(3)])
}

pub(super) fn selections(carriers: &[Name]) -> Vec<PersistentNameSelection> {
    let mut selections = vec![exact(), all(), side("left"), side("top"), side("absent")];
    selections.extend(carriers.iter().map(|carrier| bounded(&[*carrier])));
    selections.push(bounded(carriers));
    selections
}
