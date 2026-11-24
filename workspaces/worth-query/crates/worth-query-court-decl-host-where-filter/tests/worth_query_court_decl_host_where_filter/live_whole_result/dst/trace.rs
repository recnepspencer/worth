//! Compact NDJSON trace writer for `specs/tla/live_subscriptions/LiveSubscriptionsTrace.tla`.
//! Every emitted `outcome` and `rows` field is derived strictly from real observed
//! runtime outcomes. Activated when `WORTH_DST_TRACE_DIR` is set.
use super::reference::Kind;

#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

fn task_name(key: u64) -> String {
    format!("\"t{key}\"")
}

fn task_set(keys: &[u64]) -> String {
    let items: Vec<String> = keys.iter().map(|k| task_name(*k)).collect();
    format!("{{\"__set__\":[{}]}}", items.join(","))
}

impl Trace {
    pub fn toggle_status(&mut self, task: u64) {
        self.lines.push(format!(
            "{{\"action\":\"ToggleStatus\",\"task\":{}}}",
            task_name(task)
        ));
    }

    pub fn set_due(&mut self, task: u64, bucket: u8) {
        self.lines.push(format!(
            "{{\"action\":\"SetDue\",\"task\":{},\"due\":{bucket}}}",
            task_name(task)
        ));
    }

    pub fn add_edge(&mut self, from: u64, to: u64) {
        self.lines.push(format!(
            "{{\"action\":\"AddEdge\",\"from\":{},\"to\":{}}}",
            task_name(from),
            task_name(to)
        ));
    }

    pub fn remove_edge(&mut self, from: u64, to: u64) {
        self.lines.push(format!(
            "{{\"action\":\"RemoveEdge\",\"from\":{},\"to\":{}}}",
            task_name(from),
            task_name(to)
        ));
    }

    pub fn revoke_grant(&mut self) {
        self.lines.push("{\"action\":\"RevokeGrant\"}".to_owned());
    }

    pub fn restore_grant(&mut self) {
        self.lines.push("{\"action\":\"RestoreGrant\"}".to_owned());
    }

    pub fn reopen_home(&mut self) {
        self.lines.push("{\"action\":\"ReopenHome\"}".to_owned());
    }

    pub fn subscribe(&mut self, kind: Kind, outcome: &str, rows: &[u64]) {
        self.lines.push(format!(
            "{{\"action\":\"Subscribe\",\"kind\":\"{}\",\"outcome\":\"{outcome}\",\"rows\":{}}}",
            kind.as_str(),
            task_set(rows)
        ));
    }

    pub fn poll(&mut self, kind: Kind, outcome: &str, rows: &[u64]) {
        self.lines.push(format!(
            "{{\"action\":\"Poll\",\"kind\":\"{}\",\"outcome\":\"{outcome}\",\"rows\":{}}}",
            kind.as_str(),
            task_set(rows)
        ));
    }

    pub fn close_sub(&mut self, kind: Kind) {
        self.lines.push(format!(
            "{{\"action\":\"CloseSub\",\"kind\":\"{}\"}}",
            kind.as_str()
        ));
    }

    pub fn write(&self, dir: &std::path::Path, seed: u64) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("seed-{seed}.ndjson"));
        std::fs::write(&path, self.lines.join("\n") + "\n")?;
        Ok(path)
    }
}
