//! Canonical installed invariant-to-Source meaning; never a read authority.
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ApplicationSchemaMember;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    let ApplicationSchemaMember::InvariantManagedIndexSource {
        invariant,
        execution_point,
        requirement,
    } = member
    else {
        unreachable!("invariant Source canonical dispatch")
    };
    basis.text(format!("{prefix}.kind"), "invariant-managed-index-source");
    basis.text(format!("{prefix}.invariant"), invariant);
    basis.text(
        format!("{prefix}.execution-point"),
        execution_point.canonical_token(),
    );
    basis.text(format!("{prefix}.index"), requirement.index());
    basis.text(
        format!("{prefix}.interpretation"),
        requirement.interpretation().canonical_token(),
    );
    super::managed_index::append(
        basis,
        &format!("{prefix}.index-meaning"),
        requirement.definition(),
    );
    basis.text(
        format!("{prefix}.cold-source"),
        requirement.cold().canonical_token(),
    );
}
