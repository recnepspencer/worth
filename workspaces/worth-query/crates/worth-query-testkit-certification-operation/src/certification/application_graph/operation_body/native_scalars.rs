//! Native scalar carriers through one authored body: an optional Date is
//! initialized, replaced, read back and cleared beside Time, UUID and inline
//! bytes fields, and forged or non-canonical carriers refuse before effects.
#[path = "native_scalars/binding.rs"]
pub mod binding;
#[path = "native_scalars/declaration.rs"]
pub mod declaration;
#[path = "native_scalars/query.rs"]
pub mod query;
#[path = "native_scalars/records.rs"]
pub mod records;
pub use super::*;
pub use binding::{ForgedDay, ScheduleInput};
pub use query::ScheduleRow;
pub use worth_foundational::facade::{CanonicalDate, CanonicalTime, InlineBytes};

/// The only destination whose program installs the schedule operations.
pub const MODE: u64 = 40;

pub type Application =
    WorthQueryProgramApplicationRuntime<DocumentRetentionSchema<MODE>, program::BodyProgram<MODE>>;

pub fn assign(
    app: &Application,
    key: u64,
    input: ScheduleInput<CanonicalDate>,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(binding::ScheduleIntent(input))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

/// Submits a carrier an untrusted client forged in `CanonicalDate`'s shape.
pub fn assign_forged(
    app: &Application,
    key: u64,
    input: ScheduleInput<ForgedDay>,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(binding::ForgedScheduleIntent(input))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

pub fn read(app: &Application) -> ScheduleRow {
    query::read_row(app)
}
