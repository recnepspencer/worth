//! What fresh requests read after each call: the document branch's members
//! and outputs, and, while the document can be read, its selection, the
//! paged `history_page` walk and the paged `redo_choices` walk.
use super::super::fixture::label;
use super::super::fixture::reads::{read_outputs, read_tree, try_read_entities, Tree};
use super::super::reference::{History, Observed, Redo, RedoChoice, Row, Snapshot, ENTITIES};
use super::*;

impl Document<'_> {
    pub(super) fn read(&mut self, limit: u32) -> Observed {
        let live = match try_read_entities(self.host, self.branch) {
            Ok(live) => live,
            Err(WorthQueryApplicationRequestQueryDenial::ProductSelection(
                WorthQueryProductBranchAdmissionDenial::Refused(WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch),
            )) if self.closed => return self.retired(),
            Err(denial) => panic!("the member set must read: {denial:?}"),
        };
        let mut branch = Snapshot::EMPTY;
        for entity in ENTITIES {
            if let Some(member) = live[entity] {
                branch.live[entity] = true;
                let next = self.generations.len() as u32 + 1;
                branch.generation[entity] = *self.generations.entry(member.id).or_insert(next);
                branch.value[entity] = member.value;
            }
        }
        let outputs = read_outputs(self.host, self.branch, &live);
        // A writer holds the document, and a closed one is gone. Otherwise
        // the reads themselves say whether the head diverged: a write outside
        // the document is learned here, not from the next call.
        let tree = match (&self.document, self.closed || self.writer.is_some()) {
            (Some(_), false) => self.read_tree(limit),
            _ => None,
        };
        let (current, history, redo) = match tree {
            Some((current, history, redo)) => (Some(current), Some(history), Some(redo)),
            None => (None, None, None),
        };
        Observed {
            current,
            history,
            redo,
            branch: Some(branch),
            outputs: Some(outputs),
            head_diverged: (!self.closed && self.writer.is_none()).then_some(self.head_diverged),
            closed: self.closed,
            // No reader of reversal facts exists before S2's correction
            // envelope; today's route posts none the document attributes.
            reversals: Default::default(),
        }
    }

    /// After close retired the document branch: nothing reads the document
    /// or its rows.
    fn retired(&self) -> Observed {
        Observed {
            current: None,
            history: None,
            redo: None,
            branch: None,
            outputs: None,
            head_diverged: None,
            closed: true,
            reversals: Default::default(),
        }
    }

    /// `None` when the reads refuse `HeadDiverged`; any other refusal fails
    /// the seed.
    fn read_tree(&mut self, limit: u32) -> Option<(u32, History, Redo)> {
        let document = self.document.as_ref().expect("checked by the caller");
        let tree = match read_tree(self.host, document, limit) {
            Ok(tree) => tree,
            Err(refusal) => {
                assert_eq!(classify_read(&refusal), Some(Outcome::HeadDiverged), "a read refused: {refusal:?}");
                self.head_diverged = true;
                return None;
            }
        };
        let Tree { current, history, choice, children } = tree;
        let current = self.number(&current);
        let mut pages = Vec::new();
        for page in history {
            let mut rows = Vec::new();
            for entry in page {
                assert!(entry.metadata.is_empty(), "the harness records no metadata");
                let label = entry.label.as_deref().map(|text| {
                    label(text).unwrap_or_else(|| panic!("a label the DST does not write: {text}"))
                });
                rows.push(Row {
                    node: self.number(&entry.step),
                    parent: entry.parent.as_ref().map(|parent| self.number(parent)),
                    label,
                });
            }
            if !rows.is_empty() {
                pages.push(rows);
            }
        }
        let choice = match choice {
            WorthQueryHistoryRedoChoice::NoChild => RedoChoice::NoChild,
            WorthQueryHistoryRedoChoice::Single(step) => RedoChoice::Single(self.number(&step)),
            WorthQueryHistoryRedoChoice::ChoiceRequired => RedoChoice::ChoiceRequired,
        };
        let children = children
            .into_iter()
            .map(|page| page.iter().map(|step| self.number(step)).collect::<Vec<_>>())
            .filter(|page| !page.is_empty())
            .collect();
        Some((current, History { limit, pages }, Redo { choice, limit, pages: children }))
    }
}
