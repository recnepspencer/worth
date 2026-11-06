use super::Record;
use super::*;
use worth_query_decl::facade::application_schema::fragment;
trait Actions<W> {
    fn grant(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema>;
    fn probe(
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        root: &WorthQueryInvariantEntityIdentity<Schema, Record>,
        found: &mut Option<u64>,
        count: &mut u64,
    ) -> Result<(), HandlerExecutionDenial>;
}
impl<A, EP, AP> Actions<(EP, AP)> for Option<A>
where
    A: fragment::Aspect<Row = u64>
        + In<<ClassModel as fragment::Fragment>::Members, (EP, AP), (ClassModel, Record)>,
{
    fn grant(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema.operation_read_field(
            CopySelected::reference(),
            FieldSelection::<ClassModel, Record, A, u64>::at("value"),
        )
    }
    fn probe(
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        root: &WorthQueryInvariantEntityIdentity<Schema, Record>,
        found: &mut Option<u64>,
        count: &mut u64,
    ) -> Result<(), HandlerExecutionDenial> {
        if let Some(value) = reader.field(
            root,
            FieldSelection::<ClassModel, Record, A, u64>::at("value"),
        )? {
            *found = Some(value);
            *count += 1;
        }
        Ok(())
    }
}
macro_rules! tuples {
 ($($a:ident $w:ident),+) => {
 impl<$($a, $w),+> Actions<($($w,)+)> for ($($a,)+) where $($a: Actions<$w>,)+ {
 fn grant(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> { $(let schema = $a::grant(schema);)+ schema }
 fn probe(reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>, root: &WorthQueryInvariantEntityIdentity<Schema, Record>, found: &mut Option<u64>, count: &mut u64) -> Result<(), HandlerExecutionDenial> { $($a::probe(reader, root, found, count)?;)+ Ok(()) }
 }
 };
}
tuples!(A WA);
tuples!(A WA, B WB);
tuples!(A WA, B WB, C WC);
tuples!(A WA, B WB, C WC, D WD);
tuples!(A WA, B WB, C WC, D WD, E WE);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG, H WH);
pub fn grant(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    Classes::grant(schema)
}
pub fn probe(
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    root: &WorthQueryInvariantEntityIdentity<Schema, Record>,
    found: &mut Option<u64>,
    count: &mut u64,
) -> Result<(), HandlerExecutionDenial> {
    Classes::probe(reader, root, found, count)
}
