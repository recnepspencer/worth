//! NDJSON trace emitter for `specs/tla/ordered_index_adapter/OrderedIndexAdapterTrace.tla`.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

fn int_set(items: impl IntoIterator<Item = u64>) -> String {
    let parts: Vec<String> = items.into_iter().map(|v| v.to_string()).collect();
    format!("{{\"__set__\":[{}]}}", parts.join(","))
}

fn post(lifecycle: &str, live_rows: &BTreeSet<u64>) -> String {
    format!(
        "\"postLifecycle\":\"{lifecycle}\",\"postLiveRows\":{}",
        int_set(live_rows.iter().copied())
    )
}

impl Trace {
    pub fn record_insert(
        &mut self,
        row: u64,
        lo: i64,
        hi: i64,
        lifecycle: &str,
        live: &BTreeSet<u64>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"Insert\",\"row\":{row},\"lo\":{lo},\"hi\":{hi},{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_update(
        &mut self,
        row: u64,
        lo: i64,
        hi: i64,
        lifecycle: &str,
        live: &BTreeSet<u64>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"Update\",\"row\":{row},\"lo\":{lo},\"hi\":{hi},{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_delete(&mut self, row: u64, lifecycle: &str, live: &BTreeSet<u64>) {
        self.lines.push(format!(
            "{{\"action\":\"Delete\",\"row\":{row},{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_cold_reopen(&mut self, lifecycle: &str, live: &BTreeSet<u64>) {
        self.lines.push(format!(
            "{{\"action\":\"ColdReopen\",{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_rebuild(&mut self, lifecycle: &str, live: &BTreeSet<u64>) {
        self.lines.push(format!(
            "{{\"action\":\"RebuildIndex\",{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_refuse_not_ready(
        &mut self,
        probe_lo: i64,
        probe_hi: i64,
        lifecycle: &str,
        live: &BTreeSet<u64>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"RefuseNotReady\",\"probeLo\":{probe_lo},\"probeHi\":{probe_hi},{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_query(
        &mut self,
        probe_lo: i64,
        probe_hi: i64,
        matches: &[u64],
        lifecycle: &str,
        live: &BTreeSet<u64>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"Query\",\"probeLo\":{probe_lo},\"probeHi\":{probe_hi},\"observedMatches\":{},{}}}",
            int_set(matches.iter().copied()),
            post(lifecycle, live)
        ));
    }

    pub fn record_watch_open(
        &mut self,
        probe_lo: i64,
        probe_hi: i64,
        matches: &[u64],
        lifecycle: &str,
        live: &BTreeSet<u64>,
    ) {
        self.lines.push(format!(
            "{{\"action\":\"WatchOpen\",\"probeLo\":{probe_lo},\"probeHi\":{probe_hi},\"observedMatches\":{},{}}}",
            int_set(matches.iter().copied()),
            post(lifecycle, live)
        ));
    }

    pub fn record_watch_deliver(&mut self, matches: &[u64], lifecycle: &str, live: &BTreeSet<u64>) {
        self.lines.push(format!(
            "{{\"action\":\"WatchDeliver\",\"observedMatches\":{},{}}}",
            int_set(matches.iter().copied()),
            post(lifecycle, live)
        ));
    }

    pub fn record_watch_close(&mut self, lifecycle: &str, live: &BTreeSet<u64>) {
        self.lines.push(format!(
            "{{\"action\":\"WatchClose\",{}}}",
            post(lifecycle, live)
        ));
    }

    pub fn record_budget_refusal(&mut self, lifecycle: &str, live: &BTreeSet<u64>) {
        self.lines.push(format!(
            "{{\"action\":\"RefuseBudgetExhausted\",{}}}",
            post(lifecycle, live)
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
