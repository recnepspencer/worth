//! Compile-time nominal literal type-identity contracts for `#[derive(WorthQuery)]`.
//!
//! Literal predicates infer the authored literal expression independently and
//! enforce exact type identity with the selected field's `Value` type (never
//! silently deref-coercing `&Box<T>` into `&T`).
//!
//! A boxed `String` literal (`&Box::new(String::from("OPEN"))`) is rejected on a
//! `String`-valued field:
//!
//! ```compile_fail,E0277
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
//! Control fixture: an exact `&'static str` literal compiles cleanly against a
//! `&'static str`-valued field using a complete `ApplicationScalarValueBinding`:
//!
//! ```
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::typed::ScalarAspectType;
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
//! };
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.uni.strref.ok.q.v1");
//! pub struct StaticStrBinding;
//! impl ApplicationScalarValueBinding for StaticStrBinding {
//!     type Value = &'static str;
//!     type Unit = ();
//!     type Decode = ApplicationValueDecodeAvailable;
//!     type Identity = ApplicationValueIsIdentity;
//!     type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
//!     const IDENTITY_NAME: &'static str = "doc.uni.strref.ok.str.v1";
//!     const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
//!     fn validate(_: &&'static str) -> Result<(), ApplicationValueValidationDenial> { Ok(()) }
//!     fn encode(v: &&'static str) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
//!         StringApplicationValueBinding::encode(&(*v).to_owned())
//!     }
//! }
//! impl ApplicationReadableScalarValueBinding for StaticStrBinding {
//!     fn decode(_: &ApplicationValue) -> Result<&'static str, ApplicationValueDecodeDenial> { Ok("OPEN") }
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(schema, owner = "doc.uni.strref.ok", name = "S", members(F))] pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(fragment, bindings = C, members(M, P, Task))] pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C { type Binding = WorthQueryExternalPrincipalIdentityBinding; const WRITABLE: bool = false; }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C { type Binding = WorthQueryPrincipalMappingStatusBinding; }
//! impl Scalar<&'static str> for C { type Binding = StaticStrBinding; }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(TaskFacts), reference(S, F))] pub struct Task;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
//! pub struct TaskRowRec { #[worth_query(equality, read_only)] pub k: u64, #[worth_query(equality)] pub code: &'static str }
//! #[derive(Clone, Debug, Eq, PartialEq)] pub struct TaskView { pub k: u64 }
//! impl TaskView {
//!     pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding> {
//!         ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
//!     }
//! }
//! worth_query_structured_value_binding!(pub TaskViewBinding for TaskView { identity: "doc.uni.strref.ok.r.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.uni.strref.ok.den.v1" });
//! pub struct PB; pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.uni.strref.ok.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! const OPEN_CODE: &'static str = "OPEN";
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = TaskViewBinding, root = Task, scope = K, fields = TaskFacts, limits = 8,
//!     where_equal(code = OPEN_CODE),
//! )]
//! pub struct ExactStrRefLiteralQuery { #[worth_query(scope)] pub k: u64 }
//! let _def = ExactStrRefLiteralQueryQuery::definition();
//! ```
//!
//! Using the exact same `StaticStrBinding` fixture, `&Box::new("OPEN")` is
//! inferred independently without deref coercion and fails strictly at the
//! `E0277` exact-type-identity gate:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::typed::ScalarAspectType;
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
//! };
//! const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.uni.strref.q.v1");
//! pub struct StaticStrBinding;
//! impl ApplicationScalarValueBinding for StaticStrBinding {
//!     type Value = &'static str;
//!     type Unit = ();
//!     type Decode = ApplicationValueDecodeAvailable;
//!     type Identity = ApplicationValueIsIdentity;
//!     type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
//!     const IDENTITY_NAME: &'static str = "doc.uni.strref.str.v1";
//!     const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
//!     fn validate(_: &&'static str) -> Result<(), ApplicationValueValidationDenial> { Ok(()) }
//!     fn encode(v: &&'static str) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
//!         StringApplicationValueBinding::encode(&(*v).to_owned())
//!     }
//! }
//! impl ApplicationReadableScalarValueBinding for StaticStrBinding {
//!     fn decode(_: &ApplicationValue) -> Result<&'static str, ApplicationValueDecodeDenial> { Ok("OPEN") }
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(schema, owner = "doc.uni.strref", name = "S", members(F))] pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(fragment, bindings = C, members(M, P, Task))] pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C { type Binding = WorthQueryExternalPrincipalIdentityBinding; const WRITABLE: bool = false; }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C { type Binding = WorthQueryPrincipalMappingStatusBinding; }
//! impl Scalar<&'static str> for C { type Binding = StaticStrBinding; }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(TaskFacts), reference(S, F))] pub struct Task;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
//! pub struct TaskRowRec { #[worth_query(equality, read_only)] pub k: u64, #[worth_query(equality)] pub code: &'static str }
//! #[derive(Clone, Debug, Eq, PartialEq)] pub struct TaskView { pub k: u64 }
//! impl TaskView {
//!     pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding> {
//!         ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
//!     }
//! }
//! worth_query_structured_value_binding!(pub TaskViewBinding for TaskView { identity: "doc.uni.strref.r.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.uni.strref.den.v1" });
//! pub struct PB; pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.uni.strref.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(
//!     schema = S, context = Ctx, namespace = Q_NS,
//!     result = TaskViewBinding, root = Task, scope = K, fields = TaskFacts, limits = 8,
//!     where_equal(code = &Box::new("OPEN")),
//! )]
//! pub struct DerefStrRefLiteralQuery { #[worth_query(scope)] pub k: u64 }
//! ```
