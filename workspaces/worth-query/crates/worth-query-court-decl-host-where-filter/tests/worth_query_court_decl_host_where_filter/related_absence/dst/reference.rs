//! Reference graph oracle transcribed directly from
//! `specs/tla/related_absence/RelatedAbsence.tla` plus optional-parameter and
//! literal filter semantics from PR 2A. Zero shared code with WORTH's query
//! engine: computes exact declarative set comprehension over in-memory state.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskNode {
    pub key: u64,
    pub project: u64,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub rank: i64,
    pub due_days: Option<i64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphSnapshot {
    pub tasks: BTreeMap<u64, TaskNode>,
    /// `(task, blocker)` means `task --BlockedBy--> blocker`.
    pub blocked_by: BTreeSet<(u64, u64)>,
}

impl GraphSnapshot {
    pub fn set_status(&mut self, task: u64, status: &str) {
        if let Some(node) = self.tasks.get_mut(&task) {
            node.status = status.to_owned();
        }
    }

    pub fn set_notes(&mut self, task: u64, notes: &str) {
        if let Some(node) = self.tasks.get_mut(&task) {
            node.notes = Some(notes.to_owned());
        }
    }

    pub fn link(&mut self, from: u64, to: u64) -> bool {
        self.blocked_by.insert((from, to))
    }

    pub fn unlink(&mut self, from: u64, to: u64) -> bool {
        self.blocked_by.remove(&(from, to))
    }

    /// Open tasks in `project` with ZERO open `BlockedBy` blockers (sorted by key).
    pub fn ready_tasks(&self, project: u64) -> Vec<u64> {
        self.tasks
            .values()
            .filter(|t| {
                t.project == project
                    && t.status == "open"
                    && !self.blocked_by.iter().any(|&(from, to)| {
                        from == t.key && self.tasks.get(&to).is_some_and(|b| b.status == "open")
                    })
            })
            .map(|t| t.key)
            .collect()
    }

    /// Open tasks in `project` with AT LEAST ONE open `BlockedBy` blocker.
    pub fn blocked_tasks(&self, project: u64) -> Vec<u64> {
        self.tasks
            .values()
            .filter(|t| {
                t.project == project
                    && t.status == "open"
                    && self.blocked_by.iter().any(|&(from, to)| {
                        from == t.key && self.tasks.get(&to).is_some_and(|b| b.status == "open")
                    })
            })
            .map(|t| t.key)
            .collect()
    }

    /// Open tasks in `project` that block AT LEAST ONE open task (Reverse).
    pub fn blocking_tasks(&self, project: u64) -> Vec<u64> {
        self.tasks
            .values()
            .filter(|t| {
                t.project == project
                    && t.status == "open"
                    && self.blocked_by.iter().any(|&(from, to)| {
                        to == t.key
                            && self
                                .tasks
                                .get(&from)
                                .is_some_and(|dependent| dependent.status == "open")
                    })
            })
            .map(|t| t.key)
            .collect()
    }

    /// Open tasks in `project` with ZERO `BlockedBy` edges regardless of status.
    pub fn unlinked_tasks(&self, project: u64) -> Vec<u64> {
        self.tasks
            .values()
            .filter(|t| {
                t.project == project
                    && t.status == "open"
                    && !self.blocked_by.iter().any(|&(from, _)| from == t.key)
            })
            .map(|t| t.key)
            .collect()
    }

    /// Combined literal + optional root filter + related absence with optional target rank filter.
    pub fn ready_filtered(
        &self,
        project: u64,
        max_due_days: Option<i64>,
        text: Option<&str>,
        blocker_min_rank: Option<i64>,
    ) -> Vec<u64> {
        let needle = text.map(|s| s.to_ascii_lowercase());
        self.tasks
            .values()
            .filter(|t| {
                if t.project != project || t.status != "open" {
                    return false;
                }
                if let Some(max_due) = max_due_days
                    && !t.due_days.is_some_and(|d| d <= max_due)
                {
                    return false;
                }
                if let Some(needle) = &needle
                    && !t
                        .notes
                        .as_ref()
                        .is_some_and(|n| n.to_ascii_lowercase().contains(needle))
                {
                    return false;
                }
                !self.blocked_by.iter().any(|&(from, to)| {
                    from == t.key
                        && self.tasks.get(&to).is_some_and(|b| {
                            b.status == "open"
                                && blocker_min_rank.is_none_or(|min_r| b.rank >= min_r)
                        })
                })
            })
            .map(|t| t.key)
            .collect()
    }

    /// Exact-root multi-predicate + related absence check for one task.
    pub fn exact_ready_task(&self, task: u64, expected_rank: i64) -> Vec<u64> {
        let Some(t) = self.tasks.get(&task) else {
            return Vec::new();
        };
        let is_ready = t.status == "open"
            && t.rank == expected_rank
            && !self.blocked_by.iter().any(|&(from, to)| {
                from == t.key && self.tasks.get(&to).is_some_and(|b| b.status == "open")
            });
        if is_ready {
            vec![task]
        } else {
            Vec::new()
        }
    }
}
