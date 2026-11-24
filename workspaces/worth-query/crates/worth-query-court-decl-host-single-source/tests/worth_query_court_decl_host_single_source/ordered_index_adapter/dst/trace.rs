//! NDJSON trace emitter for `specs/tla/ordered_index_adapter/OrderedIndexAdapterTrace.tla`.
use super::model::{title_group, OracleTask, MAX_ROWS};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

pub struct PostState<'a> {
    pub lifecycle: &'a str,
    pub rows: &'a BTreeMap<u64, OracleTask>,
    pub watch_active: bool,
    pub title_watch_pending: bool,
    pub focus_watch_pending: bool,
}

fn int_set(items: impl IntoIterator<Item = u64>) -> String {
    let parts: Vec<String> = items.into_iter().map(|v| v.to_string()).collect();
    format!("{{\"__set__\":[{}]}}", parts.join(","))
}

fn rows_seq(rows: &BTreeMap<u64, OracleTask>) -> String {
    let entries: Vec<String> = (1..=MAX_ROWS)
        .map(|id| match rows.get(&id) {
            Some(row) => format!(
                "{{\"lo\":{},\"hi\":{},\"title\":{}}}",
                row.lo,
                row.hi,
                title_group(&row.title)
            ),
            None => "{\"lo\":-1,\"hi\":-1,\"title\":-1}".to_owned(),
        })
        .collect();
    format!("[{}]", entries.join(","))
}

fn post(state: &PostState<'_>) -> String {
    format!(
        "\"postLifecycle\":\"{}\",\"postRows\":{},\"postWatchActive\":{},\"postTitleWatchPending\":{},\"postFocusWatchPending\":{}",
        state.lifecycle,
        rows_seq(state.rows),
        state.watch_active,
        state.title_watch_pending,
        state.focus_watch_pending,
    )
}

impl Trace {
    pub fn record_insert(&mut self, row: u64, task: &OracleTask, state: &PostState<'_>) {
        self.lines.push(format!(
            "{{\"action\":\"Insert\",\"row\":{row},\"lo\":{},\"hi\":{},\"title\":{},{}}}",
            task.lo,
            task.hi,
            title_group(&task.title),
            post(state)
        ));
    }

    pub fn record_update(&mut self, row: u64, task: &OracleTask, state: &PostState<'_>) {
        self.lines.push(format!(
            "{{\"action\":\"Update\",\"row\":{row},\"lo\":{},\"hi\":{},\"title\":{},{}}}",
            task.lo,
            task.hi,
            title_group(&task.title),
            post(state)
        ));
    }

    pub fn record_delete(&mut self, row: u64, state: &PostState<'_>) {
        self.lines.push(format!(
            "{{\"action\":\"Delete\",\"row\":{row},{}}}",
            post(state)
        ));
    }

    pub fn record_cold_reopen(&mut self, state: &PostState<'_>) {
        self.lines
            .push(format!("{{\"action\":\"ColdReopen\",{}}}", post(state)));
    }

    pub fn record_rebuild(&mut self, state: &PostState<'_>) {
        self.lines
            .push(format!("{{\"action\":\"RebuildIndex\",{}}}", post(state)));
    }

    pub fn record_refuse_not_ready(
        &mut self,
        title_probe: i64,
        probe_lo: i64,
        probe_hi: i64,
        state: &PostState<'_>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"RefuseNotReady\",\"errorKind\":\"NotReady\",\"titleProbe\":{title_probe},\"probeLo\":{probe_lo},\"probeHi\":{probe_hi},{}}}",
            post(state)
        ));
    }

    pub fn record_query(
        &mut self,
        title_probe: i64,
        title_matches: &[u64],
        focus_probe: (i64, i64),
        focus_matches: &[u64],
        state: &PostState<'_>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"Query\",\"titleProbe\":{title_probe},\"observedTitleMatches\":{},\"probeLo\":{},\"probeHi\":{},\"observedFocusMatches\":{},{}}}",
            int_set(title_matches.iter().copied()),
            focus_probe.0,
            focus_probe.1,
            int_set(focus_matches.iter().copied()),
            post(state)
        ));
    }

    pub fn record_watch_open(
        &mut self,
        title_probe: i64,
        title_matches: &[u64],
        focus_probe: (i64, i64),
        focus_matches: &[u64],
        state: &PostState<'_>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"WatchOpen\",\"titleProbe\":{title_probe},\"observedTitleMatches\":{},\"probeLo\":{},\"probeHi\":{},\"observedFocusMatches\":{},{}}}",
            int_set(title_matches.iter().copied()),
            focus_probe.0,
            focus_probe.1,
            int_set(focus_matches.iter().copied()),
            post(state)
        ));
    }

    pub fn record_watch_deliver_title(&mut self, matches: &[u64], state: &PostState<'_>) {
        self.lines.push(format!(
            "{{\"action\":\"WatchDeliverTitle\",\"observedMatches\":{},{}}}",
            int_set(matches.iter().copied()),
            post(state)
        ));
    }

    pub fn record_watch_deliver_focus(&mut self, matches: &[u64], state: &PostState<'_>) {
        self.lines.push(format!(
            "{{\"action\":\"WatchDeliverFocus\",\"observedMatches\":{},{}}}",
            int_set(matches.iter().copied()),
            post(state)
        ));
    }

    pub fn record_watch_poll_idle(&mut self, state: &PostState<'_>) {
        self.lines
            .push(format!("{{\"action\":\"WatchPollIdle\",{}}}", post(state)));
    }

    pub fn record_watch_close(&mut self, state: &PostState<'_>) {
        self.lines
            .push(format!("{{\"action\":\"WatchClose\",{}}}", post(state)));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_budget_refusal(
        &mut self,
        row: u64,
        attempted: &OracleTask,
        title_probe: i64,
        title_matches: &[u64],
        focus_probe: (i64, i64),
        focus_matches: &[u64],
        state: &PostState<'_>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"RefuseBudgetExhausted\",\"errorKind\":\"BudgetExhausted\",\"row\":{row},\"lo\":{},\"hi\":{},\"title\":{},\"titleProbe\":{title_probe},\"observedTitleMatches\":{},\"probeLo\":{},\"probeHi\":{},\"observedFocusMatches\":{},{}}}",
            attempted.lo,
            attempted.hi,
            title_group(&attempted.title),
            int_set(title_matches.iter().copied()),
            focus_probe.0,
            focus_probe.1,
            int_set(focus_matches.iter().copied()),
            post(state)
        ));
    }

    pub fn write(&self, dir: &Path, seed: u64) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let body = self.lines.join("\n") + "\n";
        let path = dir.join(format!("seed-{seed}.ndjson"));
        std::fs::write(&path, &body)?;
        if seed == 0 {
            std::fs::write(dir.join("ordered_index_adapter.ndjson"), &body)?;
        }
        Ok(path)
    }
}
