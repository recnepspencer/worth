use super::*;

pub(super) fn graph_material() -> WorthQueryGraphReadMaterial {
    graph_material_rows(1)
}

pub(super) fn graph_material_rows(row_count: usize) -> WorthQueryGraphReadMaterial {
    let path = CanonicalFieldPath::single(FieldKey::new("id").expect("valid field key"));
    WorthQueryGraphReadMaterial::new((0..row_count).map(|index| {
        WorthQueryGraphReadRow::from_native_fields(
            format!("managed-entity-{index}"),
            [(
                path.clone(),
                AspectValue::String(InternedString::from(format!("entity-{index}"))),
            )]
            .into_iter()
            .collect(),
        )
        .expect("managed graph row should construct")
    }))
}
