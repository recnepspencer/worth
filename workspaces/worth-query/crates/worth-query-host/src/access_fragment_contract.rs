//! Reusable operation access through the host facade: a fragment declared
//! once with `worth_query_access_fragment!` and re-exported from its module is
//! included by path in `worth_query_operation!`'s `access` block (and in
//! `#[derive(WorthQueryAction)]`'s `access(..)`). Fragments compose, may
//! include one another, and a field read or write they repeat installs once.
//!
//! ```
//! use worth_query_decl::facade::application_schema::fragment::{NativeScalars, Registration};
//! use worth_query_decl::facade::application_schema::*;
//! use worth_query_decl::facade::{WorthQueryRecord, WorthQuerySchema};
//! use worth_query_host::facade::{
//!     worth_query_access_fragment, worth_query_operation, worth_query_structured_value_binding,
//! };
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.fragment", name = "fragment", members(Board, Contracts))]
//! pub struct Schema;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = NativeScalars, members(Task, Note))]
//! pub struct Board;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(TaskFacts))]
//! pub struct Task;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(entity, aspects(NoteFacts))]
//! pub struct Note;
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = NativeScalars, aspect = TaskFacts, fragment = Board, entity = Task)]
//! pub struct TaskRow {
//!     #[worth_query(equality, read_only)]
//!     pub task_key: u64,
//!     pub title: String,
//! }
//! #[derive(WorthQueryRecord)]
//! #[worth_query(bindings = NativeScalars, aspect = NoteFacts, fragment = Board, entity = Note)]
//! pub struct NoteRow {
//!     #[worth_query(equality, read_only)]
//!     pub note_key: u64,
//! }
//!
//! mod posting {
//!     use worth_query_host::facade::worth_query_access_fragment;
//!     worth_query_access_fragment! {
//!         /// Resolves a task by key and retitles it.
//!         task_access {
//!             read_entities: [crate::Task];
//!             read_fields: [crate::TaskFacts::task_key(), crate::TaskFacts::title()];
//!             writes: [crate::TaskFacts::title()];
//!         }
//!     }
//!     pub(crate) use task_access;
//! }
//! worth_query_access_fragment! {
//!     /// Includes `task_access` and repeats its title read and write.
//!     audited_task_access {
//!         include: [crate::posting::task_access];
//!         read_fields: [crate::TaskFacts::title()];
//!         writes: [crate::TaskFacts::title()];
//!     }
//! }
//! worth_query_access_fragment! {
//!     /// Creates a note beside the retitle; repeats the title write.
//!     note_access { creates: [Note]; writes: [TaskFacts::title()]; }
//! }
//! worth_query_structured_value_binding!(pub RetitleInput for () {
//!     identity: "doc.fragment.retitle.input"
//! });
//! worth_query_operation!(pub Retitle for Schema, input RetitleInput;
//!     access { include: [audited_task_access, note_access]; read_fields: [TaskFacts::title()]; }
//! );
//!
//! pub struct Contracts;
//! impl Registration<Schema> for Contracts {
//!     fn register(
//!         schema: ApplicationSchemaDeclarationBuilder<Schema>,
//!     ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial> {
//!         let definition = Retitle::reference().definition().no_external_effect();
//!         Ok(schema.declared_operation::<Retitle>(definition.no_aftermath().finish()))
//!     }
//! }
//!
//! fn reads<T: OperationReads<Retitle>>() {}
//! fn creates<T: OperationCreates<Retitle>>() {}
//! fn main() {
//!     reads::<Task>();
//!     creates::<Note>();
//!     let declaration = Schema::declaration().expect("the included access declares one set");
//!     let title_writes = declaration.erased().members().iter().filter(|member| matches!(
//!         member,
//!         ApplicationSchemaMember::OperationProgram {
//!             operation, target: ApplicationOperationProgramTarget::Write { field, .. },
//!         } if operation == "Retitle" && field == "Title"
//!     ));
//!     assert_eq!(title_writes.count(), 1);
//! }
//! ```
//!
//! Entity, relation and effect members implement a capability trait per
//! operation, so one operation names each once across everything it
//! includes. Including a fragment both directly and through another repeats
//! its `read_entities: [Task]` and is a conflicting-impl error:
//!
//! ```compile_fail,E0119
//! use worth_query_decl::facade::application_schema::{fragment, *};
//! use worth_query_host::facade::{
//!     worth_query_access_fragment, worth_query_operation, worth_query_structured_value_binding,
//! };
//! pub struct Schema;
//! pub struct Task;
//! impl fragment::Entity for Task {const NAME:&'static str="Task";type Aspects=();type Relations=();}
//! worth_query_access_fragment! { task_access { read_entities: [Task]; } }
//! worth_query_access_fragment! { audited_task_access { include: [task_access]; } }
//! worth_query_structured_value_binding!(pub RetitleInput for () { identity: "doc.fragment.input" });
//! worth_query_operation!(pub Retitle for Schema, input RetitleInput;
//!     access { include: [audited_task_access, task_access]; }
//! );
//! ```
