//! Intrusive custody notes: fixed-size commit identities require no allocation.
use super::super::{DemandRegistryState, SourceCustody};
use worth_runtime_world::facade::CompositeCommitIdentity as Commit;

#[derive(Default)]
pub(in super::super) struct CustodyCleanupQueue {
    pub head: Option<Commit>,
    tail: Option<Commit>,
    pub len: usize,
}

#[derive(Default)]
pub(in super::super) struct CustodyCleanupLinks {
    pub queued: bool,
    previous: Option<Commit>,
    pub next: Option<Commit>,
}

impl DemandRegistryState {
    pub(in super::super) fn pending_custody_prune(&self) -> bool {
        self.custody_cleanup.head.is_some()
    }

    pub(in super::super) fn note_custody(&mut self, commit: &Commit) {
        self.custody_cleanup.note(&mut self.source_custody, commit);
    }

    pub(in super::super) fn note_record_custody(
        &mut self,
        key: &super::super::WorthQueryOutputDemandKey,
    ) {
        if let Some(record) = self.records.get(key) {
            for commit in &record.source_commits {
                self.custody_cleanup.note(&mut self.source_custody, commit);
            }
        }
    }

    /// Existing custody edits can append their changed entries in the same
    /// traversal. Live, unchanged entries never become cleanup debt.
    pub(in super::super) fn note_custody_changes(
        &mut self,
        mut edit: impl FnMut(&Commit, &mut SourceCustody) -> bool,
    ) {
        let mut head = None;
        let mut last: Option<(Commit, &mut CustodyCleanupLinks)> = None;
        for (commit, custody) in &mut self.source_custody {
            if !edit(commit, custody) || custody.cleanup.queued {
                continue;
            }
            let previous = last
                .as_ref()
                .map(|(commit, _)| commit.clone())
                .or_else(|| self.custody_cleanup.tail.clone());
            custody.cleanup = CustodyCleanupLinks {
                queued: true,
                previous,
                next: None,
            };
            if let Some((_, links)) = last.take() {
                links.next = Some(commit.clone());
            } else {
                head = Some(commit.clone());
            }
            last = Some((commit.clone(), &mut custody.cleanup));
            self.custody_cleanup.len += 1;
        }
        if let Some((tail, _)) = last {
            if let Some(old_tail) = &self.custody_cleanup.tail {
                self.source_custody
                    .get_mut(old_tail)
                    .expect("queued tail exists")
                    .cleanup
                    .next = head;
            } else {
                self.custody_cleanup.head = head;
            }
            self.custody_cleanup.tail = Some(tail);
        }
    }

    pub(in super::super) fn forget_custody_note(&mut self, commit: &Commit) {
        let Some(custody) = self.source_custody.get_mut(commit) else {
            return;
        };
        if !custody.cleanup.queued {
            return;
        }
        let links = std::mem::take(&mut custody.cleanup);
        if let Some(previous) = &links.previous {
            self.source_custody
                .get_mut(previous)
                .expect("queued predecessor exists")
                .cleanup
                .next = links.next.clone();
        } else {
            self.custody_cleanup.head = links.next.clone();
        }
        if let Some(next) = &links.next {
            self.source_custody
                .get_mut(next)
                .expect("queued successor exists")
                .cleanup
                .previous = links.previous.clone();
        } else {
            self.custody_cleanup.tail = links.previous;
        }
        self.custody_cleanup.len -= 1;
    }

    pub(in super::super) fn discard_custody(&mut self, commit: &Commit) {
        self.forget_custody_note(commit);
        self.source_custody.remove(commit);
    }
}

impl CustodyCleanupQueue {
    fn note(
        &mut self,
        custody_map: &mut std::collections::BTreeMap<Commit, SourceCustody>,
        commit: &Commit,
    ) {
        let Some(custody) = custody_map.get_mut(commit) else {
            return;
        };
        if custody.cleanup.queued {
            return;
        }
        custody.cleanup = CustodyCleanupLinks {
            queued: true,
            previous: self.tail.clone(),
            next: None,
        };
        if let Some(tail) = &self.tail {
            custody_map
                .get_mut(tail)
                .expect("queued tail exists")
                .cleanup
                .next = Some(commit.clone());
        } else {
            self.head = Some(commit.clone());
        }
        self.tail = Some(commit.clone());
        self.len += 1;
    }
}
