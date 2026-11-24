//! A derived projection's `#[worth_query(name)]` field projects the
//! persistent name its row's root entity carries, in the read that returns
//! the row: no decision reader, no second query, and no result slot.
//!
//! A task row with its name; an `Option` field is `None` for a record that
//! carries no name:
//!
//! ```
//! # use worth_query_decl::facade::application_schema::CanonicalDate;
//! # use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//! # use worth_query_decl::facade::application_schema::ApplicationSchema;
//! # use worth_query_decl::facade::{application_identity::*, portable_identity::*, *};
//! # use worth_query_host::facade::primary_graph::*;
//! # use worth_query_host::facade::WorthQueryProjection;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = NativeScalars, members(Task))]
//! # pub struct Board;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts))]
//! # pub struct Task;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! #     pub title: String,
//! #     pub due: Option<CanonicalDate>,
//! # }
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub RowBinding for NamedTask { identity: "doc.board.named-task" });
//! # worth_query_application_query!(pub Tasks for Schema, identity "doc.board.tasks", parameters NoParameters,
//! #     result RowBinding, scope Task => "Task", name "tasks");
//! use worth_query_decl::facade::application_naming::WorthQueryName;
//!
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task)]
//! pub struct NamedTask {
//!     #[worth_query(field = TaskKey, slot = "doc.board.named-task.key")]
//!     pub key: u64,
//!     #[worth_query(name)]
//!     pub name: WorthQueryName<Task>,
//!     #[worth_query(name)]
//!     pub name_if_any: Option<WorthQueryName<Task>>,
//! }
//!
//! // A read observes names only for a projection that asks for one.
//! assert!(NamedTask::PROJECTS_PERSISTENT_NAMES);
//! assert!(<NamedTask as WorthQueryApplicationProjection<Schema, Tasks>>::PROJECTS_PERSISTENT_NAMES);
//! // The name is the row's own identity, not a declared result slot.
//! let _shape = NamedTask::result_shape::<Tasks>();
//! fn read_name(
//!     row: &WorthQueryApplicationProjectionRow<'_, Schema, Tasks>,
//! ) -> Result<WorthQueryName<Task>, WorthQueryApplicationProjectionDenial> {
//!     row.name::<Task>()
//! }
//! ```
//!
//! A name slot keeps the root's kind: a field typed as another kind's name
//! does not compile, at that field's type:
//!
//! ```compile_fail,E0308
//! # use worth_query_decl::facade::application_schema::CanonicalDate;
//! # use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//! # use worth_query_decl::facade::application_schema::ApplicationSchema;
//! # use worth_query_decl::facade::{application_identity::*, portable_identity::*, *};
//! # use worth_query_host::facade::primary_graph::*;
//! # use worth_query_host::facade::WorthQueryProjection;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = NativeScalars, members(Task))]
//! # pub struct Board;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts))]
//! # pub struct Task;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! #     pub title: String,
//! #     pub due: Option<CanonicalDate>,
//! # }
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects())]
//! # pub struct Project;
//! use worth_query_decl::facade::application_naming::WorthQueryName;
//!
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task)]
//! pub struct NamedTask {
//!     #[worth_query(field = TaskKey, slot = "doc.board.named-task.key")]
//!     pub key: u64,
//!     #[worth_query(name)]
//!     pub name: WorthQueryName<Project>,
//! }
//! ```
//!
//! A name slot is not a result slot, so it takes no slot key:
//!
//! ```compile_fail
//! # use worth_query_decl::facade::application_schema::CanonicalDate;
//! # use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//! # use worth_query_decl::facade::application_schema::ApplicationSchema;
//! # use worth_query_decl::facade::{application_identity::*, portable_identity::*, *};
//! # use worth_query_host::facade::primary_graph::*;
//! # use worth_query_host::facade::WorthQueryProjection;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = NativeScalars, members(Task))]
//! # pub struct Board;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts))]
//! # pub struct Task;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! #     pub title: String,
//! #     pub due: Option<CanonicalDate>,
//! # }
//! use worth_query_decl::facade::application_naming::WorthQueryName;
//!
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task)]
//! pub struct NamedTask {
//!     #[worth_query(name, slot = "doc.board.named-task.name")]
//!     pub name: WorthQueryName<Task>,
//! }
//! ```
