//! The task record: projected identity/title/rank, filter-only notes,
//! status, due date and length.
use super::{Board, BoardBindings, Task};
use worth_foundational::facade::{CanonicalDate, CanonicalF64};
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::application_schema::*;

pub struct TaskFacts;
impl Aspect for TaskFacts {
    const NAME: &'static str = "TaskFacts";
    type Row = TaskFactsValue;
}
pub struct TaskFactsValue {
    pub key: u64,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub rank: i64,
    pub length: CanonicalF64,
    pub due: Option<CanonicalDate>,
}
impl Record<TaskFactsValue> for BoardBindings {
    fn record() -> impl Shape<TaskFactsValue, Self> {
        field("TaskKey", |row: &TaskFactsValue| row.key)
            .equality_queryable(true)
            .read_only()
            .and("Title", |row: &TaskFactsValue| row.title.clone())
            .and("Notes", |row: &TaskFactsValue| row.notes.clone())
            .and("Status", |row: &TaskFactsValue| row.status.clone())
            .equality_queryable(true)
            .and("Rank", |row: &TaskFactsValue| row.rank)
            .and("Length", |row: &TaskFactsValue| row.length)
            .and("Due", |row: &TaskFactsValue| row.due)
            .build(
                |key, title, notes, status, rank, length, due, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(TaskFactsValue {
                        key,
                        title,
                        notes,
                        status,
                        rank,
                        length,
                        due,
                    })
                },
            )
    }
}
pub type TaskKey = FieldSelection<Board, Task, TaskFacts, u64>;
pub type Title = FieldSelection<Board, Task, TaskFacts, String>;
pub type Notes = FieldSelection<Board, Task, TaskFacts, String>;
pub type Status = FieldSelection<Board, Task, TaskFacts, String>;
pub type Rank = FieldSelection<Board, Task, TaskFacts, i64>;
pub type Length = FieldSelection<Board, Task, TaskFacts, CanonicalF64>;
pub type Due = FieldSelection<Board, Task, TaskFacts, CanonicalDate>;
