//! Stack cursors visit each chunk/tree edge once, without flattening allocation.
use super::{Fact, FactPayload, FactStorage};

#[derive(Clone, Copy)]
struct Pending<'a> {
    payload: &'a FactPayload,
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct Cursor<'a> {
    pending: [Option<Pending<'a>>; usize::BITS as usize],
    depth: usize,
    leaf: std::slice::Iter<'a, Fact>,
    reverse: bool,
}
impl<'a> Cursor<'a> {
    fn new(payload: &'a FactPayload, reverse: bool) -> Self {
        let mut cursor = Self {
            pending: [None; usize::BITS as usize],
            depth: 0,
            leaf: [].iter(),
            reverse,
        };
        cursor.push(payload, 0, payload.len());
        cursor
    }
    fn push(&mut self, payload: &'a FactPayload, start: usize, end: usize) {
        if start == end {
            return;
        }
        assert!(
            self.depth < self.pending.len(),
            "balanced native fact tree depth"
        );
        self.pending[self.depth] = Some(Pending {
            payload,
            start,
            end,
        });
        self.depth += 1;
    }
    fn next(&mut self) -> Option<&'a Fact> {
        loop {
            if let Some(fact) = if self.reverse {
                self.leaf.next_back()
            } else {
                self.leaf.next()
            } {
                return Some(fact);
            }
            self.depth = self.depth.checked_sub(1)?;
            let Pending {
                payload,
                start,
                end,
            } = self.pending[self.depth].take().expect("pending subtree");
            let start = payload.start + start;
            let end = payload.start + end;
            match &payload.storage {
                FactStorage::Original(facts) | FactStorage::Decoded { facts, .. } => {
                    self.leaf = facts[start..end].iter();
                }
                FactStorage::Native(facts) => self.leaf = facts.elements()[start..end].iter(),
                FactStorage::Joined { children } => {
                    let left = children[0].len();
                    let right_start = start.saturating_sub(left);
                    let right_end = end.saturating_sub(left);
                    if self.reverse {
                        self.push(&children[0], start.min(left), end.min(left));
                        self.push(&children[1], right_start, right_end);
                    } else {
                        self.push(&children[1], right_start, right_end);
                        self.push(&children[0], start.min(left), end.min(left));
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct FactIter<'a> {
    front: Cursor<'a>,
    back: Cursor<'a>,
    remaining: usize,
}
impl<'a> FactIter<'a> {
    pub(super) fn new(payload: &'a FactPayload) -> Self {
        Self {
            front: Cursor::new(payload, false),
            back: Cursor::new(payload, true),
            remaining: payload.len(),
        }
    }
}
impl<'a> Iterator for FactIter<'a> {
    type Item = &'a Fact;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.front.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl DoubleEndedIterator for FactIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.back.next()
    }
}
impl ExactSizeIterator for FactIter<'_> {}
impl std::iter::FusedIterator for FactIter<'_> {}
