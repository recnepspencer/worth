pub(crate) struct UiPresentationAsyncRuntime {
    owner: worth_ui_query_binding::WorthUiPresentationAsyncOwner,
    correspondence: worth_ui_query_binding::WorthUiPresentationCorrespondenceIssuer,
    last_current_presented: Option<UiDeferredPresentedCompletion>,
}

struct UiDeferredPresentedCompletion {
    receipt: worth_ui_query_binding::WorthUiPresentationPendingReceipt,
    payload_byte_len: u64,
}

pub(crate) enum UiPresentationAsyncPresentedAdmission {
    Current,
    Superseded,
}

pub(crate) struct UiPresentationAsyncTerminalCleanup {
    runtime: Box<UiPresentationAsyncRuntime>,
}

pub(crate) struct UiPresentationAsyncTerminalCloseReceipt {
    query: worth_ui_query_binding::WorthUiPresentationAsyncCloseReceipt,
}

impl std::fmt::Debug for UiPresentationAsyncTerminalCleanup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("UiPresentationAsyncTerminalCleanup(..)")
    }
}

#[derive(Debug)]
pub(crate) enum UiPresentationAsyncPendingDenial {
    Issuance,
    Admission(Box<worth_ui_query_binding::WorthUiPresentationPendingAdmissionDenial>),
}

impl UiPresentationAsyncPendingDenial {
    pub(crate) fn into_recovery_receipt(
        self,
    ) -> Option<worth_ui_query_binding::WorthUiPresentationAdmissionRecovery> {
        match self {
            Self::Admission(denial) => (*denial).into_recovery_receipt(),
            Self::Issuance => None,
        }
    }
}

impl UiPresentationAsyncRuntime {
    pub(crate) fn from_installation(
        installation: worth_ui_query_binding::WorthUiPresentationAsyncInstallation,
    ) -> Self {
        let (owner, correspondence) = installation.into_runtime_parts();
        Self {
            owner,
            correspondence,
            last_current_presented: None,
        }
    }

    pub(crate) fn admit_pending(
        &mut self,
        basis: worth_ui_query_binding::WorthUiPresentationRequestBasis,
    ) -> Result<
        worth_ui_query_binding::WorthUiPresentationPendingReceipt,
        UiPresentationAsyncPendingDenial,
    > {
        let correspondence = self
            .correspondence
            .issue(basis)
            .map_err(|_| UiPresentationAsyncPendingDenial::Issuance)?;
        self.owner
            .admit_pending(correspondence)
            .map_err(|denial| UiPresentationAsyncPendingDenial::Admission(Box::new(denial)))
    }

    pub(crate) fn admit_presented_after_validation(
        &mut self,
        receipt: worth_ui_query_binding::WorthUiPresentationPendingReceipt,
        payload_byte_len: u64,
    ) -> Result<
        UiPresentationAsyncPresentedAdmission,
        (
            worth_ui_query_binding::WorthUiPresentationPendingReceipt,
            worth_ui_query_binding::WorthUiPresentationSettlementDenial,
        ),
    > {
        self.settle_presented(UiDeferredPresentedCompletion {
            receipt,
            payload_byte_len,
        })
    }

    pub(crate) fn admit_superseded_after_physical_observation(
        &mut self,
        receipt: worth_ui_query_binding::WorthUiPresentationPendingReceipt,
        observed_payload_byte_len: u64,
    ) -> Result<
        worth_ui_query_binding::WorthUiPresentationPresentedReceipt,
        (
            worth_ui_query_binding::WorthUiPresentationPendingReceipt,
            worth_ui_query_binding::WorthUiPresentationSettlementDenial,
        ),
    > {
        let observation = self
            .correspondence
            .certify_superseded_physical(&receipt, observed_payload_byte_len);
        match self.owner.admit_superseded_physical(&receipt, observation) {
            Ok(settled) => Ok(settled),
            Err(denial) => Err((receipt, denial)),
        }
    }

    fn settle_presented(
        &mut self,
        presented: UiDeferredPresentedCompletion,
    ) -> Result<
        UiPresentationAsyncPresentedAdmission,
        (
            worth_ui_query_binding::WorthUiPresentationPendingReceipt,
            worth_ui_query_binding::WorthUiPresentationSettlementDenial,
        ),
    > {
        let completion = self
            .correspondence
            .certify_presented(&presented.receipt, presented.payload_byte_len);
        match self.owner.admit_presented(&presented.receipt, completion) {
            Ok(receipt) => match receipt.observation().posture() {
                worth_ui_query_binding::WorthUiPresentationAsyncPosture::Current => {
                    self.last_current_presented = Some(presented);
                    Ok(UiPresentationAsyncPresentedAdmission::Current)
                }
                worth_ui_query_binding::WorthUiPresentationAsyncPosture::Superseded => {
                    Ok(UiPresentationAsyncPresentedAdmission::Superseded)
                }
                posture => {
                    unreachable!("presented completion cannot settle into {posture:?} posture")
                }
            },
            Err(denial) => Err((presented.receipt, denial)),
        }
    }

    pub(crate) fn admit_duplicate_owner_observation(
        &mut self,
        presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
    ) -> Result<(), worth_ui_query_binding::WorthUiPresentationSettlementDenial> {
        let presented = self
            .last_current_presented
            .as_ref()
            .filter(|presented| {
                presented.receipt.attempt() == presentation.attempt()
                    && presented.receipt.binding() == presentation.binding()
            })
            .ok_or(
                worth_ui_query_binding::WorthUiPresentationSettlementDenial::InvalidPendingReceipt,
            )?;
        let duplicate = self
            .correspondence
            .certify_presented(&presented.receipt, presented.payload_byte_len);
        match self.owner.admit_presented(&presented.receipt, duplicate) {
            Err(
                worth_ui_query_binding::WorthUiPresentationSettlementDenial::InvalidPendingReceipt,
            ) => Ok(()),
            Err(denial) => Err(denial),
            Ok(_) => Err(
                worth_ui_query_binding::WorthUiPresentationSettlementDenial::InvalidPendingReceipt,
            ),
        }
    }

    pub(crate) fn admit_effects_indeterminate_requiring_reconstruction(
        &mut self,
        receipt: &worth_ui_query_binding::WorthUiPresentationPendingReceipt,
        observed_payload_byte_len: u64,
    ) -> Result<
        worth_ui_query_binding::WorthUiPresentationRecoveryRequiredReceipt,
        worth_ui_query_binding::WorthUiPresentationSettlementDenial,
    > {
        let observation = self
            .correspondence
            .certify_effects_indeterminate(receipt, observed_payload_byte_len);
        self.owner
            .admit_effects_indeterminate_requiring_reconstruction(receipt, observation)
    }

    pub(crate) fn reject_recovery_before_effects(
        &mut self,
        receipt: &worth_ui_query_binding::WorthUiPresentationRecoveryReceipt,
    ) -> Result<(), worth_ui_query_binding::WorthUiPresentationSettlementDenial> {
        match receipt {
            worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Pending(receipt) => {
                self.owner.reject_before_effects(receipt)
            }
            worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Admission(receipt) => {
                self.owner.reject_admission_recovery_before_effects(receipt)
            }
        }
    }

    pub(crate) fn cancel_recovery_before_effects(
        &mut self,
        receipt: &worth_ui_query_binding::WorthUiPresentationRecoveryReceipt,
    ) -> Result<(), worth_ui_query_binding::WorthUiPresentationSettlementDenial> {
        match receipt {
            worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Pending(receipt) => {
                self.owner.cancel_before_effects(receipt)
            }
            worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Admission(receipt) => {
                self.owner.reject_admission_recovery_before_effects(receipt)
            }
        }
    }

    pub(crate) fn cancel_recovery_after_effects_may_have_begun(
        &mut self,
        receipt: &worth_ui_query_binding::WorthUiPresentationRecoveryReceipt,
        observed_payload_byte_len: u64,
    ) -> Result<
        worth_ui_query_binding::WorthUiPresentationRecoveryRequiredReceipt,
        worth_ui_query_binding::WorthUiPresentationSettlementDenial,
    > {
        match receipt {
            worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Pending(receipt) => {
                let observation = self
                    .correspondence
                    .certify_cancellation_effects_may_have_begun(
                        receipt,
                        observed_payload_byte_len,
                    );
                self.owner
                    .cancel_after_effects_may_have_begun_requiring_reconstruction(
                        receipt,
                        observation,
                    )
            }
            worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Admission(_) => Err(
                worth_ui_query_binding::WorthUiPresentationSettlementDenial::InvalidPendingReceipt,
            ),
        }
    }

    pub(crate) fn close_terminal_resources(
        &mut self,
    ) -> Result<
        worth_ui_query_binding::WorthUiPresentationAsyncCloseReceipt,
        worth_ui_query_binding::WorthUiPresentationAsyncCloseDenial,
    > {
        self.owner.close_terminal_resources()
    }

    pub(crate) fn into_terminal_close(
        mut self,
    ) -> Result<UiPresentationAsyncTerminalCloseReceipt, UiPresentationAsyncTerminalCleanup> {
        match self.close_terminal_resources() {
            Ok(query) => Ok(UiPresentationAsyncTerminalCloseReceipt { query }),
            Err(_) => Err(UiPresentationAsyncTerminalCleanup {
                runtime: Box::new(self),
            }),
        }
    }
}

impl UiPresentationAsyncTerminalCleanup {
    pub(crate) fn retry(
        self,
    ) -> Result<UiPresentationAsyncTerminalCloseReceipt, UiPresentationAsyncTerminalCleanup> {
        (*self.runtime).into_terminal_close()
    }
}

impl UiPresentationAsyncTerminalCloseReceipt {
    pub(crate) const fn closed_query_resources(&self) -> u64 {
        self.query.closed_query_resources()
    }

    pub(crate) fn transitions(
        &self,
    ) -> &[worth_ui_query_binding::WorthUiPresentationTransitionObservation] {
        self.query.transitions()
    }

    pub(crate) const fn transition_trace_complete(&self) -> bool {
        self.query.transition_trace_complete()
    }
}
