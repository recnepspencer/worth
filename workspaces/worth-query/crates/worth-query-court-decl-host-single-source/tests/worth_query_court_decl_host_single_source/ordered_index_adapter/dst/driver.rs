//! Real Decl+Host + Directory Home DST driver paired with the sorted-scan oracle.
use super::super::schema::{DaymarkSchema, FocusIntervalIndex, TitlePrefixIndex};
use super::super::{
    create_task, delete_task, home_lease, install, limits, open_directory_home_with_limits,
    rename_task, reschedule_task, Home, Host,
};
use super::checks::{
    focus_q, post_state, prefix_q, read_observed_rows, verify_query_both,
    verify_same_host_budget_refusal,
};
use super::model::{title_probe_id, OracleTask, SortedScanOracle};
use super::trace::Trace;
use super::watches::WatchPair;
use std::path::PathBuf;
use worth_execution::{ExecutionRequest, ExecutionResourceLease};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

enum Backend {
    Directory {
        dir: PathBuf,
        lease: ExecutionResourceLease<'static>,
        work_ceiling: Option<u64>,
        home: Option<Home>,
    },
    Memory(Host),
}

pub struct DstDriver {
    backend: Backend,
    tracing: bool,
    title_idx: Option<WorthQueryManagedIndex<DaymarkSchema, TitlePrefixIndex>>,
    focus_idx: Option<WorthQueryManagedIndex<DaymarkSchema, FocusIntervalIndex>>,
    watches: WatchPair,
    next_idem: u64,
}

impl Drop for DstDriver {
    fn drop(&mut self) {
        self.close_handles();
        if let Backend::Directory {
            dir, lease, home, ..
        } = &mut self.backend
        {
            if let Some(home) = home.take() {
                let execution = ExecutionRequest::leased(lease);
                if let Ok(closed) = home.close(&execution) {
                    closed.reclaim();
                }
            }
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl DstDriver {
    pub fn new(seed: u64, durable_work_ceiling: Option<Option<u64>>, tracing: bool) -> Self {
        let backend = if let Some(work_ceiling) = durable_work_ceiling {
            let dir = std::env::temp_dir().join(format!(
                "worth-ordered-dst-{}-{seed}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let lease = home_lease();
            let execution = ExecutionRequest::leased(&lease);
            let home = open_directory_home_with_limits(
                &dir,
                &execution,
                true,
                &[],
                limits(None, work_ceiling),
            );
            Backend::Directory {
                dir,
                lease,
                work_ceiling,
                home: Some(home),
            }
        } else {
            Backend::Memory(install(&[], None, None))
        };
        let mut driver = Self {
            backend,
            tracing,
            title_idx: None,
            focus_idx: None,
            watches: WatchPair::default(),
            next_idem: 1,
        };
        driver.open_indexes();
        driver
    }

    pub fn set_tracing(&mut self, tracing: bool) {
        self.tracing = tracing;
    }

    pub fn reset_empty(&mut self, live_keys: &[u64]) {
        self.watches.close_quietly();
        for &key in live_keys {
            let idem = self.next_idem;
            self.next_idem += 1;
            delete_task(self.host(), key, idem);
        }
        debug_assert!(read_observed_rows(self.host()).unwrap().is_empty());
    }

    fn host(&self) -> &Host {
        match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().expect("home live").runtime(),
            Backend::Memory(host) => host,
        }
    }

    fn close_handles(&mut self) {
        self.watches.close_quietly();
        if let Some(idx) = self.title_idx.take() {
            idx.close();
        }
        if let Some(idx) = self.focus_idx.take() {
            idx.close();
        }
    }

    fn open_indexes(&mut self) {
        let (title_idx, focus_idx) = {
            let scope = request_scope();
            let principal = authenticate_operator(self.host().installed_schema(), &scope);
            let req = self.host().runtime().request(&principal, &scope);
            let title_idx = req
                .query(prefix_q(""))
                .open_index::<TitlePrefixIndex>()
                .unwrap();
            let focus_idx = req
                .query(focus_q(0, 0))
                .open_index::<FocusIntervalIndex>()
                .unwrap();
            (title_idx, focus_idx)
        };
        self.title_idx = Some(title_idx);
        self.focus_idx = Some(focus_idx);
    }

    pub fn lifecycle_str(&self) -> &'static str {
        let t = self.title_idx.as_ref().unwrap().lifecycle();
        let f = self.focus_idx.as_ref().unwrap().lifecycle();
        assert_eq!(t, f, "both managed indexes must share lifecycle");
        match f {
            WorthQueryManagedIndexLifecycle::Created => "Created",
            WorthQueryManagedIndexLifecycle::Ready => "Ready",
            other => panic!("unexpected managed index lifecycle: {other:?}"),
        }
    }

    fn check_traced_rows(
        &self,
        label: &str,
        oracle: &SortedScanOracle,
    ) -> Result<Option<std::collections::BTreeMap<u64, OracleTask>>, String> {
        if !self.tracing {
            return Ok(None);
        }
        let obs = read_observed_rows(self.host())?;
        if obs != oracle.rows {
            return Err(format!(
                "post-{label} rows {obs:?} != oracle {:?}",
                oracle.rows
            ));
        }
        Ok(Some(obs))
    }

    pub fn has_watch(&self) -> bool {
        self.watches.is_open()
    }

    pub fn rebuild(&mut self, oracle: &SortedScanOracle, trace: &mut Trace) -> Result<(), String> {
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        req.query(prefix_q(""))
            .rebuild_index(self.title_idx.as_ref().unwrap())
            .map_err(|e| format!("title rebuild: {e:?}"))?;
        req.query(focus_q(0, 0))
            .rebuild_index(self.focus_idx.as_ref().unwrap())
            .map_err(|e| format!("focus rebuild: {e:?}"))?;
        if let Some(obs) = self.check_traced_rows("rebuild", oracle)? {
            trace.record_rebuild(&post_state(self.lifecycle_str(), &obs, oracle));
        }
        Ok(())
    }

    pub fn cold_reopen(
        &mut self,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        self.close_handles();
        oracle.on_close_watches();
        if let Backend::Directory {
            dir,
            lease,
            work_ceiling,
            home,
        } = &mut self.backend
        {
            let execution = ExecutionRequest::leased(lease);
            home.take().unwrap().close(&execution).unwrap().reclaim();
            *home = Some(open_directory_home_with_limits(
                dir,
                &execution,
                false,
                &[],
                limits(None, *work_ceiling),
            ));
        }
        self.open_indexes();
        if let Some(obs) = self.check_traced_rows("reopen", oracle)? {
            trace.record_cold_reopen(&post_state(self.lifecycle_str(), &obs, oracle));
        }
        Ok(())
    }

    pub fn insert(
        &mut self,
        key: u64,
        task: OracleTask,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let idem = self.next_idem;
        self.next_idem += 1;
        create_task(self.host(), key, &task.title, task.lo, task.hi + 1, idem);
        oracle.on_mutation(None, Some(&task));
        oracle.rows.insert(key, task.clone());
        if let Some(obs) = self.check_traced_rows("insert", oracle)? {
            trace.record_insert(key, &task, &post_state(self.lifecycle_str(), &obs, oracle));
        }
        Ok(())
    }

    pub fn update(
        &mut self,
        key: u64,
        task: OracleTask,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let i1 = self.next_idem;
        let i2 = self.next_idem + 1;
        self.next_idem += 2;
        rename_task(self.host(), key, &task.title, i1);
        reschedule_task(self.host(), key, task.lo, task.hi + 1, i2);
        let before = oracle.rows.get(&key).cloned();
        oracle.on_mutation(before.as_ref(), Some(&task));
        oracle.rows.insert(key, task.clone());
        if let Some(obs) = self.check_traced_rows("update", oracle)? {
            trace.record_update(key, &task, &post_state(self.lifecycle_str(), &obs, oracle));
        }
        Ok(())
    }

    pub fn delete(
        &mut self,
        key: u64,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let idem = self.next_idem;
        self.next_idem += 1;
        delete_task(self.host(), key, idem);
        let before = oracle.rows.remove(&key);
        oracle.on_mutation(before.as_ref(), None);
        if let Some(obs) = self.check_traced_rows("delete", oracle)? {
            trace.record_delete(key, &post_state(self.lifecycle_str(), &obs, oracle));
        }
        Ok(())
    }

    pub fn query_both(
        &self,
        prefix: &str,
        lo: i64,
        hi: i64,
        oracle: &SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        verify_query_both(
            self.host(),
            self.lifecycle_str(),
            self.tracing,
            prefix,
            lo,
            hi,
            oracle,
            trace,
        )
    }

    pub fn open_watches(
        &mut self,
        prefix: &str,
        lo: i64,
        hi: i64,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let host = match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().expect("home live").runtime(),
            Backend::Memory(host) => host,
        };
        let (got_t, got_f) = self.watches.open(host, prefix, lo, hi, oracle)?;
        if self.tracing {
            let obs = read_observed_rows(self.host())?;
            trace.record_watch_open(
                title_probe_id(prefix),
                &got_t,
                (lo, hi),
                &got_f,
                &post_state(self.lifecycle_str(), &obs, oracle),
            );
        }
        Ok(())
    }

    pub fn poll_watches(
        &mut self,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<usize, String> {
        let obs = if self.tracing {
            Some(read_observed_rows(self.host())?)
        } else {
            None
        };
        let host = match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().expect("home live").runtime(),
            Backend::Memory(host) => host,
        };
        self.watches.poll(host, oracle, obs.as_ref(), trace)
    }

    pub fn close_watches(
        &mut self,
        oracle: &mut SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        if self.watches.close() {
            oracle.on_close_watches();
            if self.tracing {
                let obs = read_observed_rows(self.host())?;
                trace.record_watch_close(&post_state(self.lifecycle_str(), &obs, oracle));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn probe_same_host_budget_refusal(
        &mut self,
        row: u64,
        attempted: &OracleTask,
        prefix: &str,
        lo: i64,
        hi: i64,
        oracle: &SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let idem = self.next_idem;
        self.next_idem += 1;
        verify_same_host_budget_refusal(
            self.host(),
            self.lifecycle_str(),
            self.tracing,
            idem,
            row,
            attempted,
            prefix,
            lo,
            hi,
            oracle,
            trace,
        )
    }
}
