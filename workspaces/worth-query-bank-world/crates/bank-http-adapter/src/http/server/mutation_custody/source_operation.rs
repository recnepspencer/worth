//! Local expansion of the native source owner states and their delivery modes.
macro_rules! custody {
    (@advance retained, $operation:ident, $request:ident, $history:ident) => {
        $operation.advance($request, Some($history))
    };
    (@advance source, $operation:ident, $request:ident, $history:ident) => {{
        let _ = $history;
        $operation.advance($request)
    }};
    ($state:ident, $input:ty, $action:ty, $mode:ty, $advance:ident) => {
        pub(super) enum $state {
            Active(
                WorthQuerySourceOnlyOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    $mode,
                >,
            ),
            Completed(
                WorthQueryCompletedSourceOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    $mode,
                >,
            ),
            Terminal(
                WorthQueryTerminalSourceOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    $mode,
                >,
            ),
        }
        impl $state {
            fn advance(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
                history: NonZeroUsize,
            ) -> (Self, Option<BankHttpDenial>) {
                match self {
                    Self::Active(operation) => match custody!(@advance $advance, operation, request, history) {
                        WorthQuerySourceOnlyAdvance::Progress(operation) => {
                            (Self::Active(operation), None)
                        }
                        WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                            (Self::Active(operation), Some(source_denial(denial)))
                        }
                        WorthQuerySourceOnlyAdvance::Completed(operation) => {
                            (Self::Completed(operation), None)
                        }
                        WorthQuerySourceOnlyAdvance::Terminal(operation) => {
                            (Self::Terminal(operation), None)
                        }
                    },
                    owner => (owner, None),
                }
            }
            fn close(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
            ) -> Result<(), (Self, BankHttpDenial)> {
                match self {
                    Self::Active(operation) => {
                        operation
                            .close(request)
                            .map(|_| ())
                            .map_err(|(owner, denial)| {
                                (
                                    Self::Active(owner),
                                    super::mutation_refusal::close_denial(denial),
                                )
                            })
                    }
                    Self::Completed(operation) => {
                        operation
                            .close(request)
                            .map(|_| ())
                            .map_err(|(owner, denial)| {
                                (
                                    Self::Completed(owner),
                                    super::mutation_refusal::close_denial(denial),
                                )
                            })
                    }
                    Self::Terminal(operation) => {
                        operation
                            .close(request)
                            .map(|_| ())
                            .map_err(|(owner, denial)| {
                                (
                                    Self::Terminal(owner),
                                    super::mutation_refusal::close_denial(denial),
                                )
                            })
                    }
                }
            }
            fn release_cleanup(
                &mut self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
            ) -> Result<WorthQueryOperationCleanupReceipt, BankHttpDenial> {
                match self {
                    Self::Active(operation) => operation.release_cleanup(request),
                    Self::Terminal(operation) => operation.release_cleanup(request),
                    Self::Completed(_) => {
                        return Err(super::mutation_refusal::denial(
                            super::super::protocol::BankHttpDenialKind::Stale,
                        ))
                    }
                }
                .map_err(super::mutation_refusal::close_denial)
            }
            fn observation(&self) -> Option<&WorthQueryApplicationReadObservation> {
                match self {
                    Self::Completed(operation) => match operation.evidence() {
                        WorthQuerySourceCompletion::Fresh { retained, .. }
                        | WorthQuerySourceCompletion::Recovered { retained, .. } => {
                            retained.as_ref()
                        }
                    },
                    _ => None,
                }
            }
            fn description(&self) -> CustodyDescription {
                match self {
                    Self::Active(operation) => CustodyDescription::Pending(
                        operation.posture().into(),
                        operation
                            .managed_cleanup_failure()
                            .map(super::cleanup_description::failure),
                    ),
                    Self::Terminal(_) => CustodyDescription::Terminal,
                    Self::Completed(operation) => {
                        let (receipt, recovered) = match operation.evidence() {
                            WorthQuerySourceCompletion::Fresh { receipt, .. } => (receipt, false),
                            WorthQuerySourceCompletion::Recovered { receipt, .. } => {
                                (receipt, true)
                            }
                        };
                        CustodyDescription::Completed {
                            recovered,
                            commit: describe_commit(receipt),
                            source: receipt.correction_source().coordinates(),
                        }
                    }
                }
            }
        }
    };
}
pub(super) use custody;
