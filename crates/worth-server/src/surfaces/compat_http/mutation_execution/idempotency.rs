use crate::{
    WorthServerCompatibilityPreparedRequest, WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDenialCode,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerIdempotencyKey {
    value: String,
    canonical_digest: String,
}

impl WorthServerIdempotencyKey {
    pub(crate) fn from_prepared_request(
        prepared_request: &WorthServerCompatibilityPreparedRequest,
    ) -> Result<Option<Self>, WorthServerQueryHandoffDenial> {
        let request_context = prepared_request.admission().request_context();
        let Some(values) = prepared_request
            .request_contract()
            .canonical_headers()
            .values("idempotency-key")
        else {
            return Ok(None);
        };
        if values.len() != 1 {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                request_context.diagnostics_profile(),
                "compatibility mutation requires a single canonical `idempotency-key` header value",
            ));
        }
        let value = values[0].trim();
        if value.is_empty() {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                request_context.diagnostics_profile(),
                "compatibility mutation `idempotency-key` header may not be blank",
            ));
        }
        Ok(Some(Self {
            value: value.to_string(),
            canonical_digest: format!("compat-http-idempotency-key-v1|{value}"),
        }))
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
