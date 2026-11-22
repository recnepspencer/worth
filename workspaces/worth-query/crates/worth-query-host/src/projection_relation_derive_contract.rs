//! A derived projection traverses relations with the public relation
//! selectors and nested result-shape builders a written projection uses.
//! `relation = Relation` leaves the root along a relation it declares;
//! `reverse_from = Entity` arrives at the root from the relation's declaring
//! entity. The field type selects the traversal: `T` exactly one related row,
//! `Option<T>` at most one and `Vec<T>` many, where `T` is itself a derived
//! projection rooted at the related entity. `nested = Binding` identifies the
//! nested result shape. `selector = name` renames a slot's generated selector,
//! so a row may keep an inherent method of the field's name.
//!
//! ```
//! # use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//! # use worth_query_decl::facade::{application_identity::*, *};
//! # use worth_query_host::facade::primary_graph::*;
//! # use worth_query_host::facade::WorthQueryProjection;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = NativeScalars, members(Project, Task))]
//! # pub struct Board;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(ProjectFacts), relations(Contains))]
//! # pub struct Project;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts))]
//! # pub struct Task;
//! # #[derive(WorthQuerySchema)]
//! # #[worth_query(relation, to = Task)]
//! # pub struct Contains;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = ProjectFacts, fragment = Board, entity = Project)]
//! # pub struct ProjectFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub project_key: u64,
//! # }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! # }
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub Nested for () { identity: "doc.board.nested" });
//! # worth_query_structured_value_binding!(pub RowBinding for ProjectRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Projects for Schema, identity "doc.board.projects",
//! #     parameters NoParameters, result RowBinding, scope Project => "Project", name "projects");
//! # /// Holds when `ProjectRow` projects `Q`'s result rows.
//! # fn projects<Q>() where ProjectRow: WorthQueryApplicationProjection<Schema, Q> {}
//! # #[derive(Clone, Debug, WorthQueryProjection)]
//! # #[worth_query(schema = Schema, root = Project, namespace = ROWS)]
//! # pub struct ProjectKeyRow {
//! #     #[worth_query(field = ProjectKey, name = "project_key")]
//! #     pub key: u64,
//! # }
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Project, namespace = ROWS)]
//! pub struct ProjectRow {
//!     #[worth_query(field = ProjectKey)]
//!     pub key: u64,
//!     #[worth_query(relation = Contains, nested = Nested)]
//!     pub tasks: Vec<TaskRow>,
//! }
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task, namespace = ROWS)]
//! pub struct TaskRow {
//!     #[worth_query(field = TaskKey, name = "task_key", selector = task_key)]
//!     id: u64,
//!     #[worth_query(relation = Contains, reverse_from = Project, nested = Nested,
//!         slot = "doc.legacy.project")]
//!     pub project: Option<ProjectKeyRow>,
//! }
//! impl TaskRow {
//!     pub const fn id(&self) -> u64 {
//!         self.id
//!     }
//! }
//! projects::<Projects>();
//! let _shape = ProjectRow::result_shape::<Projects>();
//! assert_eq!(ProjectRow::tasks::<Projects>().slot_type(), "doc.board.rows.v1.slot.tasks");
//! assert_eq!(TaskRow::task_key::<Projects>().slot_type(), "doc.board.rows.v1.slot.task_key");
//! assert_eq!(TaskRow::project::<Projects>().slot_type(), "doc.legacy.project");
//! fn read_tasks(
//!     row: &WorthQueryApplicationProjectionRow<'_, Schema, Projects>,
//! ) -> Result<Vec<TaskRow>, WorthQueryApplicationProjectionDenial> {
//!     ProjectRow::project_row(row).map(|project| project.tasks)
//! }
//! ```
//!
//! Without `selector`, the generated `fn id<Q>()` selector collides with the
//! row's own `fn id(&self)`:
//!
//! ```compile_fail,E0592
//! # use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//! # use worth_query_decl::facade::{application_identity::*, *};
//! # use worth_query_host::facade::primary_graph::*;
//! # use worth_query_host::facade::WorthQueryProjection;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = NativeScalars, members(Project, Task))]
//! # pub struct Board;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(ProjectFacts), relations(Contains))]
//! # pub struct Project;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts))]
//! # pub struct Task;
//! # #[derive(WorthQuerySchema)]
//! # #[worth_query(relation, to = Task)]
//! # pub struct Contains;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = ProjectFacts, fragment = Board, entity = Project)]
//! # pub struct ProjectFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub project_key: u64,
//! # }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! # }
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub Nested for () { identity: "doc.board.nested" });
//! # worth_query_structured_value_binding!(pub RowBinding for ProjectRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Projects for Schema, identity "doc.board.projects",
//! #     parameters NoParameters, result RowBinding, scope Project => "Project", name "projects");
//! # /// Holds when `ProjectRow` projects `Q`'s result rows.
//! # fn projects<Q>() where ProjectRow: WorthQueryApplicationProjection<Schema, Q> {}
//! # #[derive(Clone, Debug, WorthQueryProjection)]
//! # #[worth_query(schema = Schema, root = Project, namespace = ROWS)]
//! # pub struct ProjectKeyRow {
//! #     #[worth_query(field = ProjectKey, name = "project_key")]
//! #     pub key: u64,
//! # }
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Project, namespace = ROWS)]
//! pub struct ProjectRow {
//!     #[worth_query(field = ProjectKey)]
//!     pub key: u64,
//!     #[worth_query(relation = Contains, nested = Nested)]
//!     pub tasks: Vec<TaskRow>,
//! }
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Task, namespace = ROWS)]
//! pub struct TaskRow {
//!     #[worth_query(field = TaskKey, name = "task_key")]
//!     id: u64,
//!     #[worth_query(relation = Contains, reverse_from = Project, nested = Nested,
//!         slot = "doc.legacy.project")]
//!     pub project: Option<ProjectKeyRow>,
//! }
//! impl TaskRow {
//!     pub const fn id(&self) -> u64 {
//!         self.id
//!     }
//! }
//! ```
//!
//! A related row must be rooted where the traversal arrives: `Contains`
//! leads from a project to tasks, so project rows cannot project its targets.
//!
//! ```compile_fail,E0308
//! # use worth_query_decl::facade::application_schema::fragment::NativeScalars;
//! # use worth_query_decl::facade::{application_identity::*, *};
//! # use worth_query_host::facade::primary_graph::*;
//! # use worth_query_host::facade::WorthQueryProjection;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(schema, owner = "doc.board", name = "board", members(Board))]
//! # pub struct Schema;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(fragment, bindings = NativeScalars, members(Project, Task))]
//! # pub struct Board;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(ProjectFacts), relations(Contains))]
//! # pub struct Project;
//! # #[derive(Clone, Copy, WorthQuerySchema)]
//! # #[worth_query(entity, aspects(TaskFacts))]
//! # pub struct Task;
//! # #[derive(WorthQuerySchema)]
//! # #[worth_query(relation, to = Task)]
//! # pub struct Contains;
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = ProjectFacts, fragment = Board, entity = Project)]
//! # pub struct ProjectFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub project_key: u64,
//! # }
//! # #[derive(WorthQueryRecord)]
//! # #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! # pub struct TaskFactsRow {
//! #     #[worth_query(equality, read_only)]
//! #     pub task_key: u64,
//! # }
//! # const ROWS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.board.rows.v1");
//! # worth_query_structured_value_binding!(pub NoParameters for () { identity: "doc.board.parameters" });
//! # worth_query_structured_value_binding!(pub Nested for () { identity: "doc.board.nested" });
//! # worth_query_structured_value_binding!(pub RowBinding for ProjectRow { identity: "doc.board.row" });
//! # worth_query_application_query!(pub Projects for Schema, identity "doc.board.projects",
//! #     parameters NoParameters, result RowBinding, scope Project => "Project", name "projects");
//! # /// Holds when `ProjectRow` projects `Q`'s result rows.
//! # fn projects<Q>() where ProjectRow: WorthQueryApplicationProjection<Schema, Q> {}
//! # #[derive(Clone, Debug, WorthQueryProjection)]
//! # #[worth_query(schema = Schema, root = Project, namespace = ROWS)]
//! # pub struct ProjectKeyRow {
//! #     #[worth_query(field = ProjectKey, name = "project_key")]
//! #     pub key: u64,
//! # }
//! #[derive(Clone, Debug, WorthQueryProjection)]
//! #[worth_query(schema = Schema, root = Project, namespace = ROWS)]
//! pub struct ProjectRow {
//!     #[worth_query(field = ProjectKey)]
//!     pub key: u64,
//!     #[worth_query(relation = Contains, nested = Nested)]
//!     pub tasks: Vec<ProjectKeyRow>,
//! }
//! ```
