//! Active 4-family watch tracker and strict per-mutation wake verifier for Hard DX DST.
use super::dst_oracle::{
    check_watch, q2, q3, qi, qp, row_matches_box2d, row_matches_box3d, row_matches_interval,
    row_matches_prefix, FourFamilyOracle,
};
use super::operations::AssetSpec;
use super::schema::*;
use super::{keys, Host};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub struct ActiveWatches {
    prefix: String,
    window: FocusWindow,
    box2d: QueryBox2D,
    box3d: QueryBox3D,
    w_prefix: WorthQueryManagedIndexWatch<ShopFloorSchema, TitlePrefixIndex>,
    w_iv: WorthQueryManagedIndexWatch<ShopFloorSchema, FocusIntervalIndex>,
    w_b2: WorthQueryManagedIndexWatch<ShopFloorSchema, EquipmentBox2DIndex>,
    w_b3: WorthQueryManagedIndexWatch<ShopFloorSchema, RouteBox3DIndex>,
    pending_wake: [bool; 4],
}

impl ActiveWatches {
    pub fn open(
        host: &Host,
        prefix: &str,
        window: FocusWindow,
        box2d: QueryBox2D,
        box3d: QueryBox3D,
        oracle: &FourFamilyOracle,
    ) -> Result<Self, String> {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let req = host.runtime().request(&principal, &scope);
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
        Ok(Self {
            prefix: prefix.to_owned(),
            window,
            box2d,
            box3d,
            w_prefix,
            w_iv,
            w_b2,
            w_b3,
            pending_wake: [false; 4],
        })
    }

    pub fn note_mutation(&mut self, old: Option<&AssetSpec>, new: Option<&AssetSpec>) {
        self.pending_wake[0] |= old.is_some_and(|r| row_matches_prefix(r, &self.prefix))
            || new.is_some_and(|r| row_matches_prefix(r, &self.prefix));
        self.pending_wake[1] |= old
            .is_some_and(|r| row_matches_interval(r, self.window.start, self.window.end))
            || new.is_some_and(|r| row_matches_interval(r, self.window.start, self.window.end));
        self.pending_wake[2] |= old.is_some_and(|r| row_matches_box2d(r, self.box2d))
            || new.is_some_and(|r| row_matches_box2d(r, self.box2d));
        self.pending_wake[3] |= old.is_some_and(|r| row_matches_box3d(r, self.box3d))
            || new.is_some_and(|r| row_matches_box3d(r, self.box3d));
    }

    pub fn poll(
        &mut self,
        host: &Host,
        oracle: &FourFamilyOracle,
    ) -> Result<(usize, usize), String> {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let req = host.runtime().request(&principal, &scope);
        let expected = self.pending_wake;
        let mut delivered = 0usize;
        let mut pending = 0usize;
        for (did, expect) in [
            (
                check_watch(
                    "TextPrefix",
                    req.query(qp(&self.prefix)).next_watch(&mut self.w_prefix),
                    &oracle.prefix_matches(&self.prefix),
                    expected[0],
                )?,
                expected[0],
            ),
            (
                check_watch(
                    "Interval1D",
                    req.query(qi(self.window)).next_watch(&mut self.w_iv),
                    &oracle.interval_matches(self.window.start, self.window.end),
                    expected[1],
                )?,
                expected[1],
            ),
            (
                check_watch(
                    "Box2D",
                    req.query(q2(self.box2d)).next_watch(&mut self.w_b2),
                    &oracle.box2d_matches(self.box2d),
                    expected[2],
                )?,
                expected[2],
            ),
            (
                check_watch(
                    "Box3D",
                    req.query(q3(self.box3d)).next_watch(&mut self.w_b3),
                    &oracle.box3d_matches(self.box3d),
                    expected[3],
                )?,
                expected[3],
            ),
        ] {
            if did && expect {
                delivered += 1;
            } else {
                pending += 1;
            }
        }
        self.pending_wake = [false; 4];
        Ok((delivered, pending))
    }

    pub fn close(self) {
        let _ = (
            self.w_prefix.close(),
            self.w_iv.close(),
            self.w_b2.close(),
            self.w_b3.close(),
        );
    }
}
