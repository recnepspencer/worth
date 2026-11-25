//! NDJSON traces of the durable-coverage courts for
//! `specs/tla/authored_index_coverage/AuthoredIndexCoverageTrace.tla`. Written
//! only when `WORTH_DST_TRACE_DIR` is set; checked by
//! `python tools/tla/run_tlc.py trace specs/tla/authored_index_coverage/AuthoredIndexCoverageTrace.tla <file>`.
//!
//! Commits are numbered from 1 in commit order. Each packet is read from the
//! durable artifact the recovery decoded, and each observation is the
//! recovered runtime's actual answer.
use super::{field_name, Answer, History};
use crate::messagepack_value_spans::{field, fields};
use worth_relational::facade::{history::CommitId, indexes::DerivedIndexId};
use worth_testkit_relational::durable_medium::TestDurableMedium;

pub(super) struct Trace<'a> {
    history: &'a History,
    lines: Vec<String>,
}

impl<'a> Trace<'a> {
    /// The durable history with its first `checkpointed` commits in the
    /// recovery's checkpoint, then the reopen that recovered it.
    pub(super) fn recovered(history: &'a History, checkpointed: usize) -> Self {
        let mut trace = Trace {
            history,
            lines: Vec::new(),
        };
        for (ordinal, commit) in (1..).zip(history.commits) {
            if ordinal == 3 {
                trace.lines.push(r#"{"action":"Register"}"#.into());
            }
            let (packet, bound) = packet(&history.medium, commit);
            trace.lines.push(format!(
                r#"{{"action":"Commit","packet":"{packet}","bound":{bound}}}"#
            ));
            if ordinal == checkpointed {
                trace.lines.push(r#"{"action":"Checkpoint"}"#.into());
            }
        }
        trace.lines.push(r#"{"action":"CrashReopen"}"#.into());
        trace
    }

    pub(super) fn register(mut self) -> Self {
        self.lines.push(r#"{"action":"Register"}"#.into());
        self
    }

    pub(super) fn admit(mut self, index: DerivedIndexId, span: usize, answer: &Answer) -> Self {
        let outcome = if answer.is_ok() { "Ok" } else { "Refuse" };
        self.lines.push(format!(
            r#"{{"action":"Admit","index":{},"span":{span},"outcome":"{outcome}"}}"#,
            index.0
        ));
        self
    }

    pub(super) fn postings(mut self, index: DerivedIndexId, span: usize, commits: &[CommitId]) -> Self {
        let ordinals = commits
            .iter()
            .map(|commit| {
                let ordinal = self.history.commits.iter().position(|known| known == commit);
                (ordinal.expect("a posting names a history commit") + 1).to_string()
            })
            .collect::<Vec<_>>();
        self.lines.push(format!(
            r#"{{"action":"Postings","index":{},"span":{span},"commits":{{"__set__":[{}]}}}}"#,
            index.0,
            ordinals.join(",")
        ));
        self
    }

    pub(super) fn write(self, name: &str) {
        let Some(dir) = std::env::var_os("WORTH_DST_TRACE_DIR") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("create WORTH_DST_TRACE_DIR");
        let path = dir.join(format!("authored-index-coverage-{name}.ndjson"));
        std::fs::write(&path, self.lines.join("\n") + "\n").expect("write the trace");
    }
}

/// The index-write packet a commit's durable artifact holds, and its bound.
fn packet(medium: &TestDurableMedium, commit: CommitId) -> (&'static str, u64) {
    let artifact = medium
        .committed_artifacts()
        .unwrap()
        .into_iter()
        .find(|artifact| artifact.commit_id() == commit)
        .expect("each history commit is durable");
    let packet = fields(field(&artifact.as_bytes()[66..], "canonical"))
        .get("authored_index_writes")
        .expect("the canonical body names its index writes")
        .to_vec();
    if packet[0] != 0x81 {
        let unit: String = rmp_serde::from_slice(&packet).unwrap();
        assert_eq!(unit, "Unavailable", "the only unit packet");
        return ("Unavail", 0);
    }
    match field_name(&packet) {
        "Complete" => ("Legacy", 0),
        "Covered" => {
            let bound = field(field(&packet, "Covered"), "covered_next_index_id");
            ("Covered", rmp_serde::from_slice(bound).unwrap())
        }
        other => panic!("unexpected packet {other}"),
    }
}
