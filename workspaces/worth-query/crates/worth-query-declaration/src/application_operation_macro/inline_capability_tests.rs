//! Inline `worth_query_operation!` capability clauses grant what their
//! standalone macros grant, for concrete and schema-generic operations.
use crate::application_schema::{
    ApplicationOperationMarkerIdentity, OperationCreates, OperationLinks, OperationReads,
    OperationWrites,
};

pub struct Board;
pub trait BoardSchema {}
impl BoardSchema for Board {}
pub struct Input;
pub struct Task;
pub struct Inbox;
pub struct InboxTasks;
pub struct Title;
crate::worth_query_structured_value_binding!(pub InputBinding for Input {
    identity: "inline-capability.input.v1"
});

crate::worth_query_operation!(pub Add for Board, input InputBinding;
    reads [Inbox]; creates [Task]; links [InboxTasks];
);
crate::worth_query_operation!(pub Retitle for Schema: BoardSchema, input InputBinding;
    reads [Task]; writes [Title]
);

fn grants<Operation, Member>()
where
    Member: OperationReads<Operation>,
{
}

#[test]
fn inline_clauses_grant_each_listed_capability() {
    fn created<Operation, Member: OperationCreates<Operation>>() {}
    fn linked<Operation, Member: OperationLinks<Operation>>() {}
    fn written<Operation, Member: OperationWrites<Operation>>() {}
    grants::<Add, Inbox>();
    created::<Add, Task>();
    linked::<Add, InboxTasks>();
    grants::<Retitle, Task>();
    written::<Retitle, Title>();
    assert_eq!(
        <Add as ApplicationOperationMarkerIdentity<Board>>::IDENTIFIER,
        "Add"
    );
    assert_eq!(
        <Retitle as ApplicationOperationMarkerIdentity<Board>>::IDENTIFIER,
        "Retitle"
    );
    assert_eq!(Add::reference().name(), "Add");
    assert_eq!(Retitle::reference::<Board>().name(), "Retitle");
}
