//! One ordinary node range or one parent's child range; no whole-tree filtering.
use super::*;
use std::collections::btree_map::Range;

pub(super) enum Entries<'a> {
    All {
        range: Range<'a, u64, RetainedHistorySource>,
        nodes: &'a BTreeMap<u64, HistoryNode>,
    },
    Children {
        range: Range<'a, (u64, u64), ChildLink>,
        nodes: &'a BTreeMap<u64, HistoryNode>,
    },
}

impl<'a> Entries<'a> {
    pub(super) fn new(source: &PageSource<'a>, start: u64, end: u64) -> Self {
        match source.parent {
            None => Self::All {
                range: source
                    .retained
                    .range((Included(start), Excluded(end.max(start)))),
                nodes: source.nodes,
            },
            Some(parent) => Self::Children {
                range: source.children.range((
                    Included((parent.ordinal, start)),
                    Excluded((parent.ordinal, end.max(start))),
                )),
                nodes: source.nodes,
            },
        }
    }

    pub(super) fn next(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        observation: Option<&EnrollmentSink>,
    ) -> Result<Option<(u64, &'a HistoryNode)>, Failure> {
        Ok(match self {
            Self::All { range, nodes } => {
                let Some((&ordinal, _source)) = range.next() else {
                    return Ok(None);
                };
                checkpoint(context, seek_work(nodes.len()) + 1)?;
                if let Some(observation) = observation {
                    observation.access(Access::Node);
                }
                Some((
                    ordinal,
                    nodes
                        .get(&ordinal)
                        .expect("retained source has canonical annotations"),
                ))
            }
            Self::Children { range, nodes } => {
                let Some(((_, ordinal), _)) = range.next() else {
                    return Ok(None);
                };
                checkpoint(context, seek_work(nodes.len()) + 1)?;
                if let Some(observation) = observation {
                    observation.access(Access::Child);
                    observation.access(Access::Node);
                }
                Some((
                    *ordinal,
                    nodes
                        .get(ordinal)
                        .expect("completed child index retains its journal node"),
                ))
            }
        })
    }
}
