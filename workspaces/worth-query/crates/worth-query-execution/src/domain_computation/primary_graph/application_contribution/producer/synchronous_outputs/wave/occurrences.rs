//! Close the finite typed occurrence frontier before preparing any supplier.
use super::*;
impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn quote_frontier(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        allowed: usize,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, SynchronousOutputFailure> {
        let denied = || {
            SynchronousOutputFailure(CoupledSourceReadStop::from(
                super::super::super::WorthQueryOutputDemandDenial::new(
                    super::super::super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    "required occurrence frontier",
                ),
            ))
        };
        let n = u64::try_from(self.tasks.len()).map_err(|_| denied())?;
        let edges = self
            .tasks
            .iter()
            .try_fold(0u64, |total, t| {
                total.checked_add(t.prerequisites.len() as u64)
            })
            .ok_or_else(denied)?;
        let width = n
            .checked_add(edges)
            .and_then(|v| v.checked_add(allowed as u64))
            .and_then(|v| v.checked_add(1))
            .ok_or_else(denied)?;
        let work = n
            .checked_add(1)
            .and_then(|v| v.checked_mul(width))
            .and_then(|v| v.checked_mul(8))
            .ok_or_else(denied)?;
        checkpoint(context.execution, "required occurrence frontier", work)?;
        let bytes = n
            .checked_mul((size_of::<usize>() + size_of::<bool>()) as u64)
            .ok_or_else(denied)?;
        context.execution.allocation_policy().reserve_memory(bytes).map_err(|cause| SynchronousOutputFailure(CoupledSourceReadStop::capture(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied("required occurrence frontier",cause))))
    }
    pub(super) fn blocked_frontier(&self, needed: &[bool]) -> SynchronousOutputFailure {
        use super::super::super::{
            WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
        };
        for task in self
            .tasks
            .iter()
            .enumerate()
            .filter(|(i, t)| needed[*i] && !t.done)
            .map(|(_, t)| t)
        {
            for parent in &task.prerequisites {
                let parent = &self.tasks[self.canonical(*parent)];
                if parent.done && parent.parent.is_none() {
                    if let Some(key) = &parent.key {
                        return SynchronousOutputFailure(CoupledSourceReadStop::from(Denial::new(
                            Kind::RequiredOutputParentUnavailable { source: key.scope },
                            task.subject,
                        )));
                    }
                }
            }
        }
        if let Some(task) = self
            .tasks
            .iter()
            .enumerate()
            .find(|(i, t)| needed[*i] && !t.done)
            .map(|(_, t)| t)
        {
            if let Some(key) = &task.key {
                return SynchronousOutputFailure(CoupledSourceReadStop::from(Denial::new(
                    Kind::CyclicRequiredOutputOccurrences { source: key.scope },
                    task.subject,
                )));
            }
        }
        super::super::occurrences::refusal("required outputs", "unavailable occurrence identity")
    }
    fn canonical(&self, mut i: usize) -> usize {
        while let Some(parent) = self.tasks[i].alias {
            i = parent;
        }
        i
    }
    pub(super) fn ready(&self, i: usize) -> bool {
        self.tasks[i].alias.is_none()
            && self.tasks[i].prerequisites.iter().all(|parent| {
                let parent = &self.tasks[self.canonical(*parent)];
                parent.done && parent.parent.is_some()
            })
    }
    pub(super) fn needed(&self, allowed: Option<&[String]>) -> Vec<bool> {
        let mut needed = self
            .tasks
            .iter()
            .map(|t| {
                !t.done
                    && allowed.is_none_or(|families| families.iter().any(|f| f == t.tree.family()))
            })
            .collect::<Vec<_>>();
        loop {
            let mut changed = false;
            for i in 0..self.tasks.len() {
                if !needed[i] {
                    continue;
                }
                let canonical = self.canonical(i);
                if canonical != i {
                    needed[i] = false;
                    if !self.tasks[canonical].done && !needed[canonical] {
                        needed[canonical] = true;
                        changed = true;
                    }
                    continue;
                }
                for parent in &self.tasks[i].prerequisites {
                    let parent = self.canonical(*parent);
                    if !self.tasks[parent].done && !needed[parent] {
                        needed[parent] = true;
                        changed = true;
                    }
                }
            }
            if !changed {
                return needed;
            }
        }
    }
    pub(super) fn close_frontier(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), SynchronousOutputFailure> {
        let mut i = 0;
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        while i < self.tasks.len() {
            if self.tasks[i].closed {
                i += 1;
                continue;
            }
            context.checkpoint(self.tasks[i].subject)?;
            self.tasks[i].key = self.tasks[i].tree.occurrence(context, candidate)?;
            self.tasks[i].closed = true;
            let Some(key) = self.tasks[i].key.clone() else {
                i += 1;
                continue;
            };
            let mut prior = None;
            for earlier in 0..i {
                context.checkpoint(self.tasks[i].subject)?;
                if self.tasks[earlier].key.as_ref() == Some(&key) {
                    prior = Some(self.canonical(earlier));
                    break;
                }
            }
            if let Some(prior) = prior {
                self.tasks[i].alias = Some(prior);
                self.tasks[i].done = true;
                i += 1;
                continue;
            }
            let mut incoming = Self::new();
            self.tasks[i]
                .tree
                .prerequisites(context, &pin, key.scope, &mut incoming)?;
            let count = incoming.tasks.len();
            if count != 0 {
                let bytes = count
                    .checked_mul(size_of::<usize>())
                    .and_then(|v| u64::try_from(v).ok())
                    .ok_or_else(|| {
                        super::super::occurrences::refusal(
                            self.tasks[i].subject,
                            "prerequisite capacity",
                        )
                    })?;
                let memory = context.execution.allocation_policy().reserve_memory(bytes).map_err(|cause|SynchronousOutputFailure(CoupledSourceReadStop::capture(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(self.tasks[i].subject,cause))))?;
                let mut parents = Vec::new();
                parents.try_reserve_exact(count).map_err(|_| {
                    super::super::occurrences::refusal(
                        self.tasks[i].subject,
                        "prerequisite allocation",
                    )
                })?;
                for task in incoming.tasks {
                    self.reserve_slot(context, task.subject)?;
                    parents.push(self.tasks.len());
                    self.tasks.push(task);
                }
                self.tasks[i].prerequisites = parents;
                self.tasks[i]._prerequisite_memory = memory;
            }
            i += 1;
        }
        Ok(())
    }
}
