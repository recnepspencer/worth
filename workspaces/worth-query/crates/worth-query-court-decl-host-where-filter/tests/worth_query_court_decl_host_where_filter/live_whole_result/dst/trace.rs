//! Compact NDJSON trace writer for `specs/tla/live_subscriptions/LiveSubscriptionsTrace.tla`.
//! Every emitted `outcome`, `reason`, `commitIdx`, ordered projected `rows`,
//! and `closeOutcome` is derived strictly from real observed runtime outcomes.
//! Activated when `WORTH_DST_TRACE_DIR` is set.
use super::reference::{Kind, ProjectedRow};

#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

fn task_name(key: u64) -> String {
    format!("\"t{key}\"")
}

fn projected_rows(rows: &[ProjectedRow]) -> String {
    let items: Vec<String> = rows
        .iter()
        .map(|&(k, status, due)| format!("[{},\"{status}\",{due}]", task_name(k)))
        .collect();
    format!("[{}]", items.join(","))
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

    pub fn subscribe(
        &mut self,
        kind: Kind,
        outcome: &str,
        reason: &str,
        commit_idx: usize,
        rows: &[ProjectedRow],
    ) {
        self.lines.push(format!(
            "{{\"action\":\"Subscribe\",\"kind\":\"{}\",\"outcome\":\"{outcome}\",\"reason\":\"{reason}\",\"commitIdx\":{commit_idx},\"rows\":{}}}",
            kind.as_str(),
            projected_rows(rows)
        ));
    }

    pub fn poll(
        &mut self,
        kind: Kind,
        outcome: &str,
        reason: &str,
        commit_idx: usize,
        rows: &[ProjectedRow],
    ) {
        self.lines.push(format!(
            "{{\"action\":\"Poll\",\"kind\":\"{}\",\"outcome\":\"{outcome}\",\"reason\":\"{reason}\",\"commitIdx\":{commit_idx},\"rows\":{}}}",
            kind.as_str(),
            projected_rows(rows)
        ));
    }

    pub fn close_sub(&mut self, kind: Kind, close_outcome: &str) {
        self.lines.push(format!(
            "{{\"action\":\"CloseSub\",\"kind\":\"{}\",\"closeOutcome\":\"{close_outcome}\"}}",
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
