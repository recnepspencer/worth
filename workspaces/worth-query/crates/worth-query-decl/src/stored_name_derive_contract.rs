//! A held persistent name is an ordinary value: `#[derive(WorthQueryRecord)]`
//! stores `WorthQueryName<K>` and `NamedTarget<K>` as columns of any binding
//! set declared with `worth_query_native_scalar_bindings!`, and
//! `#[derive(WorthQueryAction)]` carries them as input fields. Neither needs a
//! reference table, a codec or an attribute on the field.
//!
//! A feature that holds the edge it chamfers, and the action that sets it:
//!
//! ```
//! # use worth_query_decl::facade::application_schema::{fragment::*, *};
//! # use worth_query_decl::facade::{application_identity::*, authentication::*, *};
//!
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = Codecs, members(Mapping, Principal, Task))]
//! # pub struct Board;
//! # worth_query_native_scalar_bindings!(pub Codecs);
//! # impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
//! #     type Binding = WorthQueryExternalPrincipalIdentityBinding;
//! # }
//! # impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
//! #     type Binding = WorthQueryPrincipalMappingStatusBinding;
//! # }
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(MappingFacts), relations(Target))]
//! # pub struct Mapping;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(PrincipalFacts))]
//! # pub struct Principal;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts), reference(Schema, Board))]
//! # pub struct Task;
//! # #[derive(WorthQuerySchema)]
//! # #[worth_query(relation, to = Principal, from = Mapping, reference(Schema, Board))]
//! # pub struct Target;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Board, entity = Mapping)]
//! # pub struct MappingRow {
//! #     #[worth_query(equality)]
//! #     pub identity: WorthQueryExternalPrincipalIdentity,
//! #     #[worth_query(equality)]
//! #     pub status: WorthQueryPrincipalMappingStatus,
//! # }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = PrincipalFacts, fragment = Board, entity = Principal)]
//! # pub struct PrincipalRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub principal_key: u64,
//! # }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! #     pub title: String,
//! #     pub due: Option<CanonicalDate>, // named through `application_schema`
//! # }
//! # pub struct Owner;
//! # impl Owner {
//! #     fn reference() -> ApplicationPrincipalBindingRef<Schema, Self, Mapping, Principal, u64, U64ApplicationValueBinding> {
//! #         ApplicationPrincipalBindingRef::from_requirements("Owner", ApplicationPrincipalBindingRequirements {
//! #             mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(Identity::at("Identity")),
//! #             mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(Status::at("Status")),
//! #             target: ApplicationPrincipalTargetRequirement::from_relation(Target::reference()),
//! #             principal_identity: ApplicationPrincipalIdentityRequirement::from_field(PrincipalKey::at("PrincipalKey")),
//! #         })
//! #     }
//! # }
//! # worth_query_structured_value_binding!(pub Done for () { identity: "doc.board.done" });
//! # worth_query_structured_value_binding!(pub Refused for () { identity: "doc.board.refused" });
//! # pub struct Context;
//! # impl ApplicationBindingContext<Schema> for Context {
//! #     type PrincipalBinding = Owner;
//! #     type Mapping = Mapping;
//! #     type Principal = Principal;
//! #     type PrincipalIdentity = u64;
//! #     type PrincipalIdentityBinding = U64ApplicationValueBinding;
//! #     type IdempotencyKey = u64;
//! #     type DenialBinding = Refused;
//! #     const IDEMPOTENCY_IDENTITY: &'static str = "doc.board.idempotency";
//! #     fn principal_binding() -> ApplicationPrincipalBindingRef<Schema, Owner, Mapping, Principal, u64, U64ApplicationValueBinding> {
//! #         Owner::reference()
//! #     }
//! # }
//! use worth_query_decl::facade::application_naming::{
//!     EntityNameSubject, NamedTarget, Unowned, WorthQueryName, WorthQueryNamedKind,
//! };
//!
//! /// Declares `Task` as a persistent-name kind no entity owns.
//! impl WorthQueryNamedKind for Task {
//!     type Subject = EntityNameSubject;
//!     type OwnerScope = Unowned;
//! }
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(ChamferFacts))]
//! pub struct Chamfer;
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = Codecs, aspect = ChamferFacts, fragment = Board, entity = Chamfer)]
//! pub struct ChamferRow {
//!     #[worth_query(equality, read_only)]
//!     pub chamfer_key: u64,
//!     pub target_task: NamedTarget<Task>,
//!     pub support_task: Option<WorthQueryName<Task>>,
//! }
//!
//! const RETARGET: ApplicationIdentityNamespace =
//!     worth_query_identity_namespace!("doc.board.retarget.v1");
//! #[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
//! #[worth_query(schema = Schema, context = Context, namespace = RETARGET, input = RetargetInput,
//!     operation = RetargetChamfer, binding = RetargetBinding, result = Done, decision = (),
//!     reads(Task))]
//! pub struct Retarget {
//!     #[worth_query(scope = TaskKey)]
//!     pub task: u64,
//!     pub target: NamedTarget<Task>,
//!     pub support: Option<WorthQueryName<Task>>,
//! }
//!
//! fn stores<T: 'static, B: Scalar<T>>() {}
//! stores::<NamedTarget<Task>, Codecs>();
//! stores::<WorthQueryName<Task>, Codecs>();
//! fn records<Row: 'static, B: Record<Row>>() {}
//! records::<ChamferRow, Codecs>();
//! // The derived column selections carry the held types.
//! fn held<Selection: RecordSelection<Value = Held>, Held>() {}
//! held::<TargetTask, NamedTarget<Task>>();
//! held::<SupportTask, WorthQueryName<Task>>();
//! ```
//!
//! A kind that declares no persistent-name subject has no stored form, so a
//! column holding its names does not compile:
//!
//! ```compile_fail,E0277
//! use worth_query_decl::facade::application_naming::WorthQueryName;
//! use worth_query_decl::facade::application_schema::fragment::*;
//! use worth_query_decl::facade::*;
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = NativeScalars, members(Task))]
//! pub struct Board;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(TaskFacts))]
//! pub struct Task;
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! pub struct TaskRow {
//!     #[worth_query(equality, read_only)]
//!     pub task_key: u64,
//!     pub blocked_by: WorthQueryName<Task>,
//! }
//! ```
//!
//! A stored name keeps its kind: a column of one kind does not accept a name
//! of another.
//!
//! ```compile_fail,E0308
//! use worth_query_decl::facade::application_naming::{NamedTarget, WorthQueryName};
//!
//! struct Face;
//! struct Edge;
//! struct ChamferRow {
//!     target: NamedTarget<Edge>,
//! }
//! fn misuse(face: WorthQueryName<Face>) -> ChamferRow {
//!     ChamferRow { target: NamedTarget::exact(face) }
//! }
//! ```
