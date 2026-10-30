use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

use super::{
    WorthQueryInstalledGraphObligationContract, WorthQueryInstalledGraphObligationResourcePosture,
};

mod contract_encoding;

use contract_encoding::{encode_contract, encode_resources};

const OBLIGATION_IDENTITY_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(4_096, 1024 * 1024) {
        Some(budget) => budget,
        None => panic!("fixed graph-obligation identity budget is valid"),
    };

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphObligationSetIdentity(CanonicalDigestId);

impl WorthQueryInstalledGraphObligationSetIdentity {
    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.0
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        self.0.bytes()
    }

    pub fn render_support_hex(&self) -> String {
        self.0.render_hex()
    }
}

pub(super) fn derive_set_identity(
    binding: &ApplicationSchemaBindingIdentity,
    subject_kind: &str,
    subject_name: &str,
    input_type: Option<&str>,
    contracts: &[WorthQueryInstalledGraphObligationContract],
    resources: &WorthQueryInstalledGraphObligationResourcePosture,
) -> Result<
    (
        WorthQueryInstalledGraphObligationSetIdentity,
        WorthQueryCanonicalWorkEvidence,
    ),
    CanonicalDigestDerivationDenial,
> {
    let mut basis = InstallationCanonicalIdentityBasis::new(
        "worth-query.installed-graph-obligations",
        "worth-query-installed-graph-obligations-v1",
        OBLIGATION_IDENTITY_BUDGET,
    );
    basis.digest("package", *binding.package_identity())?;
    basis.digest("schema", *binding.schema_identity())?;
    basis.text("subject-kind", subject_kind)?;
    basis.text("subject-name", subject_name)?;
    if let Some(input_type) = input_type {
        basis.text(
            match subject_kind {
                "application-resource" => "resource-purpose",
                "application-managed-index" => "parent-binding",
                _ => "input-type",
            },
            input_type,
        )?;
    }
    basis.unsigned_usize("obligation-count", contracts.len())?;
    for (index, contract) in contracts.iter().enumerate() {
        encode_contract(&mut basis, index, contract)?;
    }
    encode_resources(&mut basis, resources)?;
    let (identity, work) = basis.derive()?;
    Ok((
        WorthQueryInstalledGraphObligationSetIdentity(identity),
        work,
    ))
}
