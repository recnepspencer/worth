//! Seek directly to a finite visible prefix; private reservations never enter page hints.
use super::super::paging::{
    WorthQueryHistoryPage as Page, WorthQueryHistoryPageCursor as Cursor,
    WorthQueryHistoryPageDenial as Failure, WorthQueryHistoryPageEntry as Entry,
    WorthQueryHistoryPageLimits as Limits, WorthQueryHistoryPageRow as Row,
    WorthQueryHistoryParentBoundary as Parent,
};
use super::super::{
    WorthQueryHistoryRedoChoice as Choice, WorthQueryHistoryRedoChoices as Choices,
};
use super::*;
use std::ops::Bound::{Excluded, Included};
use worth_execution::WorkCeilingDenial;
#[path = "paging/children.rs"]
mod children;
#[path = "paging/entries.rs"]
mod entries;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph) fn page(
        &self,
        context: super::super::WorthQueryHistoryDescriptionContext,
        execution: ExecutionRequest<'_, '_>,
        limits: Limits,
        after: Option<&Cursor>,
    ) -> Result<Page, Failure> {
        PageSource {
            context,
            incarnation: self.incarnation,
            current: self.current,
            view_generation: self.view_generation,
            completed_end: self.completed_end,
            nodes: &self.nodes,
            retained: &self.retained,
            children: &self.children,
            parent: None,
            observation: self.observation.as_ref(),
        }
        .page(execution, limits, after)
        .map(|(page, _)| page)
    }

    pub(in crate::domain_computation::primary_graph) fn redo_choices(
        &self,
        context: super::super::WorthQueryHistoryDescriptionContext,
        execution: ExecutionRequest<'_, '_>,
        limits: Limits,
        after: Option<&Cursor>,
    ) -> Result<Choices, Failure> {
        let (children, choice) = PageSource {
            context,
            incarnation: self.incarnation,
            current: self.current,
            view_generation: self.view_generation,
            completed_end: self.completed_end,
            nodes: &self.nodes,
            retained: &self.retained,
            children: &self.children,
            parent: Some(self.current),
            observation: self.observation.as_ref(),
        }
        .page(execution, limits, after)?;
        Ok(Choices {
            children,
            choice: choice.expect("child pages classify their completed alternatives"),
        })
    }
}

struct PageSource<'a> {
    context: super::super::WorthQueryHistoryDescriptionContext,
    incarnation: DocumentIncarnation,
    current: WorthQueryHistoryNodeId,
    view_generation: u64,
    completed_end: u64,
    nodes: &'a BTreeMap<u64, HistoryNode>,
    retained: &'a BTreeMap<u64, RetainedHistorySource>,
    children: &'a BTreeMap<(u64, u64), ChildLink>,
    parent: Option<WorthQueryHistoryNodeId>,
    observation: Option<&'a EnrollmentSink>,
}

impl PageSource<'_> {
    fn page(
        self,
        execution: ExecutionRequest<'_, '_>,
        limits: Limits,
        after: Option<&Cursor>,
    ) -> Result<(Page, Option<Choice>), Failure> {
        if let Some(cursor) = after {
            if cursor.document != self.incarnation {
                return Err(Failure::ForeignCursor);
            }
            if cursor.generation != self.view_generation {
                return Err(Failure::StaleCursor);
            }
            if cursor.parent != self.parent {
                return Err(Failure::CursorScopeMismatch);
            }
        }
        let header = if self.parent.is_some() {
            size_of::<Choices>()
        } else {
            size_of::<Page>()
        };
        if header > limits.maximum_owned_bytes.get() {
            return Err(Failure::StorageLimitExceeded);
        }
        let mut prepared = None;
        let outcome = execution.run_native_with_checkpoint(|context| {
            prepared = Some(self.assemble(context, execution, limits, after, header));
        });
        match (outcome, prepared) {
            (Ok(_), Some(result)) => result,
            (
                Err(WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::NestedStopped)),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(Failure::Request(cause)),
            (Ok(_), None) => unreachable!("completed page assembly callback"),
        }
    }

    fn assemble(
        self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        execution: ExecutionRequest<'_, '_>,
        limits: Limits,
        after: Option<&Cursor>,
        header: usize,
    ) -> Result<(Page, Option<Choice>), Failure> {
        let (choice, end) = match self.parent {
            None => (
                None,
                self.retained
                    .range(..self.completed_end)
                    .next_back()
                    .map_or(0, |(ordinal, _)| ordinal + 1),
            ),
            Some(parent) => {
                let (choice, end) = self.child_summary(context, parent)?;
                (Some(choice), end)
            }
        };
        let start = after.map_or(0, |cursor| cursor.after + 1);
        let remaining = usize::try_from(end.saturating_sub(start)).unwrap_or(usize::MAX);
        let capacity = limits
            .maximum_entries
            .get()
            .min(limits.maximum_examined_entries.get())
            .min(remaining);
        let mut bytes = capacity
            .checked_mul(size_of::<Row>())
            .and_then(|n| n.checked_add(header))
            .ok_or(Failure::SizeOverflow)?;
        if bytes > limits.maximum_owned_bytes.get() {
            return Err(Failure::StorageLimitExceeded);
        }
        let mut backing = execution
            .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Failure::SizeOverflow)?)
            .map_err(Failure::Request)?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(capacity)
            .map_err(Failure::Allocation)?;
        // std's two range descents have an explicit logarithmic granule.
        let seek = seek_work(if self.parent.is_some() {
            self.children.len()
        } else {
            self.retained.len()
        });
        checkpoint(context, seek)?;
        let mut examined = 0;
        let mut last = None;
        let mut entries = entries::Entries::new(&self, start, end);
        while examined < limits.maximum_examined_entries.get() && rows.len() < capacity {
            checkpoint(context, 1)?;
            let Some((ordinal, node)) = entries.next(context, self.observation)? else {
                break;
            };
            examined += 1;
            last = Some(ordinal);
            let NodeBinding::Completed(node) = &node.binding else {
                continue;
            };
            let next_bytes = bytes
                .checked_add(node.step.label().map_or(0, str::len))
                .ok_or(Failure::SizeOverflow)?;
            if next_bytes > limits.maximum_owned_bytes.get() {
                return Err(Failure::StorageLimitExceeded);
            }
            if let Some(ticket) = &mut backing {
                ticket
                    .resize(u64::try_from(next_bytes).map_err(|_| Failure::SizeOverflow)?)
                    .map_err(|cause| {
                        Failure::Request(WorkCeilingDenial::Admission(
                            worth_execution::LeaseDenial::MemoryExhausted(cause),
                        ))
                    })?;
            }
            bytes = next_bytes;
            checkpoint(context, seek_work(self.retained.len()) + 1)?;
            rows.push(Row {
                step: Entry {
                    id: node.step.id(),
                    parent: match node.step.parent() {
                        None => Parent::Root,
                        Some(parent)
                            if self
                                .retained
                                .get(&parent.ordinal)
                                .is_some_and(|source| source.source.is_some()) =>
                        {
                            Parent::Retained(parent)
                        }
                        Some(_) => Parent::Truncated,
                    },
                    label: node.step.label.clone(),
                    endpoint: node.step.endpoint.clone(),
                },
            });
        }
        let next = last
            .filter(|ordinal| ordinal + 1 < end)
            .map(|after| Cursor {
                document: self.incarnation,
                generation: self.view_generation,
                after,
                parent: self.parent,
            });
        Ok((
            Page {
                context: self.context.clone(),
                rows,
                current: self.current,
                next,
                examined,
                admitted_bytes: bytes,
                _backing: backing,
            },
            choice,
        ))
    }
}

fn seek_work(len: usize) -> u64 {
    2 * u64::from(usize::BITS - len.leading_zeros()) + 1
}

fn checkpoint(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    units: u64,
) -> Result<(), Failure> {
    (if context.has_accounting() {
        context.checkpoint(units)
    } else {
        context.poll()
    })
    .map_err(|cause| Failure::Request(WorkCeilingDenial::Stopped(cause)))
}
