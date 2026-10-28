//! Domain/case semantics extend the canonical vocabulary without changing old members.
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ErasedApplicationProjectedMemberDomainDefinition;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationProjectedMemberDomainDefinition,
) {
    let parts = definition.parts();
    basis.text(format!("{prefix}.kind"), "projected-member-domain");
    basis.u64(
        format!("{prefix}.semantic-version"),
        parts.semantic_version.into(),
    );
    basis.text(format!("{prefix}.domain"), &parts.domain);
    basis.text(format!("{prefix}.carrier"), &parts.carrier);
    basis.usize(format!("{prefix}.cases.len"), parts.cases.len());
    for (index, case) in parts.cases.iter().enumerate() {
        basis.text(format!("{prefix}.case[{index}]"), case);
    }
}
