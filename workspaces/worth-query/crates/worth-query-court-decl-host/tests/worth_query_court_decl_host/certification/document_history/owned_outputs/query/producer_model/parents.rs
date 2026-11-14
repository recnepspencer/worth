//! This source consumes every actual upstream realized First value.
use super::*;
pub struct Links;
pub struct Parent;
pub struct Output;
pub struct Value;
worth_query_portable_type!(Links=>"owned-outputs.source-parent-links");
worth_query_portable_type!(Parent=>"owned-outputs.source-parent");
worth_query_portable_type!(Output=>"owned-outputs.source-parent-output");
worth_query_portable_type!(Value=>"owned-outputs.source-parent-value");
fn links() -> ApplicationQueryResultRelationRef<
    Query,
    Links,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("parent-links", ModelMember::reference())
}
fn parent() -> ApplicationQueryResultRelationRef<
    Query,
    Parent,
    Schema,
    UpstreamOf,
    Member,
    Model,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("parent-model", UpstreamOf::reference())
}
fn output() -> ApplicationQueryResultRelationRef<
    Query,
    Output,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("parent-output", FirstOf::reference())
}
fn value() -> ApplicationQueryResultFieldRef<Query, Value, Schema, FirstValue> {
    ApplicationQueryResultFieldRef::new("parent-value", FirstValue::at("FirstValue"))
}
pub(super) fn sum(
    row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
) -> Result<u64, WorthQueryApplicationProjectionDenial> {
    let mut sum = 0;
    for link in row.many(links())?.iter() {
        if let Some(parent) = link.optional(parent())? {
            sum += parent.one(output())?.field(value())?;
        }
    }
    Ok(sum)
}
pub(super) fn shape(
    root: ApplicationQueryResultShapeBuilder<Schema, Query, Model, Row, RowBinding>,
) -> ApplicationQueryResultShapeBuilder<Schema, Query, Model, Row, RowBinding> {
    let output_row =
        ApplicationQueryResultShapeBuilder::<Schema, Query, FirstOutput, Row, RowBinding>::new(
            FirstOutput::reference(),
        )
        .field(value());
    let parent_row =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Row, RowBinding>::new(
            Model::reference(),
        )
        .relation(output(), output_row);
    let link = ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Row, RowBinding>::new(
        Member::reference(),
    )
    .relation(parent(), parent_row);
    root.relation(links(), link)
}
