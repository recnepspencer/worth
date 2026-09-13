use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::worth_query_structured_value_binding;
use worth_query_decl::facade::{
    application_schema::{
        ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
        ApplicationRetainedEffectBinding,
    },
    worth_query_effect,
};

use crate::{estate::EstateDeathNotificationRequest, schema::BankSchema};

worth_query_structured_value_binding!(pub EstateDeathNotificationRequestBinding for EstateDeathNotificationRequest { identity: "bank.estate.effect.death-notification.payload.v1" });

impl ApplicationRetainedEffectBinding for EstateDeathNotificationRequestBinding {
    fn retained_bytes(_: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<EstateDeathNotificationRequest>()).unwrap_or(u64::MAX)
    }
}

impl ApplicationExternalEffectBinding for EstateDeathNotificationRequestBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("bank.estate.death-notification"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 24;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MAX_EXTERNAL_BYTES as usize);
        bytes.extend_from_slice(&value.estate().get().to_be_bytes());
        bytes.extend_from_slice(&value.notice().get().to_be_bytes());
        bytes.extend_from_slice(&value.subject().get().to_be_bytes());
        bytes
    }
}

worth_query_effect!(pub EstateDeathNotificationEffect for BankSchema, payload EstateDeathNotificationRequestBinding);

#[cfg(test)]
mod tests {
    use worth_query_decl::facade::application_schema::{
        ApplicationExternalEffectBinding, ApplicationRetainedEffectBinding,
    };

    use super::{EstateDeathNotificationRequest, EstateDeathNotificationRequestBinding};
    use crate::{
        estate::{DeathNoticeId, EstateCaseId},
        model::BankPrincipalId,
    };

    #[test]
    fn death_notification_request_retains_exact_fixed_width() {
        let request = EstateDeathNotificationRequest::new(
            EstateCaseId::new(1).unwrap(),
            DeathNoticeId::new(2).unwrap(),
            BankPrincipalId::new(3).unwrap(),
        );

        assert_eq!(
            EstateDeathNotificationRequestBinding::retained_bytes(&request),
            u64::try_from(std::mem::size_of::<EstateDeathNotificationRequest>()).unwrap()
        );
    }

    #[test]
    fn v1_encoder_matches_the_frozen_external_corpus() {
        let request = EstateDeathNotificationRequest::new(
            EstateCaseId::new(8_101).unwrap(),
            DeathNoticeId::new(8_102).unwrap(),
            BankPrincipalId::new(8_103).unwrap(),
        );
        let corpus =
            include_str!("../../../../../../protocol-corpus/estate-death-notification/v1.hex");
        assert_eq!(
            EstateDeathNotificationRequestBinding::external_effect_bytes(&request),
            decode_hex(corpus)
        );
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
