use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalizationRuleVersion,
};
use worth_query_declaration::facade::application_schema::{
    application_authorization_path_canonical_components, ApplicationAuthorizationPath,
};

use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

const DOMAIN: &str = "worth-query.authorization-path";
const RULE: &str = "worth-query-authorization-path-v2";
pub(super) struct PreparedAuthorizationPathIdentity {
    pub(super) digest: CanonicalDigestId,
    pub(super) work: WorthQueryCanonicalWorkEvidence,
}

pub(super) fn prepare_authorization_path_identity(
    path: &ApplicationAuthorizationPath,
    budget: CanonicalDigestWorkBudget,
) -> Result<PreparedAuthorizationPathIdentity, CanonicalDigestDerivationDenial> {
    let version =
        CanonicalizationRuleVersion::new(RULE).expect("fixed authorization path rule is valid");
    let basis = prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Future(DOMAIN),
        path_entries(path),
    )
    .into_result()
    .expect("authorization paths always have a nonempty canonical basis");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()?;
    let derived = canonicalization().digest().derive(ready);
    Ok(PreparedAuthorizationPathIdentity {
        digest: CanonicalDigestId::new(*derived.value().bytes()),
        work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    })
}

fn path_entries(path: &ApplicationAuthorizationPath) -> Vec<CanonicalBasisEntry> {
    let mut entries = vec![text("family", "authorization-path")];
    entries.extend(
        application_authorization_path_canonical_components(path)
            .into_iter()
            .map(|component| entry(component.locus(), component.value().clone())),
    );
    entries
}

fn text(locus: impl Into<String>, value: impl Into<String>) -> CanonicalBasisEntry {
    entry(locus, CanonicalBasisValue::ExactText(value.into().into()))
}

fn entry(locus: impl Into<String>, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Future(DOMAIN),
        CanonicalBasisLocus::Named(locus.into().into()),
        CanonicalBasisEntryKind::Field,
        value,
    )
}
