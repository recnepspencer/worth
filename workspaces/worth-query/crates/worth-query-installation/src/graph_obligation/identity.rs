use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

use super::{
    WorthQueryInstalledGraphObligationContract, WorthQueryInstalledGraphObligationResourcePosture,
};

mod aspect_contract_table;
mod contract_encoding;
mod mask_encoding;
mod scope_encoding;
#[cfg(test)]
mod scope_identity_tests;

use contract_encoding::{encode_contract, encode_resources};

const RULE_VERSION: &str = "worth-query-installed-graph-obligations-v4";

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
    budget: CanonicalDigestWorkBudget,
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
        RULE_VERSION,
        budget,
    );
    basis.digest("package", *binding.package_identity())?;
    basis.digest("schema", *binding.schema_identity())?;
    basis.text("subject-kind", subject_kind)?;
    basis.text("subject-name", subject_name)?;
    if let Some(input_type) = input_type {
        basis.text(
            match subject_kind {
                "application-resource" => "resource-purpose",
                "application-inspection" => "inspection-purpose",
                "application-managed-index" => "parent-binding",
                _ => "input-type",
            },
            input_type,
        )?;
    }
    basis.unsigned_usize("obligation-count", contracts.len())?;
    let mut table = aspect_contract_table::AspectContractTable::default();
    for (index, contract) in contracts.iter().enumerate() {
        encode_contract(&mut basis, &mut table, index, contract)?;
    }
    encode_resources(&mut basis, resources)?;
    let (identity, work) = basis.derive()?;
    Ok((
        WorthQueryInstalledGraphObligationSetIdentity(identity),
        work,
    ))
}
