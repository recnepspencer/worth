//! End-to-end Store M4.4 courts verifying:
//! 1. Cross-substitution refusal for requests sharing the SAME graph contract
//!    and SAME snapshot with different parameters OR different scopes.
//! 2. Incremental `streaming_window_bytes` enforcement DURING intermediate tree
//!    materialization (aborting before full construction of an oversized root).
//! 3. Budget enforcement on the final projected typed `Row` (refusing an
//!    application projector that expands a tiny stored row past `streaming_window_bytes`).

use std::fmt::Write as _;
use std::num::NonZeroUsize;

use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_query::{
    ApplicationQueryMarkerIdentity, TypedApplicationQueryResultShape,
};
use worth_query_decl::facade::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};
use worth_query_decl::facade::{worth_query_identity_namespace, WorthQuery};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    historical_cursor::{
        AdmittedQueryWindowBinding, AdmittedQueryWindowPage, AdmittedQueryWindowSource,
        CancellationSource, CanonicalKeyDomain, ExecutionRequest, HistoricalBasisRetentionKind,
        HistoricalQueryContract, HistoricalReadDenial, KnownProjectedHeapBytes,
        SchemaVersionIdentity, SerialMemoryBudget, SerialRequest, PROJECTION_QUERY_ROWS,
    },
    primary_graph::*,
    WorthQuerySessionQuery,
};

use super::courts::temp_home_path;
use super::program::{owner, DaymarkProgram};
use super::queries::{BacklogTasks, ReadyTasks};
use super::retained_basis_cursor_security_court::{
    budget_with_window, open_custom_daymark, security_schema_version,
};
use super::schema::*;

struct OneShotPageSource<Row>(Option<WorthQueryApplicationRootWindowPage<Row>>);

impl<Row: KnownProjectedHeapBytes + Send> AdmittedQueryWindowSource<Row>
    for OneShotPageSource<Row>
{
    fn read_window(
        &mut self,
        _execution: &ExecutionRequest<'_, '_>,
        _window: WorthQueryApplicationRootWindow,
    ) -> Result<AdmittedQueryWindowPage<Row>, HistoricalReadDenial> {
        AdmittedQueryWindowPage::from_executed_page(self.0.take().expect("one-shot page"))
    }
}

fn expected_contract<Intent: WorthQuerySessionQuery<DaymarkSchema> + Clone>(
    session: &worth_query_host::facade::WorthQueryHostSession<'_, DaymarkSchema, DaymarkProgram>,
    intent: &Intent,
    schema: &SchemaVersionIdentity,
) -> HistoricalQueryContract {
    let digest = intent.derive_historical_query_digest(session).unwrap();
    let mut hex = String::with_capacity(64);
    for b in digest {
        let _ = write!(&mut hex, "{b:02x}");
    }
    HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        None,
        PROJECTION_QUERY_ROWS,
        format!("query.binding.v3:{hex}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assert_cross_substitution_refused<Intent>(
    home: &super::program::DaymarkHome,
    store: &worth_query_host::facade::historical_cursor::DurableHistoricalBasisStore,
    session: &worth_query_host::facade::WorthQueryHostSession<'_, DaymarkSchema, DaymarkProgram>,
    exec: &ExecutionRequest<'_, '_>,
    receipt: &worth_query_host::facade::historical_cursor::RetainedRevisionReceipt,
    schema: &SchemaVersionIdentity,
    budget: worth_query_host::facade::historical_cursor::ResidencyBudget,
    req_a: Intent,
    req_b: Intent,
    page_limit: usize,
) where
    Intent: WorthQuerySessionQuery<DaymarkSchema> + Clone + Send + 'static,
    Intent::Row: Clone + std::fmt::Debug + KnownProjectedHeapBytes + Send + 'static,
{
    assert_eq!(
        req_a.installed_contract_digest(session).unwrap(),
        req_b.installed_contract_digest(session).unwrap(),
        "both requests share the exact same installed graph contract digest"
    );
    assert_ne!(
        req_a.admitted_request_binding_digest(session).unwrap(),
        req_b.admitted_request_binding_digest(session).unwrap(),
        "distinct parameters or scopes must yield distinct request-binding digests"
    );
    let width = NonZeroUsize::new(page_limit).unwrap();
    let wrong_p1 = req_b
        .clone()
        .execute_window_in_session(session, None, WorthQueryApplicationRootWindow::first(width))
        .unwrap()
        .into_root_window_page()
        .unwrap();
    let wrong_p2 = req_b
        .execute_window_in_session(session, None, WorthQueryApplicationRootWindow::first(width))
        .unwrap()
        .into_root_window_page()
        .unwrap();
    let binding = AdmittedQueryWindowBinding {
        contract_digest: req_a.installed_contract_digest(session).unwrap(),
        request_binding_digest: req_a.admitted_request_binding_digest(session).unwrap(),
        snapshot_version: wrong_p1.progress().snapshot_version(),
    };
    let contract = expected_contract(session, &req_a, schema);

    let mut cur = store
        .open_admitted_query_cursor(
            exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            contract.clone(),
            binding,
            schema,
            |_| Ok(Box::new(OneShotPageSource(Some(wrong_p1)))),
        )
        .unwrap();
    assert!(matches!(
        cur.next_page(exec, page_limit).unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    drop(cur);

    let mut legit = home
        .open_historical_query_cursor(
            session,
            exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            req_a,
            schema,
            budget,
        )
        .unwrap();
    let tok = legit
        .next_page(exec, page_limit)
        .unwrap()
        .checkpoint_token()
        .clone();
    drop(legit);

    let mut res = store
        .resume_admitted_query_cursor(exec, &tok, contract, binding, schema, |_| {
            Ok(Box::new(OneShotPageSource(Some(wrong_p2))))
        })
        .unwrap();
    assert!(matches!(
        res.next_page(exec, page_limit).unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
}

#[test]
fn retained_basis_cursor_rejects_cross_substituted_parameter_and_scope_receipts() {
    let path = temp_home_path("retained-basis-param-scope-binding");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let tasks: Vec<(u64, String)> = (100..106).map(|k| (k, format!("Task-{k}"))).collect();

    let home = open_custom_daymark(&path, &clock, tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let receipt = home
        .retain_historical_basis(
            &session,
            "same-snap",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();

    assert_cross_substitution_refused(
        &home,
        &store,
        &session,
        &exec,
        &receipt,
        &schema,
        budget,
        BacklogTasks { project: 2 },
        BacklogTasks { project: 1 },
        2,
    );
    assert_cross_substitution_refused(
        &home,
        &store,
        &session,
        &exec,
        &receipt,
        &schema,
        budget,
        ReadyTasks {
            project: 2,
            priority: Some(1),
            due_before: None,
        },
        ReadyTasks {
            project: 2,
            priority: Some(2),
            due_before: None,
        },
        1,
    );

    drop((store, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

const EXPANDING_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.expanding-tasks.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandingRow {
    pub task_key: u64,
    pub expanded: String,
}

impl KnownProjectedHeapBytes for ExpandingRow {
    fn known_heap_bytes(&self) -> usize {
        self.task_key
            .known_heap_bytes()
            .saturating_add(self.expanded.known_heap_bytes())
    }
}

impl ApplicationStructuredValueBinding for ExpandingRow {
    type Value = Self;
    const IDENTITY_NAME: &'static str = "daymark.unified.expanding-row.v1:result";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

impl ExpandingRow {
    pub fn result_shape<Q: ApplicationQueryMarkerIdentity<DaymarkSchema>>(
    ) -> TypedApplicationQueryResultShape<DaymarkSchema, Q, Task, Self, Self> {
        TaskRow::result_shape_builder::<Q, Self>().build()
    }
}

impl<Query: ApplicationQueryMarkerIdentity<DaymarkSchema>>
    WorthQueryApplicationProjection<DaymarkSchema, Query> for ExpandingRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DaymarkSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let task_key = row.field(TaskRow::task_key::<Query>())?;
        let title = row.field(TaskRow::title::<Query>())?;
        Ok(Self {
            task_key,
            expanded: format!("{title}-{}", "E".repeat(4096)),
        })
    }
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = EXPANDING_NS,
    result = ExpandingRow,
    root = Task,
    scope = ProjectKey,
    root_path(Contains),
    order_by(TaskRow::task_key(), Ascending),
    limits = 8,
)]
pub struct ExpandingTasks {
    #[worth_query(scope)]
    pub project: u64,
}

#[test]
fn retained_basis_cursor_enforces_streaming_window_during_materialization_and_expanding_projection()
{
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);

    let path = temp_home_path("retained-basis-mid-tree-budget");
    let home_huge = open_custom_daymark(
        &path,
        &clock,
        vec![(100, "M".repeat(32 * 1024)), (101, "small".into())],
    );
    let me = owner(&home_huge, &clock);
    let session = home_huge.session(&me);
    let tight = budget_with_window(512);
    let receipt = home_huge
        .retain_historical_basis(
            &session,
            "oversized-tree",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            tight,
        )
        .unwrap();
    let mut cur = home_huge
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            BacklogTasks { project: 2 },
            &schema,
            tight,
        )
        .unwrap();
    assert!(matches!(
        cur.next_page(&exec, 2).unwrap_err(),
        HistoricalReadDenial::MemoryExhausted { requested_bytes, allowance_bytes: 512 } if requested_bytes >= 32 * 1024
    ));
    drop((cur, session, me));
    home_huge.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(&path);

    let path2 = temp_home_path("retained-basis-expanding-projector");
    let home_exp = open_custom_daymark(
        &path2,
        &clock,
        vec![(200, "tiny".into()), (201, "tiny2".into())],
    );
    let me2 = owner(&home_exp, &clock);
    let session2 = home_exp.session(&me2);
    let win_1024 = budget_with_window(1024);
    let exp_receipt = home_exp
        .retain_historical_basis(
            &session2,
            "expanding-proj",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            win_1024,
        )
        .unwrap();
    let mut exp_cur = home_exp
        .open_historical_query_cursor(
            &session2,
            &exec,
            exp_receipt.commit_id,
            exp_receipt.root_semantic_commitment,
            ExpandingTasks { project: 2 },
            &schema,
            win_1024,
        )
        .unwrap();
    let exp_err = exp_cur.next_page(&exec, 2).unwrap_err();
    assert!(
        matches!(exp_err, HistoricalReadDenial::MemoryExhausted { requested_bytes, allowance_bytes: 1024 } if requested_bytes >= 4096),
        "expanding projector must refuse MemoryExhausted with >=4096 B, got {exp_err:?}"
    );
    drop((exp_cur, session2, me2));
    home_exp.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(&path2);
}
