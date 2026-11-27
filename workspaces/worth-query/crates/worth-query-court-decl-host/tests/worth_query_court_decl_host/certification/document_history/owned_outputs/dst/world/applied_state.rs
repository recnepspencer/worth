//! The document's applied-state digest (`dst::applied_state`): the members `e1`
//! and `e2` of model `a`, then the model, its unchanged member `m`, the
//! upstream member and each output row of its tree, in a fixed order, read on a
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
            let model = request.query(query::model::Read("a".into())).execute();
            rows.push((
                "model:a".to_owned(),
                model.ok().map(|read| row(read.rows())),
            ));
            // `m` is never edited; reading it through the member set also pins
            // its ownership edge.
            let members = request.query(query::members::Read("a".into())).execute();
            let unchanged = members.ok().and_then(|read| {
                read.rows()[0]
                    .members
                    .iter()
                    .find(|member| member.key == "m")
                    .map(|member| (member.native_id, member.value))
            });
            rows.push(("m".to_owned(), unchanged));
            let upstream = request
                .query(query::member_public::Read("upstream:a".into()))
                .execute();
            rows.push((
                "upstream:a".to_owned(),
                upstream.ok().map(|read| row(read.rows())),
            ));
            for profile in ["dst-a", "dst-b"] {
                for name in NAMES.into_iter().chain(["m"]) {
                    let key = format!("{profile}:{name}");
                    let read = request
                        .query(query::child_output::Read(key.clone()))
                        .execute();
                    rows.push((key, read.ok().map(|read| row(read.rows()))));
                }
            }
            let model_keyed: &[&str] = match shape {
                Shape::Members => &[],
                Shape::WithConsumer => &["dst-use"],
                Shape::WithChain => &["dst-mid", "dst-leaf"],
            };
            for profile in model_keyed {
                let key = format!("{profile}:a");
                let read = request
                    .query(query::first_output::Read(key.clone()))
                    .execute();
                rows.push((key, read.ok().map(|read| row(read.rows()))));
            }
            rows
        });
        let mut digest = Vec::new();
        for entity in ENTITIES {
            digest.push(self.applied.label(
                NAMES[entity],
                live[entity].map(|member| (member.id, member.value)),
            ));
        }
        for (key, row) in rows {
            digest.push(self.applied.label(&key, row));
        }
        record(digest.join(" "));
    }
}
