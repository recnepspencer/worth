//! Real Decl+Host + Directory Home DST driver paired with the sorted-scan oracle.
use super::super::queries::*;
use super::super::schema::*;
use super::super::{
    create_task, delete_task, home_lease, install, keys, open_directory_home, rename_task,
    reschedule_task, Home, Host,
};
use super::model::{verify_tight_budget_refusal, SortedScanOracle};
use super::trace::Trace;
use super::watches::WatchPair;
use std::collections::BTreeSet;
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
        if let Backend::Directory { dir, lease, home } = &mut self.backend {
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

impl DstDriver {
    pub fn new(seed: u64, durable_directory: bool, tracing: bool) -> Self {
        let backend = if durable_directory {
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
            let home = open_directory_home(&dir, &execution, true, &[]);
            Backend::Directory {
                dir,
                lease,
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

    pub fn is_durable(&self) -> bool {
        matches!(self.backend, Backend::Directory { .. })
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
        debug_assert!(self.observed_live_rows().is_empty());
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

    pub fn observed_live_rows(&self) -> BTreeSet<u64> {
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        req.query(ReadTasks {
            workspace: "main".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.key)
        .collect()
    }

    pub fn has_watch(&self) -> bool {
        self.watches.is_open()
    }

    pub fn rebuild(&mut self, trace: &mut Trace) {
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        req.query(prefix_q(""))
            .rebuild_index(self.title_idx.as_ref().unwrap())
            .unwrap();
        req.query(focus_q(0, 0))
            .rebuild_index(self.focus_idx.as_ref().unwrap())
            .unwrap();
        if self.tracing {
            trace.record_rebuild(self.lifecycle_str(), &self.observed_live_rows());
        }
    }

    pub fn cold_reopen(&mut self, trace: &mut Trace) {
        self.close_handles();
        if let Backend::Directory { dir, lease, home } = &mut self.backend {
            let execution = ExecutionRequest::leased(lease);
            home.take().unwrap().close(&execution).unwrap().reclaim();
            *home = Some(open_directory_home(dir, &execution, false, &[]));
        }
        self.open_indexes();
        if self.tracing {
            trace.record_cold_reopen(self.lifecycle_str(), &self.observed_live_rows());
        }
    }

    pub fn insert(&mut self, key: u64, title: &str, lo: i64, hi: i64, trace: &mut Trace) {
        let idem = self.next_idem;
        self.next_idem += 1;
        create_task(self.host(), key, title, lo, hi + 1, idem);
        if self.tracing {
            trace.record_insert(
                key,
                lo,
                hi,
                self.lifecycle_str(),
                &self.observed_live_rows(),
            );
        }
    }

    pub fn update(&mut self, key: u64, title: &str, lo: i64, hi: i64, trace: &mut Trace) {
        let i1 = self.next_idem;
        let i2 = self.next_idem + 1;
        self.next_idem += 2;
        rename_task(self.host(), key, title, i1);
        reschedule_task(self.host(), key, lo, hi + 1, i2);
        if self.tracing {
            trace.record_update(
                key,
                lo,
                hi,
                self.lifecycle_str(),
                &self.observed_live_rows(),
            );
        }
    }

    pub fn delete(&mut self, key: u64, trace: &mut Trace) {
        let idem = self.next_idem;
        self.next_idem += 1;
        delete_task(self.host(), key, idem);
        if self.tracing {
            trace.record_delete(key, self.lifecycle_str(), &self.observed_live_rows());
        }
    }

    pub fn query_both(
        &self,
        prefix: &str,
        lo: i64,
        hi: i64,
        oracle: &SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        let t_res = req.query(prefix_q(prefix)).execute();
        let f_res = req.query(focus_q(lo, hi)).execute();
        let live: BTreeSet<u64> = req
            .query(ReadTasks {
                workspace: "main".into(),
            })
            .execute()
            .unwrap()
            .rows()
            .iter()
            .map(|r| r.key)
            .collect();
        if live != oracle.live_ids() {
            return Err(format!(
                "live rows {live:?} != oracle {:?}",
                oracle.live_ids()
            ));
        }
        if self.lifecycle_str() == "Created" {
            for (label, ok) in [("title", t_res.is_ok()), ("focus", f_res.is_ok())] {
                if ok {
                    return Err(format!("cold {label} query succeeded before rebuild"));
                }
            }
            if self.tracing {
                trace.record_refuse_not_ready(lo, hi, "Created", &live);
            }
            return Ok(());
        }
        let got_t = keys(t_res.map_err(|e| format!("title query err: {e:?}"))?.rows());
        let exp_t = oracle.prefix_matches(prefix);
        if got_t != exp_t {
            return Err(format!(
                "title prefix {prefix:?}: got {got_t:?}, oracle {exp_t:?}"
            ));
        }
        let got_f = keys(f_res.map_err(|e| format!("focus query err: {e:?}"))?.rows());
        let exp_f = oracle.focus_matches(lo, hi);
        if got_f != exp_f {
            return Err(format!(
                "focus [{lo},{hi}]: got {got_f:?}, oracle {exp_f:?}"
            ));
        }
        if self.tracing {
            trace.record_query(lo, hi, &got_f, "Ready", &live);
        }
        Ok(())
    }

    pub fn open_watches(
        &mut self,
        prefix: &str,
        lo: i64,
        hi: i64,
        oracle: &SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        let host = match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().expect("home live").runtime(),
            Backend::Memory(host) => host,
        };
        let got_f = self.watches.open(host, prefix, lo, hi, oracle)?;
        if self.tracing {
            trace.record_watch_open(lo, hi, &got_f, "Ready", &self.observed_live_rows());
        }
        Ok(())
    }

    pub fn poll_watches(
        &mut self,
        oracle: &SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<bool, String> {
        let live = self.tracing.then(|| self.observed_live_rows());
        let host = match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().expect("home live").runtime(),
            Backend::Memory(host) => host,
        };
        self.watches.poll(host, oracle, live.as_ref(), trace)
    }

    pub fn close_watches(&mut self, trace: &mut Trace) {
        if self.watches.close() && self.tracing {
            trace.record_watch_close(self.lifecycle_str(), &self.observed_live_rows());
        }
    }

    pub fn probe_budget_refusal(
        &self,
        oracle: &SortedScanOracle,
        trace: &mut Trace,
    ) -> Result<(), String> {
        verify_tight_budget_refusal(oracle)?;
        if self.tracing {
            trace.record_budget_refusal(self.lifecycle_str(), &self.observed_live_rows());
        }
        Ok(())
    }
}
