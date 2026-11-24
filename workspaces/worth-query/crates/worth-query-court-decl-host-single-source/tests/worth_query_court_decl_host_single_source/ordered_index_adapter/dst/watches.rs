//! Watch handle custody and oracle-checked delivery polling for the ordered-index DST.
use super::super::queries::{FindFocusOverlap, FindTitlePrefix, TaskRow};
use super::super::schema::{DaymarkSchema, FocusIntervalIndex, FocusWindow, TitlePrefixIndex};
use super::super::{keys, Host};
use super::model::SortedScanOracle;
use super::trace::Trace;
use std::collections::BTreeSet;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

type TitleWatch = (
    String,
    Vec<u64>,
    WorthQueryManagedIndexWatch<DaymarkSchema, TitlePrefixIndex>,
);
type FocusWatch = (
    (i64, i64),
    Vec<u64>,
    WorthQueryManagedIndexWatch<DaymarkSchema, FocusIntervalIndex>,
);

#[derive(Default)]
pub struct WatchPair {
    title: Option<TitleWatch>,
    focus: Option<FocusWatch>,
}

fn prefix_q(prefix: &str) -> FindTitlePrefix {
    FindTitlePrefix {
        workspace: "main".into(),
        prefix: prefix.to_owned(),
    }
}

fn focus_q(lo: i64, hi: i64) -> FindFocusOverlap {
    FindFocusOverlap {
        workspace: "main".into(),
        window: FocusWindow::new(lo, hi + 1),
    }
}

fn check_watch_outcome<Q>(
    label: &str,
    outcome: WorthQueryManagedIndexWatchOutcome<Q, TaskRow>,
    last: &mut Vec<u64>,
    expected: &[u64],
) -> Result<Option<Vec<u64>>, String> {
    match outcome {
        WorthQueryManagedIndexWatchOutcome::Delivered(upd) => {
            let got = keys(upd.read().rows());
            if got != expected {
                return Err(format!(
                    "{label} watch deliver: got {got:?}, oracle {expected:?}"
                ));
            }
            *last = got.clone();
            Ok(Some(got))
        }
        WorthQueryManagedIndexWatchOutcome::Pending if *last != expected => Err(format!(
            "{label} watch missed update: last {last:?}, oracle {expected:?}"
        )),
        WorthQueryManagedIndexWatchOutcome::Pending => Ok(None),
        WorthQueryManagedIndexWatchOutcome::Overflow {
            missed_commit_batches,
        } => Err(format!("{label} watch overflow: {missed_commit_batches}")),
        WorthQueryManagedIndexWatchOutcome::Unavailable => {
            Err(format!("{label} watch unavailable"))
        }
        WorthQueryManagedIndexWatchOutcome::Closed => {
            Err(format!("{label} watch unexpectedly closed"))
        }
    }
}

impl WatchPair {
    pub fn is_open(&self) -> bool {
        self.focus.is_some()
    }

    pub fn close_quietly(&mut self) {
        if let Some((_, _, w)) = self.title.take() {
            let _ = w.close();
        }
        if let Some((_, _, w)) = self.focus.take() {
            let _ = w.close();
        }
    }

    pub fn close(&mut self) -> bool {
        if let Some((_, _, w)) = self.title.take() {
            w.close().unwrap();
        }
        if let Some((_, _, w)) = self.focus.take() {
            w.close().unwrap();
            return true;
        }
        false
    }

    pub fn open(
        &mut self,
        host: &Host,
        prefix: &str,
        lo: i64,
        hi: i64,
        oracle: &SortedScanOracle,
    ) -> Result<Vec<u64>, String> {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let req = host.runtime().request(&principal, &scope);
        let (t_init, tw) = req
            .query(prefix_q(prefix))
            .watch_index::<TitlePrefixIndex>()
            .map_err(|e| format!("title watch open: {e:?}"))?;
        let (f_init, fw) = req
            .query(focus_q(lo, hi))
            .watch_index::<FocusIntervalIndex>()
            .map_err(|e| format!("focus watch open: {e:?}"))?;
        let got_t = keys(t_init.rows());
        let exp_t = oracle.prefix_matches(prefix);
        if got_t != exp_t {
            return Err(format!("title watch init: got {got_t:?}, oracle {exp_t:?}"));
        }
        let got_f = keys(f_init.rows());
        let exp_f = oracle.focus_matches(lo, hi);
        if got_f != exp_f {
            return Err(format!("focus watch init: got {got_f:?}, oracle {exp_f:?}"));
        }
        self.title = Some((prefix.to_owned(), got_t, tw));
        self.focus = Some(((lo, hi), got_f.clone(), fw));
        Ok(got_f)
    }

    pub fn poll(
        &mut self,
        host: &Host,
        oracle: &SortedScanOracle,
        live: Option<&BTreeSet<u64>>,
        trace: &mut Trace,
    ) -> Result<bool, String> {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let req = host.runtime().request(&principal, &scope);
        if let Some((prefix, last, tw)) = self.title.as_mut() {
            let exp = oracle.prefix_matches(prefix);
            let out = req
                .query(prefix_q(prefix))
                .next_watch(tw)
                .map_err(|e| format!("title next_watch: {e:?}"))?;
            check_watch_outcome("title", out, last, &exp)?;
        }
        let mut focus_delivered = false;
        if let Some(((lo, hi), last, fw)) = self.focus.as_mut() {
            let exp = oracle.focus_matches(*lo, *hi);
            let out = req
                .query(focus_q(*lo, *hi))
                .next_watch(fw)
                .map_err(|e| format!("focus next_watch: {e:?}"))?;
            if let Some(got) = check_watch_outcome("focus", out, last, &exp)? {
                focus_delivered = true;
                if let Some(live) = live {
                    trace.record_watch_deliver(&got, "Ready", live);
                }
            }
        }
        Ok(focus_delivered)
    }
}
