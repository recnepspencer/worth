use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::schema_reconciliation_witness_rows::RelationalSchemaReconciliationWitnessRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalSchemaReconciliationWitness {
    request_digest: String,
    branch_basis_digest: String,
    rows: Arc<[RelationalSchemaReconciliationWitnessRow]>,
    witness_digest: String,
}

impl RelationalSchemaReconciliationWitness {
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
            rows,
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
    rows: Arc<[RelationalSchemaReconciliationWitnessRow]>,
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
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.schema_reconciliation_witness.v1");
    bytes.extend_from_slice(request_digest.as_bytes());
    bytes.extend_from_slice(branch_basis_digest.as_bytes());
    bytes.extend_from_slice(rows.len().to_string().as_bytes());
    for row in rows {
        bytes.extend_from_slice(
            &rmp_serde::to_vec_named(row).expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "schema reconciliation witness row must encode"),
        );
    }
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest_is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
