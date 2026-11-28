//! The declaration derives expand into the public traits and macros a
//! hand-written declaration uses, so the same compile-time fences hold.
//!
//! Hand-authored inverse undo is unavailable through the declaration facade.
//! The retired derive cannot declare an inverse action:
//!
//! ```compile_fail,E0433
//! struct Restore;
//!
//! #[derive(worth_query_decl::facade::WorthQueryCorrection)]
//! #[worth_query(inverse = Restore)]
//! struct Correction;
//! ```
//!
//! The same derive cannot opt an action into reapplication:
//!
//! ```compile_fail,E0433
//! #[derive(worth_query_decl::facade::WorthQueryCorrection)]
//! #[worth_query(reapply = true)]
//! struct Correction;
//! ```
//!
//! One board declared through `WorthQuerySchema`, `WorthQueryRecord` and
//! `WorthQueryAction`:
//!
//! ```
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{application_identity::*, authentication::*, *};
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! pub struct Schema;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = Codecs, members(Mapping, Principal, Task))]
//! pub struct Board;
//! worth_query_native_scalar_bindings!(pub Codecs);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
//!     type Binding = WorthQueryExternalPrincipalIdentityBinding;
//! }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
//!     type Binding = WorthQueryPrincipalMappingStatusBinding;
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(MappingFacts), relations(Target))]
//! pub struct Mapping;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(PrincipalFacts))]
//! pub struct Principal;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(TaskFacts), reference(Schema, Board))]
//! pub struct Task;
//! #[derive(WorthQuerySchema)]
//! #[worth_query(relation, to = Principal, from = Mapping, reference(Schema, Board))]
//! pub struct Target;
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Board, entity = Mapping)]
//! pub struct MappingRow {
//!     #[worth_query(equality)]
//!     pub identity: WorthQueryExternalPrincipalIdentity,
//!     #[worth_query(equality)]
//!     pub status: WorthQueryPrincipalMappingStatus,
//! }
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = Codecs, aspect = PrincipalFacts, fragment = Board, entity = Principal)]
//! pub struct PrincipalRow {
//!     #[worth_query(equality, read_only)]
//!     pub principal_key: u64,
//! }
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Board, entity = Task)]
//! pub struct TaskRow {
//!     #[worth_query(equality, read_only)]
//!     pub task_key: u64,
//!     pub title: String,
//!     pub due: Option<CanonicalDate>, // named through `application_schema`
//! }
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
//! const RENAME: ApplicationIdentityNamespace =
//!     worth_query_identity_namespace!("doc.board.rename.v1", Binding => "doc.legacy.rename");
//! #[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
//! #[worth_query(schema = Schema, context = Context, namespace = RENAME, input = RenameInput,
//!     operation = RenameTask, binding = RenameBinding, delivery = Renamed, result = Done,
//!     decision = (), reads(Task), validate = titled)]
//! pub struct Rename {
//!     #[worth_query(scope = TaskKey)]
//!     pub task: u64,
//!     pub title: String,
//! }
//! /// Runs when WORTH captures the input, before any effect.
//! fn titled(rename: &Rename) -> Result<(), &'static str> {
//!     (!rename.title.trim().is_empty()).then_some(()).ok_or("a task needs a title")
//! }
//! use worth_query_decl::facade::application_operation::ApplicationMutationBinding;
//! assert_eq!(<RenameBinding as ApplicationMutationBinding<Schema>>::IDENTITY, "doc.legacy.rename");
//! assert_eq!(<RenameBinding as ApplicationMutationBinding<Schema>>::HANDLER_IDENTITY, "doc.board.rename.v1.handler");
//! let blank = Rename { task: 1, title: " ".to_owned() };
//! let denial = RenameInput::validate(&blank).expect_err("a blank title is refused");
//! assert_eq!(denial.reason(), "a task needs a title");
//! assert_eq!(denial.binding_identity().as_str(), "doc.board.rename.v1.input");
//! ```
//!
//! `validate` names a `fn(&Action) -> Result<(), &'static str>`, exactly as
//! a written `worth_query_structured_value_binding!` does; a validator that
//! takes the input by value does not compile:
//!
//! ```compile_fail,E0308
//! # use worth_query_decl::facade::application_schema::{fragment::*, *};
//! # use worth_query_decl::facade::{application_identity::*, authentication::*, *};
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = Codecs, members(Mapping, Principal, Task))]
//! # pub struct Board;
//! # worth_query_native_scalar_bindings!(pub Codecs);
//! # impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs { type Binding = WorthQueryExternalPrincipalIdentityBinding; }
//! # impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs { type Binding = WorthQueryPrincipalMappingStatusBinding; }
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
//! # pub struct MappingRow { #[worth_query(equality)] pub identity: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub status: WorthQueryPrincipalMappingStatus }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = PrincipalFacts, fragment = Board, entity = Principal)]
//! # pub struct PrincipalRow { #[worth_query(equality, read_only)] pub principal_key: u64 }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskRow { #[worth_query(equality, read_only)] pub task_key: u64, pub title: String }
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
//! # const RENAME: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rename.v1");
//! #[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
//! #[worth_query(schema = Schema, context = Context, namespace = RENAME, input = RenameInput,
//!     operation = RenameTask, binding = RenameBinding, result = Done, decision = (),
//!     validate = titled)]
//! pub struct Rename {
//!     #[worth_query(scope = TaskKey)]
//!     pub task: u64,
//!     pub title: String,
//! }
//! fn titled(rename: Rename) -> Result<(), &'static str> {
//!     (!rename.title.is_empty()).then_some(()).ok_or("a task needs a title")
//! }
//! ```
//!
//! An action's scope field carries the scope selection's value, exactly as a
//! written `value` function returns it; `task: String` against the `u64`
//! `TaskKey` column does not compile:
//!
//! ```compile_fail,E0308
//! # use worth_query_decl::facade::application_schema::{fragment::*, *};
//! # use worth_query_decl::facade::{application_identity::*, authentication::*, *};
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = Codecs, members(Mapping, Principal, Task))]
//! # pub struct Board;
//! # worth_query_native_scalar_bindings!(pub Codecs);
//! # impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs { type Binding = WorthQueryExternalPrincipalIdentityBinding; }
//! # impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs { type Binding = WorthQueryPrincipalMappingStatusBinding; }
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
//! # pub struct MappingRow { #[worth_query(equality)] pub identity: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub status: WorthQueryPrincipalMappingStatus }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = PrincipalFacts, fragment = Board, entity = Principal)]
//! # pub struct PrincipalRow { #[worth_query(equality, read_only)] pub principal_key: u64 }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskRow { #[worth_query(equality, read_only)] pub task_key: u64, pub title: String }
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
//! # const RENAME: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rename.v1");
//! #[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
//! #[worth_query(schema = Schema, context = Context, namespace = RENAME, input = RenameInput,
//!     operation = RenameTask, binding = RenameBinding, result = Done, decision = ())]
//! pub struct Rename {
//!     #[worth_query(scope = TaskKey)]
//!     pub task: String,
//!     pub title: String,
//! }
//! ```
//!
//! Every action declares exactly one scope; an action without one is refused
//! at its name with the fix (`missing action scope: mark the field carrying
//! it ..`), before any binding is generated:
//!
//! ```compile_fail
//! use worth_query_decl::facade::WorthQueryAction;
//! #[derive(WorthQueryAction)]
//! #[worth_query(schema = Schema, context = Context, namespace = RENAME, input = RenameInput,
//!     operation = RenameTask, binding = RenameBinding, result = Done, decision = ())]
//! pub struct Rename {
//!     pub task: u64,
//! }
//! ```
//!
//! A record names its binding set; one without it is refused at its name:
//!
//! ```compile_fail
//! use worth_query_decl::facade::WorthQueryRecord;
//! #[derive(WorthQueryRecord)]
//! #[worth_query(aspect = TaskFacts)]
//! pub struct TaskRow {
//!     pub task_key: u64,
//! }
//! ```
//!
//! Derived items keep their declaration's visibility. A private row's aspect
//! and column selections stay private to its module, as written ones would:
//!
//! ```compile_fail,E0603
//! mod board {
//!     use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//!     use worth_query_decl::facade::{WorthQueryRecord, WorthQuerySchema};
//!     #[derive(Clone, Copy, WorthQuerySchema)]
//!     #[worth_query(fragment, bindings = NativeScalars, members(Task))]
//!     struct Board;
//!     #[derive(Clone, Copy, WorthQuerySchema)]
//!     #[worth_query(entity, aspects(TaskFacts))]
//!     struct Task;
//!     #[derive(WorthQueryRecord)]
//!     #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//!     struct TaskRow {
//!         task_key: u64,
//!     }
//! }
//! type Key = board::TaskKey;
//! ```
//!
//! The same declarations made `pub` expose them:
//!
//! ```
//! mod board {
//!     use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//!     use worth_query_decl::facade::{WorthQueryRecord, WorthQuerySchema};
//!     #[derive(Clone, Copy, WorthQuerySchema)]
//!     #[worth_query(fragment, bindings = NativeScalars, members(Task))]
//!     pub struct Board;
//!     #[derive(Clone, Copy, WorthQuerySchema)]
//!     #[worth_query(entity, aspects(TaskFacts))]
//!     pub struct Task;
//!     #[derive(WorthQueryRecord)]
//!     #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//!     pub struct TaskRow {
//!         pub task_key: u64,
//!     }
//! }
//! type Key = board::TaskKey;
//! assert_eq!(Key::at("TaskKey").path(), "TaskKey");
//! ```
