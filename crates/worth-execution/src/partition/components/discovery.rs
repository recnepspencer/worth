//! Interleaved discovery of all but the largest piece of a forest cut.
use super::*;
use std::collections::VecDeque;

pub(super) trait Discovery {
    type Error;
    /// Admit candidate storage before inserting a member, then account the visit.
    fn member(&mut self) -> Result<(), Self::Error>;
    fn edge(&mut self) -> Result<(), Self::Error>;
}

pub(super) struct Offline;
impl Discovery for Offline {
    type Error = std::convert::Infallible;
    fn member(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn edge(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct Search<'graph> {
    found: BTreeSet<PartitionItemId>,
    pending: VecDeque<PartitionItemId>,
    incidence: Option<std::collections::btree_set::Iter<'graph, PartitionItemId>>,
    source: Option<PartitionItemId>,
}
impl<'graph> Search<'graph> {
    fn start<D: Discovery>(seed: PartitionItemId, charge: &mut D) -> Result<Self, D::Error> {
        charge.member()?;
        Ok(Self {
            found: BTreeSet::from([seed]),
            pending: VecDeque::from([seed]),
            incidence: None,
            source: None,
        })
    }
}

pub(super) fn smaller_pieces<D: Discovery>(
    graph: &ComponentPartitioner,
    removed_item: Option<PartitionItemId>,
    removed_edge: Option<(PartitionItemId, PartitionItemId)>,
    seeds: &[PartitionItemId],
    charge: &mut D,
    work: &mut PartitionWork,
) -> Result<Vec<BTreeSet<PartitionItemId>>, D::Error> {
    // A search retains its current source so one excluded incidence is skipped
    // without constructing filtered copies of either forest half.
    let mut searches = Vec::new();
    for &seed in seeds {
        searches.push(Search::start(seed, charge)?);
        work.members_visited += 1;
    }
    let mut active: Vec<_> = (0..searches.len()).collect();
    let mut completed = Vec::new();
    let mut cursor = 0;
    while active.len() > 1 {
        let index = active[cursor];
        if searches[index].step(graph, removed_item, removed_edge, charge, work)? {
            completed.push(std::mem::take(&mut searches[index].found));
            active.swap_remove(cursor);
        } else {
            cursor += 1;
        }
        cursor %= active.len();
    }
    Ok(completed)
}

impl<'graph> Search<'graph> {
    fn step<D: Discovery>(
        &mut self,
        graph: &'graph ComponentPartitioner,
        removed_item: Option<PartitionItemId>,
        removed_edge: Option<(PartitionItemId, PartitionItemId)>,
        charge: &mut D,
        work: &mut PartitionWork,
    ) -> Result<bool, D::Error> {
        if let Some(edges) = &mut self.incidence {
            if let Some(&next) = edges.next() {
                charge.edge()?;
                work.edges_visited += 1;
                let excluded = removed_edge.is_some_and(|(a, b)| {
                    (self.source == Some(a) && next == b) || (self.source == Some(b) && next == a)
                });
                if !excluded && Some(next) != removed_item && !self.found.contains(&next) {
                    charge.member()?;
                    work.members_visited += 1;
                    self.found.insert(next);
                    self.pending.push_back(next);
                }
                return Ok(false);
            }
            self.incidence = None;
        }
        let Some(item) = self.pending.pop_front() else {
            return Ok(true);
        };
        self.source = Some(item);
        self.incidence = Some(graph.forest[&item].iter());
        Ok(false)
    }
}
