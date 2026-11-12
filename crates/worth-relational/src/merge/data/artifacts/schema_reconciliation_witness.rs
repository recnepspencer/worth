use super::MergeWitnessRows;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::sync::Arc;

use super::correspondence_witness::{lowercase_sha256, CorrespondenceHashWriter};

use super::schema_reconciliation_witness_rows::RelationalSchemaReconciliationWitnessRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalSchemaReconciliationWitness {
    request_digest: String,
    branch_basis_digest: String,
    rows: MergeWitnessRows<RelationalSchemaReconciliationWitnessRow>,
    witness_digest: String,
}

impl RelationalSchemaReconciliationWitness {
    pub(crate) fn retain_rows_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.rows.retain_decoded_memory(memory)
    }

    pub(crate) fn retained(
        request_digest: String,
        branch_basis_digest: String,
        rows: Arc<[RelationalSchemaReconciliationWitnessRow]>,
    ) -> Self {
        let witness_digest =
            schema_reconciliation_witness_digest(&request_digest, &branch_basis_digest, &rows);
        Self {
            request_digest,
            branch_basis_digest,
            rows: rows.into(),
            witness_digest,
        }
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn branch_basis_digest(&self) -> &str {
        &self.branch_basis_digest
    }

    pub fn rows(&self) -> &[RelationalSchemaReconciliationWitnessRow] {
        &self.rows
    }

    pub fn witness_digest(&self) -> &str {
        &self.witness_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalSchemaReconciliationWitnessWire {
    request_digest: String,
    branch_basis_digest: String,
    rows: MergeWitnessRows<RelationalSchemaReconciliationWitnessRow>,
    witness_digest: String,
}

impl<'de> Deserialize<'de> for RelationalSchemaReconciliationWitness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalSchemaReconciliationWitnessWire::deserialize(deserializer)?;
        if !digest_is_lowercase_sha256_hex(&wire.request_digest) {
            return Err(D::Error::custom(
                "schema reconciliation witness request digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.branch_basis_digest) {
            return Err(D::Error::custom(
                "schema reconciliation witness branch basis digest is not valid lowercase sha256 hex",
            ));
        }
        let witness_digest = schema_reconciliation_witness_digest(
            &wire.request_digest,
            &wire.branch_basis_digest,
            &wire.rows,
        );
        if witness_digest != wire.witness_digest {
            return Err(D::Error::custom(
                "schema reconciliation witness digest does not match retained schema truth",
            ));
        }
        Ok(Self {
            request_digest: wire.request_digest,
            branch_basis_digest: wire.branch_basis_digest,
            rows: wire.rows,
            witness_digest: wire.witness_digest,
        })
    }
}

fn schema_reconciliation_witness_digest(
    request_digest: &str,
    branch_basis_digest: &str,
    rows: &[RelationalSchemaReconciliationWitnessRow],
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.merge.schema_reconciliation_witness.v1");
    digest.update(request_digest.as_bytes());
    digest.update(branch_basis_digest.as_bytes());
    {
        let mut writer = CorrespondenceHashWriter(&mut digest);
        write!(&mut writer, "{}", rows.len()).expect("hash writing cannot fail");
        for row in rows {
            rmp_serde::encode::write_named(&mut writer, row)
                .expect("closed native schema reconciliation row must encode");
        }
    }
    lowercase_sha256(digest.finalize().into())
}

fn digest_is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
