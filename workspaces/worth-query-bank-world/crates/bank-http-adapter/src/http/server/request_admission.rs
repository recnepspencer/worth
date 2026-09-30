use std::time::{Duration, Instant};

use bank_domain::model::AccountId;

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind, BankHttpNextAction, BankHttpProtocolVersion,
};

pub(super) struct UnadmittedBankHttpRequestBasis {
    pub(super) protocol: BankHttpProtocolVersion,
    pub(super) request_id: String,
    pub(super) account: String,
    pub(super) deadline_milliseconds: u64,
}

pub(super) struct UnadmittedBankHttpControls {
    pub(super) protocol: BankHttpProtocolVersion,
    pub(super) request_id: String,
    pub(super) deadline_milliseconds: u64,
}

pub(super) struct AdmittedBankHttpControls {
    pub(super) request_id: String,
    pub(super) deadline: Instant,
}

pub(super) struct AdmittedBankHttpRequestBasis {
    pub(super) request_id: String,
    pub(super) account: AccountId,
    pub(super) deadline: Instant,
}

pub(super) struct RejectedBankHttpRequest {
    pub(super) request_id: Option<String>,
    pub(super) denial: BankHttpDenial,
}

impl UnadmittedBankHttpRequestBasis {
    pub(super) fn admit(
        self,
        maximum_deadline: Duration,
    ) -> Result<AdmittedBankHttpRequestBasis, RejectedBankHttpRequest> {
        let controls = UnadmittedBankHttpControls {
            protocol: self.protocol,
            request_id: self.request_id,
            deadline_milliseconds: self.deadline_milliseconds,
        }
        .admit(maximum_deadline)?;
        let request_id = controls.request_id;
        let Some(account) = AccountId::parse_canonical_text(&self.account) else {
            return Err(malformed(Some(request_id)));
        };
        Ok(AdmittedBankHttpRequestBasis {
            request_id,
            account,
            deadline: controls.deadline,
        })
    }
}

impl UnadmittedBankHttpControls {
    pub(super) fn admit(
        self,
        maximum_deadline: Duration,
    ) -> Result<AdmittedBankHttpControls, RejectedBankHttpRequest> {
        if self.protocol != BankHttpProtocolVersion::V2 {
            return Err(RejectedBankHttpRequest {
                request_id: Some(self.request_id),
                denial: BankHttpDenial::new(
                    BankHttpDenialKind::UnsupportedProtocol,
                    BankHttpNextAction::CorrectRequest,
                ),
            });
        }
        if self.request_id.is_empty()
            || self.request_id.len() > 128
            || self.request_id.chars().any(char::is_control)
        {
            return Err(malformed(None));
        }
        let request_id = self.request_id;
        let deadline_duration = Duration::from_millis(self.deadline_milliseconds);
        if deadline_duration.is_zero() || deadline_duration > maximum_deadline {
            return Err(malformed(Some(request_id)));
        }
        let Some(deadline) = Instant::now().checked_add(deadline_duration) else {
            return Err(malformed(Some(request_id)));
        };
        Ok(AdmittedBankHttpControls {
            request_id,
            deadline,
        })
    }
}

fn malformed(request_id: Option<String>) -> RejectedBankHttpRequest {
    RejectedBankHttpRequest {
        request_id,
        denial: BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
    }
}
