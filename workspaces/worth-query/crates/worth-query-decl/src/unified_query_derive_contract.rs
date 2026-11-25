//! Compile-time hygiene and nominal-type contract doctests for unified
//! `#[derive(WorthQuery)]` (`where_*`, `*_optional`, `where_no_related`,
//! `where_related_exists`, `live = whole_result`) and `#[derive(WorthQueryRow)]`.
//!
//! `where_optional` / `where_lte_optional` / `where_equal` parameters reject
//! `Deref` wrappers such as `Box<String>` via exact-type nominal witnesses:
//!
//! ```compile_fail,E0308
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
//! };
//!
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.uni.deref.q.v1");
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.uni.deref", name = "S", members(F))]
//! pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = C, members(M, P, Task))]
//! pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C {
//!     type Binding = WorthQueryExternalPrincipalIdentityBinding;
//!     const WRITABLE: bool = false;
//! }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C {
//!     type Binding = WorthQueryPrincipalMappingStatusBinding;
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(TaskFacts), reference(S, F))] pub struct Task;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
//! pub struct TaskRowRec {
//!     #[worth_query(equality, read_only)] pub k: u64,
//!     #[worth_query(equality)] pub state: String,
//! }
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct TaskView { pub k: u64 }
//! impl TaskView {
//!     pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding> {
//!         ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
//!     }
//! }
//! worth_query_structured_value_binding!(pub TaskViewBinding for TaskView { identity: "doc.uni.deref.r.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.uni.deref.den.v1" });
//! pub struct PB;
//! pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.uni.deref.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = TaskViewBinding, root = Task, scope = K, fields = TaskFacts, limits = 8,
//!     where_optional(state = boxed_state),
//! )]
//! pub struct DerefOptionalQuery {
//!     #[worth_query(scope)] pub k: u64,
//!     pub boxed_state: Option<Box<String>>,
//! }
//! ```
//!
//! `live = whole_result` refuses an incompatible `lanes = OneShot` declaration:
//!
//! ```compile_fail
//! use worth_query_decl::facade::WorthQuery;
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = RowB, root = Task, scope = K, limits = 8,
//!     lanes = OneShot, live = whole_result,
//! )]
//! pub struct BadLive {
//!     #[worth_query(scope)] pub k: u64,
//! }
//! ```
//!
//! `where_no_related` with an unknown or malformed clause refuses at compile time:
//!
//! ```compile_fail
//! use worth_query_decl::facade::WorthQuery;
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = RowB, root = Task, scope = K, limits = 8,
//!     where_no_related(),
//! )]
//! pub struct EmptyRelated {
//!     #[worth_query(scope)] pub k: u64,
//! }
//! ```
//!
//! Caller raw identifiers (`r#type`), `const value`, and `__worth_*` constants
//! do not collide with generated query macro hygiene:
//!
//! ```
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
//! };
//!
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.uni.hyg.q.v1");
//! #[allow(dead_code, non_upper_case_globals)]
//! const value: i32 = 7;
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_query_value: &str = "caller shadow";
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_query_clauses: &str = "caller shadow 2";
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.uni.hyg", name = "S", members(F))]
//! pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = C, members(M, P, Task))]
//! pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C {
//!     type Binding = WorthQueryExternalPrincipalIdentityBinding;
//!     const WRITABLE: bool = false;
//! }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C {
//!     type Binding = WorthQueryPrincipalMappingStatusBinding;
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(TaskFacts), reference(S, F))] pub struct Task;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
//! pub struct TaskRowRec {
//!     #[worth_query(equality, read_only)] pub k: u64,
//!     #[worth_query(equality)] pub state: String,
//!     pub due_date: CanonicalDate,
//! }
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct TaskView { pub k: u64 }
//! impl TaskView {
//!     pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding> {
//!         ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
//!     }
//! }
//! worth_query_structured_value_binding!(pub TaskViewBinding for TaskView { identity: "doc.uni.hyg.r.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.uni.hyg.den.v1" });
//! pub struct PB;
//! pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.uni.hyg.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = TaskViewBinding, root = Task, scope = K, fields = TaskFacts, limits = 8,
//!     where_equal(state = "OPEN"),
//!     where_lte_optional(due_date = r#type),
//!     live = whole_result,
//! )]
//! pub struct HygienicLiveQuery {
//!     #[worth_query(scope)] pub k: u64,
//!     pub r#type: Option<CanonicalDate>,
//! }
//! let q = HygienicLiveQuery { k: 1, r#type: Some(CanonicalDate::parse_iso("2026-11-24").unwrap()) };
//! let _ = q.parameters();
//! ```
//!
//! Authored literal predicates reject `Deref` wrappers such as `&Box<String>`
//! at compile time rather than silently coercing:
//!
//! ```compile_fail,E0271
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
//! };
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.uni.lit.q.v1");
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(schema, owner = "doc.uni.lit", name = "S", members(F))] pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(fragment, bindings = C, members(M, P, Task))] pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C { type Binding = WorthQueryExternalPrincipalIdentityBinding; const WRITABLE: bool = false; }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C { type Binding = WorthQueryPrincipalMappingStatusBinding; }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(TaskFacts), reference(S, F))] pub struct Task;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
//! pub struct TaskRowRec { #[worth_query(equality, read_only)] pub k: u64, #[worth_query(equality)] pub state: String }
//! #[derive(Clone, Debug, Eq, PartialEq)] pub struct TaskView { pub k: u64 }
//! impl TaskView {
//!     pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding> {
//!         ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
//!     }
//! }
//! worth_query_structured_value_binding!(pub TaskViewBinding for TaskView { identity: "doc.uni.lit.r.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.uni.lit.den.v1" });
//! pub struct PB; pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.uni.lit.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = TaskViewBinding, root = Task, scope = K, fields = TaskFacts, limits = 8,
//!     where_equal(state = &Box::new(String::from("OPEN"))),
//! )]
//! pub struct BoxedLiteralQuery { #[worth_query(scope)] pub k: u64 }
//! ```
//!
//! Related-record comparison filters enforce nominal `Field::Entity = Target`
//! at compile time:
//!
//! ```compile_fail,E0271
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
//! };
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.uni.rel.q.v1");
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(schema, owner = "doc.uni.rel", name = "S", members(F))] pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(fragment, bindings = C, members(M, P, Task))] pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C { type Binding = WorthQueryExternalPrincipalIdentityBinding; const WRITABLE: bool = false; }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C { type Binding = WorthQueryPrincipalMappingStatusBinding; }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(TaskFacts), relations(BlockedBy), reference(S, F))] pub struct Task;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = Task, from = Task, reference(S, F))] pub struct BlockedBy;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
//! pub struct TaskRowRec { #[worth_query(equality, read_only)] pub k: u64, #[worth_query(equality)] pub prio: u64 }
//! #[derive(Clone, Debug, Eq, PartialEq)] pub struct TaskView { pub k: u64 }
//! impl TaskView {
//!     pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding> {
//!         ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
//!     }
//! }
//! worth_query_structured_value_binding!(pub TaskViewBinding for TaskView { identity: "doc.uni.rel.r.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.uni.rel.den.v1" });
//! pub struct PB; pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.uni.rel.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = TaskViewBinding, root = Task, scope = K, fields = TaskFacts, limits = 8,
//!     where_no_related(BlockedBy, where_gt(PA::pid() = 1)),
//! )]
//! pub struct WrongEntityRelatedFilter { #[worth_query(scope)] pub k: u64 }
//! ```
