//! Actual ordinary and predecode decision reads, including whole absence.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
pub fn observe<Schema: BirthSchema, const KEEP: bool>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema, KEEP>>,
    scope_key: &str,
    materialized: bool,
) -> Result<(), HandlerExecutionDenial>
where
    Binding<Schema, KEEP>: ApplicationMutationBinding<Schema, ScopeBinding = PlanarMutationScope>,
{
    let root = reader
        .resolve_entity(BodyKey::at("BodyKey"), scope_key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    // Both APIs observe ONLY the untouched selected field. No Content read
    // may mask the whole-aspect dependency when another field materializes it.
    let selection = reader.field(&root, SourceSelection::at("SourceSelection"))?;
    if materialized {
        assert_eq!(
            selection.as_ref().map(source_text::SourceText::as_str),
            Some("selected")
        );
    } else {
        assert!(selection.is_none());
    }
    let selected = reader
        .field_with_predecode_admission(
            &root,
            SourceSelection::at("SourceSelection"),
            |_, checkpoint| {
                checkpoint().unwrap();
                Ok::<_, ()>(())
            },
        )?
        .unwrap();
    assert_eq!(
        selected, selection,
        "ordinary and predecode observe the same actual selected field"
    );
    assert!(
        materialized || selected.is_none(),
        "the selected field remains absent even after another field materializes the aspect"
    );
    Ok(())
}
pub fn birth<Schema: BirthSchema>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema, false>>,
    scope_key: &str,
) -> Result<(), HandlerExecutionDenial> {
    observe(reader, scope_key, false)
}
pub fn preserve<Schema: BirthSchema>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema, true>>,
    scope_key: &str,
) -> Result<(), HandlerExecutionDenial> {
    let materialized = matches!(Schema::SOURCE_WRITE, crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::own_effect::SourceWrite::Materialize);
    observe(reader, scope_key, materialized)?;
    if materialized {
        let root = reader
            .resolve_entity(BodyKey::at("BodyKey"), scope_key.to_owned())
            .map_err(HandlerExecutionDenial::new)?;
        assert_eq!(
            reader
                .field(&root, SourceContent::at("SourceContent"))?
                .as_ref()
                .map(source_text::SourceText::as_str),
            Some("complete")
        );
    }
    Ok(())
}
