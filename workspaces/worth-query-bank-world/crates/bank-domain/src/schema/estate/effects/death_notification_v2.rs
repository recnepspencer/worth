use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationRetainedEffectBinding,
};
use worth_query_decl::facade::worth_query_structured_value_binding;

use crate::estate::EstateDeathNotificationRequest;

/// Independently declared v2 producer projection for the death-notification
/// protocol family.
///
/// The active Bank schema still emits v1. This projection gives rollout code a
/// real v2 encoder to deploy independently while the rail admits both exact
/// versions during the compatibility window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateDeathNotificationV2Payload(EstateDeathNotificationRequest);

impl From<EstateDeathNotificationRequest> for EstateDeathNotificationV2Payload {
    fn from(request: EstateDeathNotificationRequest) -> Self {
        Self(request)
    }
}

worth_query_structured_value_binding!(pub EstateDeathNotificationV2PayloadBinding for EstateDeathNotificationV2Payload { identity: "bank.estate.effect.death-notification.payload.v2" });

impl ApplicationRetainedEffectBinding for EstateDeathNotificationV2PayloadBinding {
    fn retained_bytes(_: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<EstateDeathNotificationV2Payload>()).unwrap_or(u64::MAX)
    }
}

impl ApplicationExternalEffectBinding for EstateDeathNotificationV2PayloadBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("bank.estate.death-notification"),
        BoundaryProtocolVersion::new(2),
    );
    const MAX_EXTERNAL_BYTES: u64 = 32;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MAX_EXTERNAL_BYTES as usize);
        bytes.extend_from_slice(b"DEATHV2!");
        bytes.extend_from_slice(&value.0.estate().get().to_be_bytes());
        bytes.extend_from_slice(&value.0.notice().get().to_be_bytes());
        bytes.extend_from_slice(&value.0.subject().get().to_be_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use worth_query_decl::facade::application_schema::ApplicationExternalEffectBinding;

    use super::{EstateDeathNotificationV2Payload, EstateDeathNotificationV2PayloadBinding};
    use crate::{
        estate::{DeathNoticeId, EstateCaseId, EstateDeathNotificationRequest},
        model::BankPrincipalId,
    };

    #[test]
    fn v2_encoder_matches_the_frozen_external_corpus() {
        let payload = EstateDeathNotificationV2Payload::from(EstateDeathNotificationRequest::new(
            EstateCaseId::new(8_101).unwrap(),
            DeathNoticeId::new(8_102).unwrap(),
            BankPrincipalId::new(8_103).unwrap(),
        ));
        let corpus =
            include_str!("../../../../../../protocol-corpus/estate-death-notification/v2.hex");
        let encoded = EstateDeathNotificationV2PayloadBinding::external_effect_bytes(&payload);
        assert_eq!(
            EstateDeathNotificationV2PayloadBinding::PROTOCOL.identity(),
            &worth_foundational::facade::BoundaryProtocolIdentity::new(
                "bank.estate.death-notification"
            )
        );
        assert_eq!(
            EstateDeathNotificationV2PayloadBinding::PROTOCOL.version(),
            worth_foundational::facade::BoundaryProtocolVersion::new(2)
        );
        assert_eq!(
            EstateDeathNotificationV2PayloadBinding::MAX_EXTERNAL_BYTES,
            32
        );
        assert_eq!(
            encoded.len() as u64,
            EstateDeathNotificationV2PayloadBinding::MAX_EXTERNAL_BYTES
        );
        assert_eq!(encoded, decode_hex(corpus));
    }

    fn decode_hex(corpus: &str) -> Vec<u8> {
        let corpus = corpus.trim();
        assert!(corpus.len().is_multiple_of(2), "hex corpus has a remainder");
        let bytes = corpus
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(bytes.len() * 2, corpus.len());
        bytes
    }
}
