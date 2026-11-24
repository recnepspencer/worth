//! Atomic `ensure_index_ready` lifecycle courts (Blocker 6):
//! 1. An `Evicted` registration stays refused (`NotReady`) even after a second
//!    principal refills the shared Native maintained root.
//! 2. Closing a handle unregisters the association so a fresh `ensure_index_ready`
//!    opens a new `Created` registration and attaches/builds to `Ready`, while a
//!    sibling handle to the closed registration observes `Disposed`.
//! 3. Non-`Created` transitions (`Evicted`, `Failed`) never turn into a silent
//!    `ensure_index_ready` rebuild — even when a subsequent rebuild would succeed.
use super::harness::authenticate_secondary_operator;
use super::operations::AssetSpec;
use super::schema::*;
use super::{install, keys};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{worth_query_identity_namespace, WorthManagedIndex, WorthQuery};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::ordered_index::{
    WorthQueryOrderedIndexDomain, WorthQueryOrderedIndexSources,
};
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

const FAIL_ONCE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.fail_once.v1");

static FAIL_ONCE_BUILD: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FailOnceDenial;
impl std::fmt::Display for FailOnceDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("injected build failure")
    }
}
impl std::error::Error for FailOnceDenial {}

#[derive(WorthManagedIndex)]
#[worth_index(
    schema = ShopFloorSchema,
    record = Asset,
    source = AssetSourceBinding,
    probe = String,
    maintenance = WhenRequired,
    namespace = "hard_dx.fail_once",
    version = 1,
)]
pub(super) struct FailOnceIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = ShopFloorSchema,
    context = ShopContext,
    namespace = FAIL_ONCE_NS,
    scope = FloorId,
    managed_index = FailOnceIndex,
    limits = 64
)]
pub(super) struct FindFailOnce {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub probe: String,
}

pub(super) struct FailOnceDomain;
impl WorthQueryOrderedIndexDomain<ShopFloorSchema, FailOnceIndex> for FailOnceDomain {
    type Summary = ();
    type Denial = FailOnceDenial;
    const IDENTITY: &'static str = "hard_dx.fail_once.v1";
    const VERSION: u32 = 1;

    fn compare_rows(
        &self,
        _left: WorthQueryManagedIndexSourceRow<'_, ShopFloorSchema, FailOnceIndex>,
        _right: WorthQueryManagedIndexSourceRow<'_, ShopFloorSchema, FailOnceIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(Ordering::Equal)
    }

    fn source_summary(
        &self,
        _slot: WorthQueryManagedIndexSourceSlot,
        _row: WorthQueryManagedIndexSourceRow<'_, ShopFloorSchema, FailOnceIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        if FAIL_ONCE_BUILD.load(AtomicOrdering::SeqCst) {
            return Err(WorthQueryManagedIndexProviderStop::Domain(FailOnceDenial));
        }
        Ok(())
    }

    fn combine_summaries(
        &self,
        _left: &Self::Summary,
        _right: &Self::Summary,
        _sources: &WorthQueryOrderedIndexSources<'_, '_, ShopFloorSchema, FailOnceIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(())
    }

    fn summary_may_match(
        &self,
        _summary: &Self::Summary,
        _sources: &WorthQueryOrderedIndexSources<'_, '_, ShopFloorSchema, FailOnceIndex>,
        _probe: &WorthQueryManagedIndexProbeValue<ShopFloorSchema, FailOnceIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(true)
    }

    fn matches_source(
        &self,
        _row: WorthQueryManagedIndexSourceRow<'_, ShopFloorSchema, FailOnceIndex>,
        _probe: &WorthQueryManagedIndexProbeValue<ShopFloorSchema, FailOnceIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(true)
    }
}

fn q_box() -> FindEquipmentInBox2D {
    FindEquipmentInBox2D {
        floor: "floor-1".into(),
        region: QueryBox2D {
            min_x: -20,
            min_y: -20,
            max_x: 20,
            max_y: 20,
        },
    }
}

fn assert_exact_not_ready<R>(res: Result<R, WorthQueryApplicationRequestQueryDenial>) {
    match res {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::NotReady)
            ) => {}
        Err(other) => panic!("expected exact NotReady refusal, got Err({other:?})"),
        Ok(_) => panic!("expected exact NotReady refusal, got Ok"),
    }
}

#[test]
fn evicted_registration_stays_refused_after_second_principal_refills_shared_native_root() {
    let seed = [AssetSpec::i64_only(
        1,
        "CNC-Alpha",
        (10, 30),
        [-5, -5, 0],
        [5, 5, 3],
    )];
    let host = install(&seed, None, None);
    let scope = request_scope();
    let p1 = authenticate_operator(host.installed_schema(), &scope);
    let p2 = authenticate_secondary_operator(host.installed_schema(), &scope);
    let req1 = host.runtime().request(&p1, &scope);
    let req2 = host.runtime().request(&p2, &scope);

    // Principal 1 opens two sibling handles on its registration, then explicitly evicts it.
    let h1 = req1
        .query(q_box())
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    let h1_sibling = req1
        .query(q_box())
        .open_index::<EquipmentBox2DIndex>()
        .unwrap();
    assert_eq!(h1.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    req1.query(q_box()).evict_index(&h1).unwrap();
    assert_eq!(h1.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    assert_eq!(
        h1_sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Evicted
    );

    // Principal 2 calls ensure_index_ready on the same host & refills the shared Native root.
    let h2 = req2
        .query(q_box())
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    assert_eq!(h2.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(keys(req2.query(q_box()).execute().unwrap().rows()), [1]);

    // Principal 1 calls ensure_index_ready: must NOT resurrect its Evicted registration
    // into Ready even though the shared Native maintained root is now populated!
    assert_exact_not_ready(
        req1.query(q_box())
            .ensure_index_ready::<EquipmentBox2DIndex>(),
    );
    assert_eq!(h1.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);

    // Closing h1 disposes the old registration (observed by h1_sibling) and unregisters
    // it from the registry so a fresh ensure_index_ready attaches the Ready Native root!
    h1.close();
    assert_eq!(
        h1_sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Disposed
    );
    let h1_fresh = req1
        .query(q_box())
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    assert_eq!(h1_fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(keys(req1.query(q_box()).execute().unwrap().rows()), [1]);

    drop(h1_sibling);
    h1_fresh.close();
    h2.close();
}

#[test]
fn failed_and_evicted_transitions_never_turn_into_ensure_index_ready_rebuild() {
    let seed = [AssetSpec::i64_only(
        1,
        "CNC-Failed-State",
        (10, 30),
        [-5, -5, 0],
        [5, 5, 3],
    )];
    let host = install(&seed, None, None);
    let scope = request_scope();
    let p1 = authenticate_operator(host.installed_schema(), &scope);
    let req = host.runtime().request(&p1, &scope);

    // 1. Evicted registration on a live host: repeated ensure_index_ready refuses NotReady.
    let live = req
        .query(q_box())
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    req.query(q_box()).evict_index(&live).unwrap();
    assert_eq!(live.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    for _ in 0..2 {
        assert_exact_not_ready(
            req.query(q_box())
                .ensure_index_ready::<EquipmentBox2DIndex>(),
        );
        assert_eq!(live.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    }
    live.close();

    // 2. WhenRequired index: Created -> Rebuilding -> Failed via provider denial,
    //    then disarm failure so an illegal rebuild WOULD succeed: ensure_index_ready
    //    MUST still refuse exact NotReady and preserve Failed; closing unregisters
    //    so a fresh ensure_index_ready transitions Created -> Ready!
    let q = FindFailOnce {
        floor: "floor-1".into(),
        probe: "any".into(),
    };

    FAIL_ONCE_BUILD.store(true, AtomicOrdering::SeqCst);
    let cold = req.query(q.clone()).open_index::<FailOnceIndex>().unwrap();
    assert_eq!(cold.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
    assert!(matches!(
        req.query(q.clone()).rebuild_index(&cold),
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Work(
                    WorthQueryManagedIndexWorkDenial::Provider(_)
                )
            )
    ));
    assert_eq!(cold.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);

    // Disarm the injected failure: if ensure_index_ready illegally rebuilt Failed, it would succeed!
    FAIL_ONCE_BUILD.store(false, AtomicOrdering::SeqCst);
    assert_exact_not_ready(req.query(q.clone()).ensure_index_ready::<FailOnceIndex>());
    assert_eq!(cold.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);

    // Closing unregisters the Failed association; a fresh ensure_index_ready builds to Ready!
    cold.close();
    let fresh = req
        .query(q.clone())
        .ensure_index_ready::<FailOnceIndex>()
        .unwrap();
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(keys(req.query(q).execute().unwrap().rows()), [1]);
    fresh.close();
}
