//! Real Decl+Host + Directory Home 4-family DST driver.
use super::dst_oracle::{check_watch, q2, q3, qi, qp, FourFamilyOracle, UNIT_B2, UNIT_B3};
use super::operations::AssetSpec;
use super::schema::*;
use super::{
    create_asset, delete_asset, home_lease, install, keys, limits, open_directory_home_with_limits,
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

struct ActiveWatches {
    prefix: String,
    window: FocusWindow,
    box2d: QueryBox2D,
    box3d: QueryBox3D,
    w_prefix: WorthQueryManagedIndexWatch<ShopFloorSchema, TitlePrefixIndex>,
    w_iv: WorthQueryManagedIndexWatch<ShopFloorSchema, FocusIntervalIndex>,
    w_b2: WorthQueryManagedIndexWatch<ShopFloorSchema, EquipmentBox2DIndex>,
    w_b3: WorthQueryManagedIndexWatch<ShopFloorSchema, RouteBox3DIndex>,
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
        let res = req.query(q2(UNIT_B2)).execute();
        match res {
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Refused(
                        WorthQueryManagedIndexRefusal::NotReady
                            | WorthQueryManagedIndexRefusal::RootUnavailable
                    )
                ) => {}
            Err(other) => return Err(format!("cold reopen expected NotReady, got Err({other:?})")),
            Ok(_) => return Err("cold reopen expected NotReady, got Ok".to_owned()),
        }
        self.ensure_all_ready();
        Ok(())
    }

    pub fn insert(&mut self, spec: AssetSpec) {
        let idem = self.next_idem;
        self.next_idem += 1;
        create_asset(self.host(), spec, idem);
    }
    pub fn update(&mut self, spec: AssetSpec) {
        let idem = self.next_idem;
        self.next_idem += 1;
        update_asset(self.host(), spec, idem);
    }
    pub fn delete(&mut self, key: u64) {
        let idem = self.next_idem;
        self.next_idem += 1;
        delete_asset(self.host(), key, idem);
    }

    pub fn reset_empty(&mut self, live: &[u64]) {
        self.close_watches();
        for &k in live {
            self.delete(k);
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
                keys(
                    req.query(qp(prefix))
                        .execute()
                        .map_err(|e| format!("prefix: {e:?}"))?
                        .rows(),
                ),
                oracle.prefix_matches(prefix),
            ),
            (
                "Interval1D",
                keys(
                    req.query(qi(w))
                        .execute()
                        .map_err(|e| format!("iv: {e:?}"))?
                        .rows(),
                ),
                oracle.interval_matches(w.start, w.end),
            ),
            (
                "Box2D",
                keys(
                    req.query(q2(b2))
                        .execute()
                        .map_err(|e| format!("box2d: {e:?}"))?
                        .rows(),
                ),
                oracle.box2d_matches(b2),
            ),
            (
                "Box3D",
                keys(
                    req.query(q3(b3))
                        .execute()
                        .map_err(|e| format!("box3d: {e:?}"))?
                        .rows(),
                ),
                oracle.box3d_matches(b3),
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
        let (prefix, window, box2d, box3d) = (probe.prefix, probe.window, probe.box2d, probe.box3d);
        let scope = request_scope();
        let principal = authenticate_operator(self.host().installed_schema(), &scope);
        let req = self.host().runtime().request(&principal, &scope);
        let (rp, w_prefix) = req
            .query(qp(prefix))
            .watch_index::<TitlePrefixIndex>()
            .map_err(|e| format!("w_p: {e:?}"))?;
        let (ri, w_iv) = req
            .query(qi(window))
            .watch_index::<FocusIntervalIndex>()
            .map_err(|e| format!("w_i: {e:?}"))?;
        let (r2, w_b2) = req
            .query(q2(box2d))
            .watch_index::<EquipmentBox2DIndex>()
            .map_err(|e| format!("w_2: {e:?}"))?;
        let (r3, w_b3) = req
            .query(q3(box3d))
            .watch_index::<RouteBox3DIndex>()
            .map_err(|e| format!("w_3: {e:?}"))?;
        if keys(rp.rows()) != oracle.prefix_matches(prefix)
            || keys(ri.rows()) != oracle.interval_matches(window.start, window.end)
            || keys(r2.rows()) != oracle.box2d_matches(box2d)
            || keys(r3.rows()) != oracle.box3d_matches(box3d)
        {
            return Err("watch initial snapshot mismatch".to_owned());
        }
        self.watches = Some(ActiveWatches {
            prefix: prefix.to_owned(),
            window,
            box2d,
            box3d,
            w_prefix,
            w_iv,
            w_b2,
            w_b3,
        });
        Ok(())
    }

    pub fn poll_watches(&mut self, oracle: &FourFamilyOracle) -> Result<bool, String> {
        let host = match &self.backend {
            Backend::Directory { home, .. } => home.as_ref().unwrap().runtime(),
            Backend::Memory(h) => h,
        };
        let Some(w) = self.watches.as_mut() else {
            return Ok(false);
        };
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let req = host.runtime().request(&principal, &scope);
        let mut delivered = false;
        delivered |= check_watch(
            "TextPrefix",
            req.query(qp(&w.prefix)).next_watch(&mut w.w_prefix),
            &oracle.prefix_matches(&w.prefix),
        )?;
        delivered |= check_watch(
            "Interval1D",
            req.query(qi(w.window)).next_watch(&mut w.w_iv),
            &oracle.interval_matches(w.window.start, w.window.end),
        )?;
        delivered |= check_watch(
            "Box2D",
            req.query(q2(w.box2d)).next_watch(&mut w.w_b2),
            &oracle.box2d_matches(w.box2d),
        )?;
        delivered |= check_watch(
            "Box3D",
            req.query(q3(w.box3d)).next_watch(&mut w.w_b3),
            &oracle.box3d_matches(w.box3d),
        )?;
        Ok(delivered)
    }

    pub fn close_watches(&mut self) {
        if let Some(w) = self.watches.take() {
            let _ = (
                w.w_prefix.close(),
                w.w_iv.close(),
                w.w_b2.close(),
                w.w_b3.close(),
            );
        }
    }
}
