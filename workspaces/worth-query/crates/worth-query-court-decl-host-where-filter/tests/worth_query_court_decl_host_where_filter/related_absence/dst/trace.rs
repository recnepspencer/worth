//! Compact NDJSON trace writer for `specs/tla/related_absence/RelatedAbsenceTrace.tla`.
//! Activated when `WORTH_DST_TRACE_DIR` is set; validated by
//! `python3 tools/tla/run_tlc.py trace specs/tla/related_absence/RelatedAbsenceTrace.tla <file>`.

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

    pub fn observe(&mut self, pol: &str, dir: &str, with_pred: bool, rows: &[u64]) {
        self.lines.push(format!(
            "{{\"action\":\"ObserveQuery\",\"pol\":\"{pol}\",\"dir\":\"{dir}\",\
              \"withPred\":{with_pred},\"allowRel\":true,\"allowField\":true,\
              \"denied\":false,\"rows\":{}}}",
            task_set(rows)
        ));
    }

    pub fn pinned_read(
        &mut self,
        snap_idx: usize,
        pol: &str,
        dir: &str,
        with_pred: bool,
        rows: &[u64],
    ) {
        self.lines.push(format!(
            "{{\"action\":\"PinnedRead\",\"snapIdx\":{snap_idx},\"pol\":\"{pol}\",\
              \"dir\":\"{dir}\",\"withPred\":{with_pred},\"rows\":{}}}",
            task_set(rows)
        ));
    }

    pub fn write(&self, dir: &std::path::Path, seed: u64) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("seed-{seed}.ndjson"));
        std::fs::write(&path, self.lines.join("\n") + "\n")?;
        Ok(path)
    }
}
