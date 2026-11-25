//! Watch handle custody and exact Idle/Pending/Delivered verification for ordered-index DST.
use super::super::queries::{FindFocusOverlap, FindTitlePrefix, TaskRow};
use super::super::schema::{DaymarkSchema, FocusIntervalIndex, FocusWindow, TitlePrefixIndex};
use super::super::{keys, Host};
use super::model::{OracleTask, SortedScanOracle};
use super::trace::{PostState, Trace};
use std::collections::BTreeMap;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

type TitleWatch = (
    String,
    WorthQueryManagedIndexWatch<DaymarkSchema, TitlePrefixIndex>,
);
type FocusWatch = (
    (i64, i64),
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

fn drain_watch<Q, F>(
    label: &str,
    expected_pending: bool,
    expected_rows: &[u64],
    mut poll_once: F,
) -> Result<Option<Vec<u64>>, String>
where
    F: FnMut() -> Result<WorthQueryManagedIndexWatchOutcome<Q, TaskRow>, String>,
{
    let mut delivered: Option<Vec<u64>> = None;
    loop {
        match poll_once()? {
            WorthQueryManagedIndexWatchOutcome::Delivered(upd) => {
                if !expected_pending && delivered.is_none() {
                    return Err(format!("{label} watch spuriously delivered while Idle"));
                }
                let got = keys(upd.read().rows());
                if got != expected_rows {
                    return Err(format!(
                        "{label} watch deliver: got {got:?}, oracle {expected_rows:?}"
                    ));
                }
                delivered = Some(got);
            }
            WorthQueryManagedIndexWatchOutcome::Pending => {
                if expected_pending && delivered.is_none() {
                    return Err(format!(
                        "{label} watch stayed Pending when delivery expected"
                    ));
                }
                return Ok(delivered);
            }
            WorthQueryManagedIndexWatchOutcome::Overflow {
                missed_commit_batches,
            } => return Err(format!("{label} watch overflow: {missed_commit_batches}")),
            WorthQueryManagedIndexWatchOutcome::Unavailable => {
                return Err(format!("{label} watch unavailable"));
            }
            WorthQueryManagedIndexWatchOutcome::Closed => {
                return Err(format!("{label} watch unexpectedly closed"));
            }
        }
    }
}

impl WatchPair {
    pub fn is_open(&self) -> bool {
        self.focus.is_some()
    }

    pub fn close_quietly(&mut self) {
        if let Some((_, w)) = self.title.take() {
            let _ = w.close();
        }
        if let Some((_, w)) = self.focus.take() {
            let _ = w.close();
        }
    }

    pub fn close(&mut self) -> bool {
        if let Some((_, w)) = self.title.take() {
            w.close().unwrap();
        }
        if let Some((_, w)) = self.focus.take() {
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
        oracle: &mut SortedScanOracle,
    ) -> Result<(Vec<u64>, Vec<u64>), String> {
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
        self.title = Some((prefix.to_owned(), tw));
        self.focus = Some(((lo, hi), fw));
        oracle.on_open_watches(prefix, lo, hi);
        Ok((got_t, got_f))
    }

    pub fn poll(
        &mut self,
        host: &Host,
        oracle: &mut SortedScanOracle,
        observed_rows: Option<&BTreeMap<u64, OracleTask>>,
        trace: &mut Trace,
    ) -> Result<usize, String> {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let req = host.runtime().request(&principal, &scope);
        let both_idle = !oracle.title_watch_pending && !oracle.focus_watch_pending;
        let mut deliveries = 0;

        if let Some((prefix, tw)) = self.title.as_mut() {
            let exp = oracle.prefix_matches(prefix);
            if let Some(got) = drain_watch("title", oracle.title_watch_pending, &exp, || {
                req.query(prefix_q(prefix))
                    .next_watch(tw)
                    .map_err(|e| format!("title next_watch: {e:?}"))
            })? {
                oracle.title_watch_pending = false;
                deliveries += 1;
                if let Some(rows) = observed_rows {
                    let state = PostState {
                        lifecycle: "Ready",
                        rows,
                        watch_active: true,
                        title_watch_pending: oracle.title_watch_pending,
                        focus_watch_pending: oracle.focus_watch_pending,
                    };
                    trace.record_watch_deliver_title(&got, &state);
                }
            }
        }
        if let Some(((lo, hi), fw)) = self.focus.as_mut() {
            let exp = oracle.focus_matches(*lo, *hi);
            if let Some(got) = drain_watch("focus", oracle.focus_watch_pending, &exp, || {
                req.query(focus_q(*lo, *hi))
                    .next_watch(fw)
                    .map_err(|e| format!("focus next_watch: {e:?}"))
            })? {
                oracle.focus_watch_pending = false;
                deliveries += 1;
                if let Some(rows) = observed_rows {
                    let state = PostState {
                        lifecycle: "Ready",
                        rows,
                        watch_active: true,
                        title_watch_pending: oracle.title_watch_pending,
                        focus_watch_pending: oracle.focus_watch_pending,
                    };
                    trace.record_watch_deliver_focus(&got, &state);
                }
            }
        }
        if both_idle && let Some(rows) = observed_rows {
            let state = PostState {
                lifecycle: "Ready",
                rows,
                watch_active: true,
                title_watch_pending: false,
                focus_watch_pending: false,
            };
            trace.record_watch_poll_idle(&state);
        }
        Ok(deliveries)
    }
}
