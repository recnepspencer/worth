//! A successor round's walk and orbit guard (#277). Native walks, and reads an
//! orbit's aggregate, at the round's current image. The round's view serves
//! its own chain's records and relations as they were at the chain head, and
//! it serves no walk in that view (public #315). So a walk that crosses a list
//! the chain relinked, or reaches a record the chain created, never answers
//! the round. A walk or orbit that fails where the chain created or deleted a
//! record, or changed an edge, failed on the chain's effect. In each case the
//! view is marked incoherent at that record, and the round's untyped stop
//! becomes `PriorEffectRead` (`successor_prepare_stop`). A walk that passes is
//! the same walk in both images. The guard only adds marks: a failure on a
//! graph the chain did not touch, and every typed stop, stays what it was.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::own_outputs::{
    live_in, OwnOutputOverlay,
};
use worth_relational::facade::runtime::{
    RelationalAdjacencyDirection as Side, VisibilityProjectionView,
};

/// Where the guard marks the view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mark {
    /// A record the chain created or deleted.
    Record(EntityId),
    /// A record whose list of the walked kind holds a relation of the chain's.
    List(EntityId),
}

/// One of the two images a successor round's view is served from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Image {
    /// The round's own candidate: what Native walks.
    Current,
    /// The chain head's pre-pin: what the view serves the chain's records at.
    Head,
}

/// What the guard asks about the round's chain.
trait ChainEffects {
    /// A read stop: the round's own stop, never a mark.
    type Stop;
    /// Whether a relation of the chain's lies on `record`'s list of the
    /// walked kind on `side`.
    fn touches(&self, record: EntityId, side: Side) -> bool;
    /// Whether the chain wrote `record`.
    fn covers(&self, record: EntityId) -> bool;
    /// Whether `record` is live in `image`. One charged point read.
    fn live(&mut self, record: EntityId, image: Image) -> Result<bool, Self::Stop>;
}

/// Whether the chain created `record`. Only a covered record is read.
fn created<C: ChainEffects>(chain: &mut C, record: EntityId) -> Result<bool, C::Stop> {
    Ok(chain.covers(record) && !chain.live(record, Image::Head)?)
}

/// Whether the chain created or deleted `record`. A record the chain only
/// patched is live in both images. Only a covered record is read.
fn made_or_removed<C: ChainEffects>(chain: &mut C, record: EntityId) -> Result<bool, C::Stop> {
    Ok(chain.covers(record)
        && (!chain.live(record, Image::Current)? || !chain.live(record, Image::Head)?))
}

/// A record Native's walk visited: its list in the walk's direction is the
/// chain's, or the chain created it.
fn visited<C: ChainEffects>(
    chain: &mut C,
    record: EntityId,
    side: Side,
) -> Result<Option<Mark>, C::Stop> {
    if chain.touches(record, side) {
        return Ok(Some(Mark::List(record)));
    }
    Ok(created(chain, record)?.then_some(Mark::Record(record)))
}

/// Native's walk failed on an unavailable record and named it `endpoint`.
/// Native found that record not live in the current image, so if the chain
/// wrote it, the chain deleted it. Otherwise the first seed root the chain
/// created: no walk from it is the view's. A live root the chain only
/// patched, and a record another publication deleted, are not marked.
fn failed_walk<C: ChainEffects>(
    chain: &mut C,
    endpoint: Option<EntityId>,
    roots: &[EntityId],
) -> Result<Option<Mark>, C::Stop> {
    if let Some(record) = endpoint.filter(|&record| chain.covers(record)) {
        return Ok(Some(Mark::Record(record)));
    }
    for &root in roots {
        if created(chain, root)? {
            return Ok(Some(Mark::Record(root)));
        }
    }
    Ok(None)
}

/// The first record of a clean forward walk whose incoming list is the
/// chain's. An orbit's aggregate reads both lists of each member.
fn entered<C: ChainEffects>(chain: &C, walked: impl IntoIterator<Item = EntityId>) -> Option<Mark> {
    walked
        .into_iter()
        .find(|record| chain.touches(*record, Side::Incoming))
        .map(Mark::List)
}

/// An orbit whose aggregate read failed `failure` before any walk. Only an
/// unavailable member or an invalid orbit can be the chain's effect. In
/// order: the member is one the chain created or deleted; an edge at the
/// member is the chain's; or, for an invalid orbit, the break is `further`
/// along the member's forward walk.
fn failed_orbit<C: ChainEffects>(
    chain: &mut C,
    failure: Kind,
    member: EntityId,
    further: impl FnOnce(&mut C) -> Result<Option<Mark>, C::Stop>,
) -> Result<Option<Mark>, C::Stop> {
    if !matches!(failure, Kind::EndpointUnavailable | Kind::InvalidOrbit) {
        return Ok(None);
    }
    if made_or_removed(chain, member)? {
        return Ok(Some(Mark::Record(member)));
    }
    if chain.touches(member, Side::Outgoing) || chain.touches(member, Side::Incoming) {
        return Ok(Some(Mark::List(member)));
    }
    if failure != Kind::InvalidOrbit {
        return Ok(None);
    }
    further(chain)
}

/// A successor round's chain, as one read of one relation kind sees it.
struct RoundChain<'x, 'a, 'r> {
    read: &'x mut NativeRead<'a, 'r>,
    own: &'r OwnOutputOverlay,
    current: VisibilityProjectionView,
    kind: KindId,
    subject: &'x str,
}

impl<'x, 'a, 'r> RoundChain<'x, 'a, 'r> {
    /// `None` outside a successor round: there is no chain to guard against.
    fn of(read: &'x mut NativeRead<'a, 'r>, kind: KindId, subject: &'x str) -> Option<Self> {
        let own = read.native.own_outputs()?;
        let current = read.native.projection(read.snapshot)?;
        Some(Self {
            read,
            own,
            current,
            kind,
            subject,
        })
    }

    fn mark(&self, mark: Mark) {
        match mark {
            Mark::Record(record) => self.own.mark_incoherent(record),
            Mark::List(record) => self.own.mark_incoherent_relation(record),
        }
    }
}

impl ChainEffects for RoundChain<'_, '_, '_> {
    type Stop = Denial;
    fn touches(&self, record: EntityId, side: Side) -> bool {
        self.own.touches_adjacency(record, self.kind, side)
    }
    fn covers(&self, record: EntityId) -> bool {
        self.own.covers(record)
    }
    fn live(&mut self, record: EntityId, image: Image) -> Result<bool, Denial> {
        let live = match image {
            Image::Current => live_in(&self.current, self.read.execution, record),
            Image::Head => self.own.live_at_head(self.read.execution, record),
        };
        live.map_err(|cause| self.read.walk_stop(cause, self.subject))
    }
}

impl NativeRead<'_, '_> {
    /// Native's walk succeeded at the current image. In a successor round it
    /// answers only if it is the walk the round's view holds.
    pub(super) fn guard_walk(
        &mut self,
        walk: &StructuralWalkObservation,
        kind: KindId,
        direction: RelationDirection,
        relation: &str,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        let Some(mut chain) = RoundChain::of(self, kind, relation) else {
            return Ok(());
        };
        let side = match direction {
            RelationDirection::Forward => Side::Outgoing,
            RelationDirection::Reverse => Side::Incoming,
        };
        for visit in walk.native().visits() {
            chain.read.walk_checkpoint(context, relation)?;
            if let Some(mark) = visited(&mut chain, visit.entity.entity_id, side)? {
                chain.mark(mark);
                return Err(Denial::new(Kind::EndpointUnavailable, relation));
            }
        }
        Ok(())
    }

    /// Native's walk failed on the unavailable record `endpoint`. In a
    /// successor round that record is marked if the chain deleted it, or
    /// else a seed root the chain created; the failure stays.
    pub(super) fn guard_failed_walk(
        &mut self,
        endpoint: Option<EntityId>,
        roots: &[EntityId],
        kind: KindId,
        relation: &str,
    ) -> Result<(), Denial> {
        let Some(mut chain) = RoundChain::of(self, kind, relation) else {
            return Ok(());
        };
        if let Some(mark) = failed_walk(&mut chain, endpoint, roots)? {
            chain.mark(mark);
        }
        Ok(())
    }

    /// An orbit's aggregate read failed with `denial` before its walk. In a
    /// successor round a failure the chain caused is marked. The answer is
    /// still `denial`, unless a read of the guard's own stops the round.
    pub fn failed_orbit(
        &mut self,
        denial: Denial,
        relation: &str,
        entity: &str,
        member: NativeIdentity<'_>,
        kind: KindId,
    ) -> Denial {
        let Some(mut chain) = RoundChain::of(self, kind, relation) else {
            return denial;
        };
        let marked = failed_orbit(&mut chain, denial.kind(), member.entity_id, |chain| {
            let walk = chain.read.closure(
                relation,
                entity,
                RelationDirection::Forward,
                &OneRoot(member),
            );
            match walk {
                Ok(walk) => Ok(entered(
                    chain,
                    walk.native()
                        .visits()
                        .iter()
                        .map(|visit| visit.entity.entity_id),
                )),
                Err(stop)
                    if matches!(
                        stop.kind(),
                        Kind::ReadExecution(_) | Kind::WorkBudgetExceeded
                    ) =>
                {
                    Err(stop)
                }
                // The walk's own guard marked the view, or the walk broke
                // where the chain changed nothing.
                Err(_) => Ok(None),
            }
        });
        match marked {
            Ok(Some(mark)) => {
                chain.mark(mark);
                denial
            }
            Ok(None) => denial,
            Err(stop) => stop,
        }
    }
}

#[cfg(test)]
mod tests;
