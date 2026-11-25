//! The guard's decisions that no public edit reaches (#277 step 4). Native
//! refuses a head round that leaves an orbit open or unanchored
//! (`InvalidPermutation`), so no successor can meet an orbit its chain broke.
//! No handler can hold the identity of a record that is not live and not its
//! chain's. The arms for those states are defensive, and are pinned here.
use super::*;
use worth_relational::facade::identity::PartitionId;

fn record(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

/// A chain by its effects. Every liveness read is logged.
#[derive(Default)]
struct Chain {
    covered: Vec<EntityId>,
    deleted: Vec<EntityId>,
    created: Vec<EntityId>,
    lists: Vec<(EntityId, Side)>,
    reads: Vec<(EntityId, Image)>,
}

impl ChainEffects for Chain {
    type Stop = &'static str;
    fn touches(&self, record: EntityId, side: Side) -> bool {
        self.lists.contains(&(record, side))
    }
    fn covers(&self, record: EntityId) -> bool {
        self.covered.contains(&record)
    }
    fn live(&mut self, record: EntityId, image: Image) -> Result<bool, &'static str> {
        self.reads.push((record, image));
        Ok(match image {
            Image::Current => !self.deleted.contains(&record),
            Image::Head => !self.created.contains(&record),
        })
    }
}

/// The break is not further along: the member's walk must not be asked.
fn no_walk(_: &mut Chain) -> Result<Option<Mark>, &'static str> {
    panic!("the member's walk was not due")
}

#[test]
fn an_orbit_at_a_member_its_chain_deleted_or_created_is_marked_at_the_record() {
    // No list of the chain's is on the member: only its liveness tells. (A
    // public delete also retires the member's relations, so the court for
    // this arm is marked by the next arm too.)
    for failure in [Kind::InvalidOrbit, Kind::EndpointUnavailable] {
        let mut deleted = Chain {
            covered: vec![record(1)],
            deleted: vec![record(1)],
            ..Chain::default()
        };
        let mut created = Chain {
            covered: vec![record(1)],
            created: vec![record(1)],
            ..Chain::default()
        };
        for chain in [&mut deleted, &mut created] {
            assert_eq!(
                failed_orbit(chain, failure, record(1), no_walk),
                Ok(Some(Mark::Record(record(1)))),
                "{failure:?}"
            );
        }
    }
}

#[test]
fn an_orbit_that_fails_at_an_edge_its_chain_changed_is_marked_at_the_member() {
    for side in [Side::Outgoing, Side::Incoming] {
        for failure in [Kind::InvalidOrbit, Kind::EndpointUnavailable] {
            let mut chain = Chain {
                covered: vec![record(1)],
                lists: vec![(record(1), side)],
                ..Chain::default()
            };
            assert_eq!(
                failed_orbit(&mut chain, failure, record(1), no_walk),
                Ok(Some(Mark::List(record(1)))),
                "{side:?} {failure:?}"
            );
        }
    }
}

#[test]
fn an_orbit_broken_further_along_is_marked_where_its_walk_meets_the_chain() {
    // The member is live in both images and no edge of the chain's is on it.
    let mut chain = Chain {
        covered: vec![record(1)],
        lists: vec![(record(3), Side::Incoming)],
        ..Chain::default()
    };
    // The member's walk runs clean; the chain's edge enters its third record.
    let marked = failed_orbit(&mut chain, Kind::InvalidOrbit, record(1), |chain| {
        Ok(entered(chain, [record(1), record(2), record(3)]))
    });
    assert_eq!(marked, Ok(Some(Mark::List(record(3)))));
    // The walk's own guard marked the view and denied: nothing more to mark.
    assert_eq!(
        failed_orbit(&mut chain, Kind::InvalidOrbit, record(1), |_| Ok(None)),
        Ok(None)
    );
    // A read stop inside the walk is the round's stop.
    assert_eq!(
        failed_orbit(&mut chain, Kind::InvalidOrbit, record(1), |_| Err("stop")),
        Err("stop")
    );
    // An unavailable member has no walk to run.
    assert_eq!(
        failed_orbit(&mut chain, Kind::EndpointUnavailable, record(1), no_walk),
        Ok(None)
    );
}

#[test]
fn an_invalid_orbit_its_chain_did_not_touch_is_not_marked() {
    // The chain patched the member and relinked a list off the orbit.
    let mut chain = Chain {
        covered: vec![record(1)],
        lists: vec![(record(9), Side::Outgoing), (record(9), Side::Incoming)],
        ..Chain::default()
    };
    let marked = failed_orbit(&mut chain, Kind::InvalidOrbit, record(1), |chain| {
        Ok(entered(chain, [record(1), record(2)]))
    });
    assert_eq!(marked, Ok(None));
    // A member the chain never wrote is not read.
    let mut chain = Chain::default();
    assert_eq!(
        failed_orbit(&mut chain, Kind::InvalidOrbit, record(2), |_| Ok(None)),
        Ok(None)
    );
    assert!(chain.reads.is_empty());
}

#[test]
fn an_orbit_that_fails_for_another_reason_is_never_tested() {
    for failure in [
        Kind::WorkBudgetExceeded,
        Kind::NotOrbitRelation,
        Kind::AggregateDefinitionUnavailable,
        Kind::ForeignIdentity,
    ] {
        // Every arm would mark this member.
        let mut chain = Chain {
            covered: vec![record(1)],
            deleted: vec![record(1)],
            lists: vec![(record(1), Side::Outgoing)],
            ..Chain::default()
        };
        assert_eq!(
            failed_orbit(&mut chain, failure, record(1), no_walk),
            Ok(None),
            "{failure:?}"
        );
        assert!(chain.reads.is_empty(), "{failure:?}");
    }
}

#[test]
fn a_failed_walk_marks_the_named_record_its_chain_deleted_or_a_root_it_created() {
    // Native named record 5 not live. The chain wrote it, so it deleted it:
    // marked with no further read, ahead of any root.
    let mut chain = Chain {
        covered: vec![record(4), record(5)],
        created: vec![record(4)],
        ..Chain::default()
    };
    assert_eq!(
        failed_walk(&mut chain, Some(record(5)), &[record(4)]),
        Ok(Some(Mark::Record(record(5))))
    );
    assert!(chain.reads.is_empty());
    // The named record is not the chain's (another publication deleted it).
    // Root 1 the chain only patched; root 2 it never wrote. Nothing is the
    // chain's effect, and only the covered root is read, at the head.
    let mut chain = Chain {
        covered: vec![record(1)],
        ..Chain::default()
    };
    assert_eq!(
        failed_walk(&mut chain, Some(record(5)), &[record(1), record(2)]),
        Ok(None)
    );
    assert_eq!(chain.reads, [(record(1), Image::Head)]);
    // A root the chain created, behind one it only patched: no walk from it
    // is the view's, whichever record failed and whether Native named one.
    for endpoint in [None, Some(record(5))] {
        let mut chain = Chain {
            covered: vec![record(1), record(4)],
            created: vec![record(4)],
            ..Chain::default()
        };
        assert_eq!(
            failed_walk(&mut chain, endpoint, &[record(1), record(4)]),
            Ok(Some(Mark::Record(record(4)))),
            "{endpoint:?}"
        );
    }
}
