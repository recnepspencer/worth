use sha2::{Digest, Sha256};

const MAGIC: &[u8; 8] = b"WQAPCP01";
const FORMAT_VERSION: u16 = 2;
const CHECKSUM_BYTES: usize = 32;
const BODY_PREFIX_BYTES: usize = 2 + 8 + 8;
const HEADER_BYTES: usize = MAGIC.len() + CHECKSUM_BYTES + BODY_PREFIX_BYTES;

/// Opaque bytes for one complete Query application checkpoint.
///
/// Query alone defines and admits the payload. Hosts retain and transport the
/// bytes without interpreting model facts or reconstructing a partial runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCheckpoint {
    bytes: Box<[u8]>,
}

pub(in crate::domain_computation::primary_graph) struct DecodedApplicationCheckpoint {
    pub(super) native: worth_relational::facade::durability::RelationalNativeCheckpoint,
    pub(super) bootstrap_commit_id: worth_relational::facade::history::CommitId,
}

impl WorthQueryApplicationCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[cfg(feature = "test-durability-faults")]
    #[doc(hidden)]
    pub fn rewrite_authenticated_body_for_durability_test(
        &self,
        rewrite: impl FnOnce(&mut [u8]),
    ) -> Self {
        let mut bytes = self.bytes.to_vec();
        let body_start = MAGIC.len() + CHECKSUM_BYTES;
        rewrite(&mut bytes[body_start..]);
        let checksum = Sha256::digest(&bytes[body_start..]);
        bytes[MAGIC.len()..body_start].copy_from_slice(&checksum);
        Self::from_untrusted_bytes(bytes.into_boxed_slice())
    }

    fn encode(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &super::WorthQueryPrimaryGraphPublication,
    ) -> Self {
        let native_bytes = native.bytes();
        let mut body = Vec::with_capacity(BODY_PREFIX_BYTES + native_bytes.len());
        body.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        body.extend_from_slice(&publication.bootstrap_commit_id().0.to_be_bytes());
        body.extend_from_slice(&(native_bytes.len() as u64).to_be_bytes());
        body.extend_from_slice(native_bytes);
        let checksum = Sha256::digest(&body);
        let mut bytes = Vec::with_capacity(MAGIC.len() + CHECKSUM_BYTES + body.len());
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&checksum);
        bytes.extend_from_slice(&body);
        Self {
            bytes: bytes.into_boxed_slice(),
        }
    }

    pub(super) fn decode(&self) -> Result<DecodedApplicationCheckpoint, String> {
        if self.bytes.len() < HEADER_BYTES || &self.bytes[..MAGIC.len()] != MAGIC {
            return Err("Query application checkpoint header is invalid".to_owned());
        }
        let checksum_start = MAGIC.len();
        let body_start = checksum_start + CHECKSUM_BYTES;
        let expected_checksum = &self.bytes[checksum_start..body_start];
        let body = &self.bytes[body_start..];
        if Sha256::digest(body).as_slice() != expected_checksum {
            return Err("Query application checkpoint checksum differs".to_owned());
        }
        let version = u16::from_be_bytes([body[0], body[1]]);
        if version != FORMAT_VERSION {
            return Err(format!(
                "Query application checkpoint format {version} is unsupported"
            ));
        }
        let mut cursor = 2;
        let mut next_u64 = || {
            let end = cursor + 8;
            let value = u64::from_be_bytes(
                body[cursor..end]
                    .try_into()
                    .expect("checkpoint header bounds were validated"),
            );
            cursor = end;
            value
        };
        let bootstrap_commit_id = worth_relational::facade::history::CommitId(next_u64());
        let native_len = usize::try_from(next_u64())
            .map_err(|_| "checkpoint payload length exceeds this host".to_owned())?;
        let native = body
            .get(cursor..)
            .filter(|bytes| bytes.len() == native_len)
            .ok_or_else(|| "Query application checkpoint payload length differs".to_owned())?;
        Ok(DecodedApplicationCheckpoint {
            native: worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
                native.to_vec().into_boxed_slice(),
            ),
            bootstrap_commit_id,
        })
    }
}

impl<Schema> super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
{
    pub fn capture_application_checkpoint(
        &self,
    ) -> Result<
        WorthQueryApplicationCheckpoint,
        worth_relational::facade::durability::DurabilityError,
    > {
        self.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .durability_authority()
                .native_checkpoint()
                .map(|checkpoint| {
                    WorthQueryApplicationCheckpoint::encode(checkpoint, self.publication())
                })
        })
    }
}
