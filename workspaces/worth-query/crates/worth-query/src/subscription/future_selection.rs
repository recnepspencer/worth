use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct QuerySubscriptionAsyncRequestIdentityPart {
    key: String,
    value: crate::application::WorthQueryAsyncRequestIdentityValue,
}

impl QuerySubscriptionAsyncRequestIdentityPart {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: crate::application::WorthQueryAsyncRequestIdentityValue::Text(value.into()),
        }
    }

    pub(crate) fn from_async_identity_part(
        part: &crate::application::WorthQueryAsyncRequestIdentityPart,
    ) -> Self {
        Self {
            key: part.key().to_owned(),
            value: part.value().clone(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &crate::application::WorthQueryAsyncRequestIdentityValue {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuerySubscriptionFutureSelectionClass {
    Ordinary,
    Temporal,
    AsyncResource,
    TemporalAsync,
}

impl QuerySubscriptionFutureSelectionClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::Temporal => "temporal",
            Self::AsyncResource => "async_resource",
            Self::TemporalAsync => "temporal_async",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionFutureSelection {
    class: QuerySubscriptionFutureSelectionClass,
    requests_completion_lifecycle: bool,
    async_request_identity: Vec<QuerySubscriptionAsyncRequestIdentityPart>,
    projection_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionFutureSelection {
    pub fn ordinary() -> Self {
        Self::new(
            QuerySubscriptionFutureSelectionClass::Ordinary,
            false,
            Vec::new(),
        )
    }

    pub fn temporal() -> Self {
        Self::new(
            QuerySubscriptionFutureSelectionClass::Temporal,
            false,
            Vec::new(),
        )
    }

    pub fn async_resource(requests_completion_lifecycle: bool) -> Self {
        Self::async_resource_with_identity(requests_completion_lifecycle, Vec::new())
    }

    pub fn async_resource_with_identity(
        requests_completion_lifecycle: bool,
        async_request_identity: Vec<QuerySubscriptionAsyncRequestIdentityPart>,
    ) -> Self {
        Self::new(
            QuerySubscriptionFutureSelectionClass::AsyncResource,
            requests_completion_lifecycle,
            async_request_identity,
        )
    }

    pub fn temporal_async(requests_completion_lifecycle: bool) -> Self {
        Self::temporal_async_with_identity(requests_completion_lifecycle, Vec::new())
    }

    pub fn temporal_async_with_identity(
        requests_completion_lifecycle: bool,
        async_request_identity: Vec<QuerySubscriptionAsyncRequestIdentityPart>,
    ) -> Self {
        Self::new(
            QuerySubscriptionFutureSelectionClass::TemporalAsync,
            requests_completion_lifecycle,
            async_request_identity,
        )
    }

    fn new(
        class: QuerySubscriptionFutureSelectionClass,
        requests_completion_lifecycle: bool,
        async_request_identity: Vec<QuerySubscriptionAsyncRequestIdentityPart>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let async_request_identity = normalize_async_request_identity(async_request_identity);
        let mut projection_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_future_selection_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("class"), class.as_str())
        .field_usize(
            WorthQueryEvidenceTag::new("async_request_width"),
            async_request_identity.len(),
        );
        if requests_completion_lifecycle {
            projection_identity = projection_identity.field_shape(
                WorthQueryEvidenceTag::new("completion_lifecycle"),
                "requested",
            );
        }
        projection_identity = projection_identity.field_value_sequence(
            WorthQueryEvidenceTag::new("async_keys"),
            async_request_identity.iter().map(|part| part.key()),
        );
        let async_values = async_request_identity
            .iter()
            .map(|part| part.value().reporting_value())
            .collect::<Vec<_>>();
        projection_identity = projection_identity.field_value_sequence(
            WorthQueryEvidenceTag::new("async_values"),
            async_values.iter().map(String::as_str),
        );
        let projection_identity = projection_identity.seal()?;
        Ok(Self {
            class,
            requests_completion_lifecycle,
            async_request_identity,
            projection_identity,
        })
    }

    pub fn class(&self) -> QuerySubscriptionFutureSelectionClass {
        self.class
    }

    pub fn requests_completion_lifecycle(&self) -> bool {
        self.requests_completion_lifecycle
    }

    pub fn async_request_identity(&self) -> &[QuerySubscriptionAsyncRequestIdentityPart] {
        &self.async_request_identity
    }

    pub fn projection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.projection_identity
    }
    pub fn retained_facts(&self) -> Vec<String> {
        let mut facts = vec![format!("future-selection-class:{}", self.class.as_str())];
        if self.requests_completion_lifecycle {
            facts.push("future-selection-completion-lifecycle:requested".to_string());
        }
        facts.extend(self.async_request_identity.iter().map(|part| {
            format!(
                "future-selection-async-request-identity:{}={}",
                part.key(),
                part.value().reporting_value()
            )
        }));
        facts
    }
}

fn normalize_async_request_identity(
    async_request_identity: Vec<QuerySubscriptionAsyncRequestIdentityPart>,
) -> Vec<QuerySubscriptionAsyncRequestIdentityPart> {
    let mut async_request_identity = async_request_identity;
    async_request_identity.sort();
    async_request_identity.dedup();
    async_request_identity
}
