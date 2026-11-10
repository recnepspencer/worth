//! Checked reconstruction of the three declared correctable source inputs.
mod identities;
mod inverse;
pub(super) use inverse::{frozen, journal};
#[cfg(test)]
mod tests;
use super::{
    ApprovePayment, ApprovePaymentInputBinding, EstateActionInputBinding, SendMoney,
    SendMoneyInputBinding,
};
use crate::{
    estate::EstateAction,
    model::{Money, USD},
};
use identities::*;
use serde::{de::DeserializeSeed, Deserialize};
use std::marker::PhantomData;
use worth_query_decl::facade::{application_operation::*, application_schema::*};

pub(super) fn sent(
    bytes: &[u8],
    expected: CanonicalDigestId,
    bounds: ValueDecodeBounds,
) -> Result<ApplicationEncodedInput<SendMoneyInputBinding>, ApplicationRetainedInputDenial> {
    ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, SentSeed)
}
pub(super) fn approved(
    bytes: &[u8],
    expected: CanonicalDigestId,
    bounds: ValueDecodeBounds,
) -> Result<ApplicationEncodedInput<ApprovePaymentInputBinding>, ApplicationRetainedInputDenial> {
    ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, ApprovedSeed)
}
pub(super) fn freeze(
    bytes: &[u8],
    expected: CanonicalDigestId,
    bounds: ValueDecodeBounds,
) -> Result<ApplicationEncodedInput<EstateActionInputBinding>, ApplicationRetainedInputDenial> {
    ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, FreezeSeed)
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "Money")]
struct RecordedMoney {
    minor_units: i64,
    _currency: PhantomData<()>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "SendMoney")]
struct RecordedSendMoney {
    from: RecordedAccountId,
    recipient: RecordedPrincipalId,
    amount: RecordedMoney,
}
struct SentSeed;
impl<'de> DeserializeSeed<'de> for SentSeed {
    type Value = SendMoney;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<SendMoney, D::Error> {
        let value = RecordedSendMoney::deserialize(decoder)?;
        Ok(SendMoney {
            from: value
                .from
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid account identity"))?,
            recipient: value
                .recipient
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid principal identity"))?,
            amount: Money::<USD>::from_minor(value.amount.minor_units)
                .map_err(serde::de::Error::custom)?,
        })
    }
}
#[derive(Deserialize)]
#[serde(rename = "ApprovePayment")]
struct RecordedApproval {
    payment: RecordedPaymentId,
    approver: RecordedPrincipalId,
}
struct ApprovedSeed;
impl<'de> DeserializeSeed<'de> for ApprovedSeed {
    type Value = ApprovePayment;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        decoder: D,
    ) -> Result<ApprovePayment, D::Error> {
        let value = RecordedApproval::deserialize(decoder)?;
        Ok(ApprovePayment {
            payment: value
                .payment
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid payment identity"))?,
            approver: value
                .approver
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid principal identity"))?,
        })
    }
}
#[derive(Deserialize)]
#[serde(rename = "EstateAction")]
enum RecordedFreeze {
    FreezeAccount {
        estate: RecordedEstateId,
        account: RecordedAccountId,
    },
}
struct FreezeSeed;
impl<'de> DeserializeSeed<'de> for FreezeSeed {
    type Value = EstateAction;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        decoder: D,
    ) -> Result<EstateAction, D::Error> {
        let RecordedFreeze::FreezeAccount { estate, account } =
            RecordedFreeze::deserialize(decoder)?;
        Ok(EstateAction::FreezeAccount {
            estate: estate
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid estate identity"))?,
            account: account
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid account identity"))?,
        })
    }
}
