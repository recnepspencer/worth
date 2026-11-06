use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Debug, Eq, PartialEq)]
pub struct SubscriptionConsumerAttachmentRequest {
    consumer_identity: WorthQueryEvidenceIdentity,
    delivery_cursor_seed_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionConsumerAttachmentRequest {
    pub fn admitted(
        consumer_id: impl Into<String>,
        delivery_cursor_seed: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let consumer_id = consumer_id.into();
        let delivery_cursor_seed = delivery_cursor_seed.into();
        let consumer_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_consumer_attachment_request_v1",
        )
        .field_value(WorthQueryEvidenceTag::new("consumer"), consumer_id)
        .field_value(
            WorthQueryEvidenceTag::new("cursor_seed"),
            &delivery_cursor_seed,
        )
        .seal()?;
        let delivery_cursor_seed_identity = delivery_cursor_seed_identity(&delivery_cursor_seed)?;
        Ok(Self {
            consumer_identity,
            delivery_cursor_seed_identity,
        })
    }

    pub fn from_consumer_identity(
        consumer_identity: WorthQueryEvidenceIdentity,
        delivery_cursor_seed_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self {
            consumer_identity,
            delivery_cursor_seed_identity,
        }
    }

    pub fn consumer_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.consumer_identity
    }

    pub fn delivery_cursor_seed_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_cursor_seed_identity
    }
}

fn delivery_cursor_seed_identity(
    seed: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_delivery_cursor_seed_v1",
            )
            .field_value(WorthQueryEvidenceTag::new("seed"), seed)
            .seal()?,
    )
}
