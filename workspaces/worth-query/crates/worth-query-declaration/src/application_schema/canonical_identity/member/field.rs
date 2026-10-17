use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ApplicationSchemaMember;
mod value_shape;

pub(super) fn append_schema_field(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    let ApplicationSchemaMember::Field {
        entity,
        aspect,
        field,
        presence,
        scalar_family,
        value_type,
        canonical,
        unit,
        frame,
        writable,
        equality_queryable,
        unique,
    } = member
    else {
        unreachable!("field lowering requires a field member")
    };
    basis.text(format!("{prefix}.kind"), "field");
    basis.text(format!("{prefix}.entity"), entity);
    basis.text(format!("{prefix}.aspect"), aspect);
    basis.text(format!("{prefix}.field"), field);
    basis.text(format!("{prefix}.presence"), presence.canonical_name());
    basis.text(
        format!("{prefix}.scalar-family"),
        scalar_family.canonical_name(),
    );
    basis.text(format!("{prefix}.value-type"), value_type);
    if let Some(contract) = canonical {
        value_shape::append_contract(basis, prefix, contract);
    }
    basis.optional_text(format!("{prefix}.unit"), unit.as_deref());
    basis.optional_text(format!("{prefix}.frame"), frame.as_deref());
    basis.bool(format!("{prefix}.writable"), *writable);
    basis.bool(format!("{prefix}.equality-queryable"), *equality_queryable);
    basis.bool(format!("{prefix}.unique"), *unique);
}
