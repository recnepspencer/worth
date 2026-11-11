Text loans may produce owned results while keeping decoded backing inside the
original invocation:

```rust
# use worth_query_execution::facade::primary_graph::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexTextReadDenial};
# use worth_query_declaration::facade::{application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef}, application_schema::{ApplicationSchema, ApplicationReadableScalarValueBinding}, portable_identity::WorthQueryPortableType};
# type Query<S,I> = <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
fn lend<S, I, Slot, Field>(
    row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<Query<S,I>, Slot, S, Field>,
) -> Result<usize, WorthQueryManagedIndexTextReadDenial>
where S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>,
    Slot: WorthQueryPortableType, Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Value=String>,
    Field::Binding: ApplicationReadableScalarValueBinding,
{
    row.with_text(selector, |text| text.len())
}
```

A text borrow cannot escape its callback, including through a warm memo hit:

```compile_fail
# use worth_query_execution::facade::primary_graph::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexTextReadDenial};
# use worth_query_declaration::facade::{application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef}, application_schema::{ApplicationSchema, ApplicationReadableScalarValueBinding}, portable_identity::WorthQueryPortableType};
# type Query<S,I> = <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
fn escape<'row, S, I, Slot, Field>(
    row: &'row WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<Query<S,I>, Slot, S, Field>,
) -> Result<&'row str, WorthQueryManagedIndexTextReadDenial>
where S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>,
    Slot: WorthQueryPortableType, Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Value=String>,
    Field::Binding: ApplicationReadableScalarValueBinding,
{
    row.with_text(selector, |text| text)
}
```


Content uses the same installed field selection and original controls:

```rust
# use worth_query_execution::facade::primary_graph::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexContentReadDenial};
# use worth_query_declaration::facade::{application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef}, application_schema::{ApplicationSchema,ApplicationContentBinding,ApplicationReadableScalarValueBinding}, portable_identity::WorthQueryPortableType};
# use worth_foundational::facade::ContentRefId;
# type Query<S,I> = <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
fn content_length<S,I,Slot,Field,C>(
 row:&WorthQueryManagedIndexSourceRow<'_,S,I>,
 selector:ApplicationQueryResultFieldRef<Query<S,I>,Slot,S,Field>,format:C,
)->Result<usize,WorthQueryManagedIndexContentReadDenial>
where S:ApplicationSchema,I:ApplicationManagedIndexBinding<S>,C:ApplicationContentBinding,
 Slot:WorthQueryPortableType,Field:worth_query_declaration::facade::application_schema::fragment::RecordSelection<Value=ContentRefId>,
 Field::Binding:worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
{row.with_content(selector,format,|payload|payload.len())}
```

A content borrow cannot escape its callback:

```compile_fail
# use worth_query_execution::facade::primary_graph::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexContentReadDenial};
# use worth_query_declaration::facade::{application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef}, application_schema::{ApplicationSchema,ApplicationContentBinding,ApplicationReadableScalarValueBinding}, portable_identity::WorthQueryPortableType};
# use worth_foundational::facade::ContentRefId;
# type Query<S,I> = <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
fn escape<'row,S,I,Slot,Field,C>(
 row:&'row WorthQueryManagedIndexSourceRow<'_,S,I>,
 selector:ApplicationQueryResultFieldRef<Query<S,I>,Slot,S,Field>,format:C,
)->Result<&'row [u8],WorthQueryManagedIndexContentReadDenial>
where S:ApplicationSchema,I:ApplicationManagedIndexBinding<S>,C:ApplicationContentBinding,
 Slot:WorthQueryPortableType,Field:worth_query_declaration::facade::application_schema::fragment::RecordSelection<Value=ContentRefId>,
 Field::Binding:worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
{row.with_content(selector,format,|payload|payload)}
```
