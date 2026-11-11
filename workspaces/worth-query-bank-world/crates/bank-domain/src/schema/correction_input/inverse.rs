//! Exact performed inverse DATA, never a second inverse derivation.
use super::identities::*;
use crate::schema::{
    AccountStatus, RestoreFrozenEstateAccount, RestoreFrozenEstateAccountInputBinding,
    ReversalReason, ReverseJournal, ReverseJournalInputBinding,
};
use serde::{de::DeserializeSeed, Deserialize};
use worth_query_decl::facade::{application_operation::*, application_schema::*};
#[cfg(test)]
mod tests;

pub(in crate::schema) fn journal(
    bytes: &[u8],
    expected: CanonicalDigestId,
    bounds: ValueDecodeBounds,
) -> Result<ApplicationEncodedInput<ReverseJournalInputBinding>, ApplicationRetainedInputDenial> {
    ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, JournalSeed)
}
pub(in crate::schema) fn frozen(
    bytes: &[u8],
    expected: CanonicalDigestId,
    bounds: ValueDecodeBounds,
) -> Result<
    ApplicationEncodedInput<RestoreFrozenEstateAccountInputBinding>,
    ApplicationRetainedInputDenial,
> {
    ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, FrozenSeed)
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "ReversalReason")]
enum RecordedReason {
    Duplicate,
    OperatorCorrection,
    ExternalReturn,
}
impl RecordedReason {
    fn restore(self) -> ReversalReason {
        match self {
            Self::Duplicate => ReversalReason::Duplicate,
            Self::OperatorCorrection => ReversalReason::OperatorCorrection,
            Self::ExternalReturn => ReversalReason::ExternalReturn,
        }
    }
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "ReverseJournal")]
struct RecordedReverseJournal {
    institution: RecordedInstitutionId,
    journal: RecordedJournalId,
    reason: RecordedReason,
}
struct JournalSeed;
impl<'de> DeserializeSeed<'de> for JournalSeed {
    type Value = ReverseJournal;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        let value = RecordedReverseJournal::deserialize(decoder)?;
        Ok(ReverseJournal {
            institution: value
                .institution
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid institution identity"))?,
            journal: value
                .journal
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid journal identity"))?,
            reason: value.reason.restore(),
        })
    }
}
#[derive(Deserialize)]
#[serde(rename = "AccountStatus")]
enum RecordedStatus {
    Open,
    Frozen,
    Closed,
}
impl RecordedStatus {
    fn restore(self) -> AccountStatus {
        match self {
            Self::Open => AccountStatus::Open,
            Self::Frozen => AccountStatus::Frozen,
            Self::Closed => AccountStatus::Closed,
        }
    }
}
#[derive(Deserialize)]
#[serde(rename = "RestoreFrozenEstateAccount")]
struct RecordedFrozen {
    estate: RecordedEstateId,
    account: RecordedAccountId,
    prior_status: RecordedStatus,
}
struct FrozenSeed;
impl<'de> DeserializeSeed<'de> for FrozenSeed {
    type Value = RestoreFrozenEstateAccount;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        let value = RecordedFrozen::deserialize(decoder)?;
        Ok(RestoreFrozenEstateAccount {
            estate: value
                .estate
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid estate identity"))?,
            account: value
                .account
                .restore()
                .ok_or_else(|| serde::de::Error::custom("invalid account identity"))?,
            prior_status: value.prior_status.restore(),
        })
    }
}
