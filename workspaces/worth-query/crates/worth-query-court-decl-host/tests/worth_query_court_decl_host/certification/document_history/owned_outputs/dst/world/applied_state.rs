//! The document's applied-state digest (`dst::applied_state`): the members of
//! model `a`, then each output row of its tree, in a fixed order, read on a
//! fresh request on the document branch.
use super::super::applied_state::record;
use super::super::fixture::reads::{with_branch_request, Live};
use super::super::reference::{ENTITIES, NAMES};
use super::*;

impl Document<'_> {
    pub(super) fn record_applied_state(&mut self, live: &[Option<Live>; 2]) {
        let shape = self.shape;
        let rows = with_branch_request(self.host, self.branch, |request| {
            let row = |rows: &[query::Row]| (rows[0].native_id, rows[0].value);
            let mut rows = Vec::new();
            for profile in ["dst-a", "dst-b"] {
                for entity in ENTITIES {
                    let key = format!("{profile}:{}", NAMES[entity]);
                    let read = request
                        .query(query::child_output::Read(key.clone()))
                        .execute();
                    rows.push((key, read.ok().map(|read| row(read.rows()))));
                }
            }
            if shape == Shape::WithChain {
                for profile in ["dst-mid", "dst-leaf"] {
                    let key = format!("{profile}:a");
                    let read = request
                        .query(query::first_output::Read(key.clone()))
                        .execute();
                    rows.push((key, read.ok().map(|read| row(read.rows()))));
                }
            }
            rows
        });
        let mut digest = Vec::new();
        for entity in ENTITIES {
            digest.push(self.applied.record(
                NAMES[entity],
                live[entity].map(|member| (member.id, member.value)),
            ));
        }
        for (key, row) in rows {
            digest.push(self.applied.record(&key, row));
        }
        record(digest.join(" "));
    }
}
