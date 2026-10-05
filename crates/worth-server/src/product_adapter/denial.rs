#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductOperationDenialCode {
    PayloadSchemaMismatch,
    DeclaredPayloadValidator,
    ProductSemantic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationDenialFacts {
    code: WorthServerProductOperationDenialCode,
    expected_basis_digest: Option<String>,
    observed_basis_digest: Option<String>,
}

impl WorthServerProductOperationDenialFacts {
    pub fn code(&self) -> WorthServerProductOperationDenialCode {
        self.code
    }

    pub fn expected_basis_digest(&self) -> Option<&str> {
        self.expected_basis_digest.as_deref()
    }

    pub fn observed_basis_digest(&self) -> Option<&str> {
        self.observed_basis_digest.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationDenial {
    reason_key: String,
    detail: String,
    facts: Option<WorthServerProductOperationDenialFacts>,
}

impl WorthServerProductOperationDenial {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::{option_string, string};
        string(&self.reason_key)
            .saturating_add(string(&self.detail))
            .saturating_add(self.facts.as_ref().map_or(0, |facts| {
                option_string(&facts.expected_basis_digest)
                    .saturating_add(option_string(&facts.observed_basis_digest))
            }))
    }

    pub fn new(reason_key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            reason_key: reason_key.into(),
            detail: detail.into(),
            facts: None,
        }
    }

    pub(crate) fn with_code(mut self, code: WorthServerProductOperationDenialCode) -> Self {
        match self.facts.as_mut() {
            Some(facts) => facts.code = code,
            None => {
                self.facts = Some(WorthServerProductOperationDenialFacts {
                    code,
                    expected_basis_digest: None,
                    observed_basis_digest: None,
                });
            }
        }
        self
    }

    pub fn with_basis_mismatch(
        mut self,
        expected_basis_digest: impl Into<String>,
        observed_basis_digest: impl Into<String>,
    ) -> Self {
        self.facts = Some(WorthServerProductOperationDenialFacts {
            code: WorthServerProductOperationDenialCode::ProductSemantic,
            expected_basis_digest: Some(expected_basis_digest.into()),
            observed_basis_digest: Some(observed_basis_digest.into()),
        });
        self
    }

    pub fn reason_key(&self) -> &str {
        &self.reason_key
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn facts(&self) -> Option<&WorthServerProductOperationDenialFacts> {
        self.facts.as_ref()
    }
}
