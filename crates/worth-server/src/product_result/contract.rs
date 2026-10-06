use super::WorthServerProductResultContractError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductResultEncoding {
    CanonicalJson,
}

impl WorthServerProductResultEncoding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalJson => "canonical-json",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductResultCanonicalization {
    CanonicalJsonV1,
}

impl WorthServerProductResultCanonicalization {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalJsonV1 => "canonical-json-v1",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductResultSchema {
    identity: String,
    version: u32,
}

impl WorthServerProductResultSchema {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        crate::product_adapter::execution_pipeline::read_batch_accounting::string(&self.identity)
    }

    pub fn new(
        identity: impl Into<String>,
        version: u32,
    ) -> Result<Self, WorthServerProductResultContractError> {
        let identity = identity.into().trim().to_string();
        if identity.is_empty() {
            return Err(WorthServerProductResultContractError::blank_schema_identity());
        }
        if version == 0 {
            return Err(WorthServerProductResultContractError::zero_schema_version());
        }
        Ok(Self { identity, version })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductResultContract {
    schema: WorthServerProductResultSchema,
    encoding: WorthServerProductResultEncoding,
    canonicalization: WorthServerProductResultCanonicalization,
    max_inline_bytes: usize,
    canonical_digest: String,
}

impl WorthServerProductResultContract {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.schema
            .owned_allocation_capacity_bytes()
            .saturating_add(
                crate::product_adapter::execution_pipeline::read_batch_accounting::string(
                    &self.canonical_digest,
                ),
            )
    }

    pub fn canonical_json(
        schema_identity: impl Into<String>,
        schema_version: u32,
        max_inline_bytes: usize,
    ) -> Result<Self, WorthServerProductResultContractError> {
        if max_inline_bytes == 0 {
            return Err(WorthServerProductResultContractError::zero_inline_budget());
        }
        let schema = WorthServerProductResultSchema::new(schema_identity, schema_version)?;
        let encoding = WorthServerProductResultEncoding::CanonicalJson;
        let canonicalization = WorthServerProductResultCanonicalization::CanonicalJsonV1;
        let schema_version = schema.version().to_string();
        let max_inline_bytes_text = max_inline_bytes.to_string();
        let canonical_digest = crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
            "worth-server-product-result-contract-v2",
        )
        .field("schema", schema.identity())
        .field("version", &schema_version)
        .field("encoding", encoding.as_str())
        .field("canonicalization", canonicalization.as_str())
        .field("max_inline_bytes", &max_inline_bytes_text)
        .finish();
        Ok(Self {
            schema,
            encoding,
            canonicalization,
            max_inline_bytes,
            canonical_digest,
        })
    }

    pub fn schema(&self) -> &WorthServerProductResultSchema {
        &self.schema
    }

    pub fn encoding(&self) -> WorthServerProductResultEncoding {
        self.encoding
    }

    pub fn canonicalization(&self) -> WorthServerProductResultCanonicalization {
        self.canonicalization
    }

    pub fn max_inline_bytes(&self) -> usize {
        self.max_inline_bytes
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
