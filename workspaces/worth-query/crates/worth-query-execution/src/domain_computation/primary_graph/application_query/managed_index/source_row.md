Text loans may produce owned results while keeping decoded backing inside the
original invocation:

```rust
# use worth_query_execution::facade::primary_graph::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexTextReadDenial};
# use worth_query_declaration::facade::{application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef}, application_schema::{ApplicationSchema, ApplicationFieldUnit, ApplicationReadableScalarValueBinding, RequiredApplicationFieldValue, WritePosture}, portable_identity::WorthQueryPortableType};
# type Query<S,I> = <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
fn lend<S, I, Slot, Entity, Aspect, Field, Write, Equality, Unit>(
    row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<Query<S,I>, Slot, S, Entity, Aspect, Field, String, Write, Equality, Unit>,
) -> Result<usize, WorthQueryManagedIndexTextReadDenial>
where S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>,
    Slot: WorthQueryPortableType, Field: RequiredApplicationFieldValue<Value=String>,
    Field::Binding: ApplicationReadableScalarValueBinding, Write: WritePosture, Unit: ApplicationFieldUnit,
{
    row.with_text(selector, |text| text.len())
}
```

A text borrow cannot escape its callback, including through a warm memo hit:

```compile_fail
# use worth_query_execution::facade::primary_graph::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexTextReadDenial};
# use worth_query_declaration::facade::{application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef}, application_schema::{ApplicationSchema, ApplicationFieldUnit, ApplicationReadableScalarValueBinding, RequiredApplicationFieldValue, WritePosture}, portable_identity::WorthQueryPortableType};
# type Query<S,I> = <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
fn escape<'row, S, I, Slot, Entity, Aspect, Field, Write, Equality, Unit>(
    row: &'row WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<Query<S,I>, Slot, S, Entity, Aspect, Field, String, Write, Equality, Unit>,
) -> Result<&'row str, WorthQueryManagedIndexTextReadDenial>
where S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>,
    Slot: WorthQueryPortableType, Field: RequiredApplicationFieldValue<Value=String>,
    Field::Binding: ApplicationReadableScalarValueBinding, Write: WritePosture, Unit: ApplicationFieldUnit,
{
    row.with_text(selector, |text| text)
}
```
