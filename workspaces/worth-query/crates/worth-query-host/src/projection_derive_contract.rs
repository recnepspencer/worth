//! A derived projection expands into slot markers, their portable
//! identities, query-bound selectors, a result shape and
//! `WorthQueryApplicationProjection`, so it keeps every fence of a written one.
//! Its expansion names `::worth_query_decl` and `::worth_query_host`; a crate
//! deriving it depends on both.
//!
//! One row projected by every query whose result binding carries it, with a
//! kept wire identity on one slot:
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
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Tasks for Schema, identity "doc.board.tasks", parameters NoParameters,
//! #     result RowBinding, scope Task => "Task", name "tasks");
//! # pub struct OtherRow;
//! # worth_query_structured_value_binding!(pub OtherBinding for OtherRow { identity: "doc.board.other" });
//! # worth_query_application_query!(pub Others for Schema, identity "doc.board.others", parameters NoParameters,
//! #     result OtherBinding, scope Task => "Task", name "others");
//! # /// Holds when `TaskRow` projects `Q`'s result rows.
//! # fn projects<Q>() where TaskRow: WorthQueryApplicationProjection<Schema, Q> {}
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task, namespace = ROWS)]
//! pub struct TaskRow {
//!     #[worth_query(field = TaskKey)]
//!     pub key: u64,
//!     #[worth_query(field = Title, slot = "doc.legacy.title")]
//!     pub title: String,
//!     #[worth_query(field = Due)]
//!     pub due: Option<CanonicalDate>,
//! }
//! projects::<Tasks>();
//! let _shape = TaskRow::result_shape::<Tasks>();
//! assert_eq!(TaskRow::key::<Tasks>().slot_type(), "doc.board.rows.v1.slot.key");
//! assert_eq!(
//!     <task_row_slots::Title as WorthQueryPortableType>::PORTABLE_TYPE_NAME,
//!     "doc.legacy.title"
//! );
//! fn read_key(
//!     row: &WorthQueryApplicationProjectionRow<'_, Schema, Tasks>,
//! ) -> Result<u64, WorthQueryApplicationProjectionDenial> {
//!     row.field(TaskRow::key::<Tasks>())
//! }
//! ```
//!
//! A field's type is its selection's value type; `title: u64` against the
//! `String` column does not compile, at that field's type:
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
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Tasks for Schema, identity "doc.board.tasks", parameters NoParameters,
//! #     result RowBinding, scope Task => "Task", name "tasks");
//! # pub struct OtherRow;
//! # worth_query_structured_value_binding!(pub OtherBinding for OtherRow { identity: "doc.board.other" });
//! # worth_query_application_query!(pub Others for Schema, identity "doc.board.others", parameters NoParameters,
//! #     result OtherBinding, scope Task => "Task", name "others");
//! # /// Holds when `TaskRow` projects `Q`'s result rows.
//! # fn projects<Q>() where TaskRow: WorthQueryApplicationProjection<Schema, Q> {}
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task, namespace = ROWS)]
//! pub struct TaskRow {
//!     #[worth_query(field = TaskKey)]
//!     pub key: u64,
//!     #[worth_query(field = Title, slot = "doc.legacy.title")]
//!     pub title: u64,
//!     #[worth_query(field = Due)]
//!     pub due: Option<CanonicalDate>,
//! }
//! ```
//!
//! A row projects only the queries whose result binding carries it, never the
//! rows of another query:
//!
//! ```compile_fail,E0271
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
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Tasks for Schema, identity "doc.board.tasks", parameters NoParameters,
//! #     result RowBinding, scope Task => "Task", name "tasks");
//! # pub struct OtherRow;
//! # worth_query_structured_value_binding!(pub OtherBinding for OtherRow { identity: "doc.board.other" });
//! # worth_query_application_query!(pub Others for Schema, identity "doc.board.others", parameters NoParameters,
//! #     result OtherBinding, scope Task => "Task", name "others");
//! # /// Holds when `TaskRow` projects `Q`'s result rows.
//! # fn projects<Q>() where TaskRow: WorthQueryApplicationProjection<Schema, Q> {}
//! # #[derive(Clone, Debug, WorthQueryProjection)]
//! # #[worth_query(schema = Schema, root = Task, namespace = ROWS)]
//! # pub struct TaskRow {
//! #     #[worth_query(field = TaskKey)]
//! #     pub key: u64,
//! #     #[worth_query(field = Title, slot = "doc.legacy.title")]
//! #     pub title: String,
//! #     #[worth_query(field = Due)]
//! #     pub due: Option<CanonicalDate>,
//! # }
//! projects::<Others>();
//! ```
//!
//! Its selectors are bound to one query, like written selectors: another
//! query's row cannot read them.
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
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Tasks for Schema, identity "doc.board.tasks", parameters NoParameters,
//! #     result RowBinding, scope Task => "Task", name "tasks");
//! # pub struct OtherRow;
//! # worth_query_structured_value_binding!(pub OtherBinding for OtherRow { identity: "doc.board.other" });
//! # worth_query_application_query!(pub Others for Schema, identity "doc.board.others", parameters NoParameters,
//! #     result OtherBinding, scope Task => "Task", name "others");
//! # /// Holds when `TaskRow` projects `Q`'s result rows.
//! # fn projects<Q>() where TaskRow: WorthQueryApplicationProjection<Schema, Q> {}
//! # #[derive(Clone, Debug, WorthQueryProjection)]
//! # #[worth_query(schema = Schema, root = Task, namespace = ROWS)]
//! # pub struct TaskRow {
//! #     #[worth_query(field = TaskKey)]
//! #     pub key: u64,
//! #     #[worth_query(field = Title, slot = "doc.legacy.title")]
//! #     pub title: String,
//! #     #[worth_query(field = Due)]
//! #     pub due: Option<CanonicalDate>,
//! # }
//! fn read_key(
//!     row: &WorthQueryApplicationProjectionRow<'_, Schema, Others>,
//! ) -> Result<u64, WorthQueryApplicationProjectionDenial> {
//!     row.field(TaskRow::key::<Tasks>())
//! }
//! ```
//!
//! Every slot has an identity: without a `namespace` on the row, a field
//! without `slot = ".."` is refused at that field.
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
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Tasks for Schema, identity "doc.board.tasks", parameters NoParameters,
//! #     result RowBinding, scope Task => "Task", name "tasks");
//! # pub struct OtherRow;
//! # worth_query_structured_value_binding!(pub OtherBinding for OtherRow { identity: "doc.board.other" });
//! # worth_query_application_query!(pub Others for Schema, identity "doc.board.others", parameters NoParameters,
//! #     result OtherBinding, scope Task => "Task", name "others");
//! # /// Holds when `TaskRow` projects `Q`'s result rows.
//! # fn projects<Q>() where TaskRow: WorthQueryApplicationProjection<Schema, Q> {}
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task)]
//! pub struct TaskRow {
//!     #[worth_query(field = TaskKey)]
//!     pub key: u64,
//!     #[worth_query(field = Title, slot = "doc.legacy.title")]
//!     pub title: String,
//!     #[worth_query(field = Due)]
//!     pub due: Option<CanonicalDate>,
//! }
//! ```
