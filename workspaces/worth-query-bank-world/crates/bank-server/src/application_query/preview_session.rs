//! Bank-owned read context for one exact product commit.

use bank_domain::schema::BankSchema;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryRequestInterruption, WorthQueryRequestScope},
    application_entry::WorthQueryApplicationReadObservation,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};

use super::{BankApplicationPreviewSessionDenialKind, BankApplicationQueryDenial};

/// Opaque Bank read context retaining one exact product occurrence while
/// every use receives fresh request admission.
pub struct BankPreviewSession {
    observation: WorthQueryApplicationReadObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPreviewSessionDiscardReceipt {
    discarded: bool,
}

impl BankPreviewSession {
    pub(crate) fn open(
        application: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
        request: &WorthQueryRequestScope,
    ) -> Result<Self, BankApplicationQueryDenial> {
        match request.interruption() {
            Some(WorthQueryRequestInterruption::Cancelled) => {
                return Err(BankApplicationQueryDenial::PreviewSession(
                    BankApplicationPreviewSessionDenialKind::Cancelled,
                ));
            }
            Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
                return Err(BankApplicationQueryDenial::PreviewSession(
                    BankApplicationPreviewSessionDenialKind::DeadlineExceeded,
                ));
            }
            None => {}
        }
        let branch = application.current_world();
        let observation =
            WorthQueryApplicationReadObservation::retain_on_branch(application, branch)
                .map_err(BankApplicationQueryDenial::from_product_selection)?;
        Ok(Self { observation })
    }

    pub(crate) const fn observation(&self) -> &WorthQueryApplicationReadObservation {
        &self.observation
    }

    pub fn discard(self) -> Result<BankPreviewSessionDiscardReceipt, BankApplicationQueryDenial> {
        Ok(BankPreviewSessionDiscardReceipt { discarded: true })
    }
}

impl BankPreviewSessionDiscardReceipt {
    pub const fn discarded(self) -> bool {
        self.discarded
    }
}
