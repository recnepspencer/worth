//! Real Decl+Host + Directory Home 4-family DST driver.
use super::dst_oracle::{q2, q3, qi, qp, FourFamilyOracle, UNIT_B2, UNIT_B3};
use super::dst_watches::ActiveWatches;
use super::operations::AssetSpec;
use super::schema::*;
use super::{
    create_asset, delete_asset, home_lease, install, limits, open_directory_home_with_limits,
    update_asset, Home, Host,
};
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

pub struct FourFamilyDriver {
    backend: Backend,
    h_prefix: Option<WorthQueryManagedIndex<ShopFloorSchema, TitlePrefixIndex>>,
    h_iv: Option<WorthQueryManagedIndex<ShopFloorSchema, FocusIntervalIndex>>,
    h_b2: Option<WorthQueryManagedIndex<ShopFloorSchema, EquipmentBox2DIndex>>,
    h_b3: Option<WorthQueryManagedIndex<ShopFloorSchema, RouteBox3DIndex>>,
    watches: Option<ActiveWatches>,
    next_idem: u64,
}

impl Drop for FourFamilyDriver {
    fn drop(&mut self) {
        self.close_handles();
        if let Backend::Directory { dir, lease, home } = &mut self.backend {
            if let Some(home) = home.take() {
                let exec = ExecutionRequest::leased(lease);
                if let Ok(closed) = home.close(&exec) {
                    closed.reclaim();
                }
            }
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

pub struct FourFamilyProbe<'a> {
    pub prefix: &'a str,
    pub window: FocusWindow,
    pub box2d: QueryBox2D,
    pub box3d: QueryBox3D,
}

impl FourFamilyDriver {
    pub fn new(seed: u64, durable: bool) -> Self {
        let backend = if durable {
            let dir = std::env::temp_dir().join(format!(
                "worth-4family-dst-{}-{seed}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let lease = home_lease();
            let exec = ExecutionRequest::leased(&lease);
            let home = open_directory_home_with_limits(&dir, &exec, true, &[], limits(None, None));
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
            h_prefix: None,
            h_iv: None,
            h_b2: None,
            h_b3: None,
            watches: None,
            next_idem: 1,
        };
        driver.ensure_all_ready();
        driver
    }

    pub fn is_durable(&self) -> bool {
        matches!(self.backend, Backend::Directory { .. })
    }
    pub fn has_watches(&self) -> bool {
        self.watches.is_some()
    }

    fn host(&self) -> &Host {
        match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().expect("home live").runtime(),
            Backend::Memory(host) => host,
        }
    }

    fn close_handles(&mut self) {
        self.close_watches();
        let _ = (
            self.h_prefix.take().map(|h| h.close()),
            self.h_iv.take().map(|h| h.close()),
            self.h_b2.take().map(|h| h.close()),
            self.h_b3.take().map(|h| h.close()),
        );
    }

    pub fn ensure_all_ready(&mut self) {
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        let p = req
            .query(qp(""))
            .ensure_index_ready::<TitlePrefixIndex>()
            .unwrap();
        let i = req
            .query(qi(FocusWindow { start: 0, end: 1 }))
            .ensure_index_ready::<FocusIntervalIndex>()
            .unwrap();
        let b2 = req
            .query(q2(UNIT_B2))
            .ensure_index_ready::<EquipmentBox2DIndex>()
            .unwrap();
        let b3 = req
            .query(q3(UNIT_B3))
            .ensure_index_ready::<RouteBox3DIndex>()
            .unwrap();
        self.h_prefix = Some(p);
        self.h_iv = Some(i);
        self.h_b2 = Some(b2);
        self.h_b3 = Some(b3);
    }

    pub fn cold_reopen_and_verify_not_ready(&mut self) -> Result<(), String> {
        self.close_handles();
        if let Backend::Directory { dir, lease, home } = &mut self.backend {
            let exec = ExecutionRequest::leased(lease);
            home.take().unwrap().close(&exec).unwrap().reclaim();
            *home = Some(open_directory_home_with_limits(
                dir,
                &exec,
                false,
                &[],
                limits(None, None),
            ));
        }
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        // Unopened cold index refuses with exact `RootUnavailable`.
        match req.query(q2(UNIT_B2)).execute() {
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Refused(
                        WorthQueryManagedIndexRefusal::RootUnavailable
                    )
                ) => {}
            Err(other) => {
                return Err(format!(
                    "cold reopen unopened execute expected exact RootUnavailable, got Err({other:?})"
                ));
            }
            Ok(_) => return Err("cold reopen expected RootUnavailable, got Ok".to_owned()),
        }
        // Opening without rebuilding leaves the registration `Created`, and the
        // cold Native maintained index still refuses execute with exact `RootUnavailable`.
        let cold_handle = req
            .query(q2(UNIT_B2))
            .open_index::<EquipmentBox2DIndex>()
            .map_err(|e| format!("cold open_index failed: {e:?}"))?;
        assert_eq!(
            cold_handle.lifecycle(),
            WorthQueryManagedIndexLifecycle::Created
        );
        match req.query(q2(UNIT_B2)).execute() {
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Refused(
                        WorthQueryManagedIndexRefusal::RootUnavailable
                    )
                ) => {}
            Err(other) => {
                return Err(format!(
                    "cold reopen opened execute expected exact RootUnavailable, got Err({other:?})"
                ));
            }
            Ok(_) => {
                return Err(
                    "cold reopen opened execute expected RootUnavailable, got Ok".to_owned(),
                );
            }
        }
        drop(cold_handle);
        self.ensure_all_ready();
        Ok(())
    }

    pub fn insert(&mut self, spec: AssetSpec) {
        let idem = self.next_idem;
        self.next_idem += 1;
        create_asset(self.host(), spec.clone(), idem);
        if let Some(w) = self.watches.as_mut() {
            w.note_mutation(None, Some(&spec));
        }
    }
    pub fn update(&mut self, old: Option<&AssetSpec>, spec: AssetSpec) {
        let idem = self.next_idem;
        self.next_idem += 1;
        update_asset(self.host(), spec.clone(), idem);
        if let Some(w) = self.watches.as_mut() {
            w.note_mutation(old, Some(&spec));
        }
    }
    pub fn delete(&mut self, key: u64, old: Option<&AssetSpec>) {
        let idem = self.next_idem;
        self.next_idem += 1;
        delete_asset(self.host(), key, idem);
        if let Some(w) = self.watches.as_mut() {
            w.note_mutation(old, None);
        }
    }

    pub fn reset_empty(&mut self, live: &[u64]) {
        self.close_watches();
        for &k in live {
            self.delete(k, None);
        }
    }

    pub fn query_all(
        &self,
        probe: &FourFamilyProbe<'_>,
        oracle: &FourFamilyOracle,
    ) -> Result<(), String> {
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        let (prefix, w, b2, b3) = (probe.prefix, probe.window, probe.box2d, probe.box3d);
        let checks = [
            (
                "TextPrefix",
                req.query(qp(prefix))
                    .execute()
                    .map_err(|e| format!("prefix: {e:?}"))?
                    .rows()
                    .to_vec(),
                oracle.prefix_rows(prefix),
            ),
            (
                "Interval1D",
                req.query(qi(w))
                    .execute()
                    .map_err(|e| format!("iv: {e:?}"))?
                    .rows()
                    .to_vec(),
                oracle.interval_rows(w.start, w.end),
            ),
            (
                "Box2D",
                req.query(q2(b2))
                    .execute()
                    .map_err(|e| format!("box2d: {e:?}"))?
                    .rows()
                    .to_vec(),
                oracle.box2d_rows(b2),
            ),
            (
                "Box3D",
                req.query(q3(b3))
                    .execute()
                    .map_err(|e| format!("box3d: {e:?}"))?
                    .rows()
                    .to_vec(),
                oracle.box3d_rows(b3),
            ),
        ];
        for (name, got, exp) in checks {
            if got != exp {
                return Err(format!("{name}: got {got:?}, oracle {exp:?}"));
            }
        }
        Ok(())
    }

    pub fn open_watches(
        &mut self,
        probe: &FourFamilyProbe<'_>,
        oracle: &FourFamilyOracle,
    ) -> Result<(), String> {
        let active = ActiveWatches::open(
            self.host(),
            probe.prefix,
            probe.window,
            probe.box2d,
            probe.box3d,
            oracle,
        )?;
        self.watches = Some(active);
        Ok(())
    }

    pub fn poll_watches(&mut self, oracle: &FourFamilyOracle) -> Result<(usize, usize), String> {
        let host = match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().unwrap().runtime(),
            Backend::Memory(h) => h,
        };
        let Some(w) = self.watches.as_mut() else {
            return Ok((0, 0));
        };
        w.poll(host, oracle)
    }

    pub fn close_watches(&mut self) {
        if let Some(w) = self.watches.take() {
            w.close();
        }
    }
}
