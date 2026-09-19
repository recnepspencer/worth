use std::net::SocketAddr;
use std::time::Duration;

use bank_external_rail::{
    dispatch, inquire_status, LedgerStatus, RailCorrelation, RailDispatch, RailEffectPayload,
    RailExchangeOutcome, RailRejection,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};

pub(super) struct BankProcessRailTransport {
    address: SocketAddr,
}

impl BankProcessRailTransport {
    pub(super) const fn connected_to(address: SocketAddr) -> Self {
        Self { address }
    }
}

impl WorthQueryExternalEffectTransport for BankProcessRailTransport {
    fn dispatch(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        let correlation = RailCorrelation::new(
            request.correlation_family().as_str(),
            request.correlation_token().to_vec(),
        );
        let outbound = RailDispatch {
            correlation: correlation.clone(),
            payload: RailEffectPayload::new(
                request.effect(),
                request.protocol_identity().clone(),
                request.protocol_version(),
                request.maximum_payload_bytes(),
                request.payload(),
            ),
        };
        let address = self.address;
        let exchange = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|_| ())?;
                    let observed =
                        runtime.block_on(dispatch(address, outbound, Duration::from_secs(5)));
                    let status = if matches!(
                        observed,
                        RailExchangeOutcome::Disconnected
                            | RailExchangeOutcome::DuplicateAcknowledgement
                    ) {
                        runtime
                            .block_on(inquire_status(address, correlation, Duration::from_secs(5)))
                            .ok()
                    } else {
                        None
                    };
                    Ok((observed, status))
                })
                .join()
        });
        match exchange {
            Ok(Ok((RailExchangeOutcome::Completed, _))) => {
                WorthQueryExternalTransportOutcome::Completed
            }
            Ok(Ok((RailExchangeOutcome::Acknowledged, _))) => {
                WorthQueryExternalTransportOutcome::Acknowledged
            }
            Ok(Ok((
                RailExchangeOutcome::Rejected(RailRejection::UnsupportedProtocolVersion(version)),
                _,
            ))) => WorthQueryExternalTransportOutcome::UnsupportedProtocolVersion(version),
            Ok(Ok((RailExchangeOutcome::Rejected(_), _))) => {
                WorthQueryExternalTransportOutcome::Rejected
            }
            Ok(Ok((
                RailExchangeOutcome::DuplicateAcknowledgement,
                Some(LedgerStatus::Completed),
            ))) => WorthQueryExternalTransportOutcome::Completed,
            Ok(Ok((RailExchangeOutcome::DuplicateAcknowledgement, _))) => {
                WorthQueryExternalTransportOutcome::DuplicateAcknowledgement
            }
            Ok(Ok((RailExchangeOutcome::TimedOut, _))) => {
                WorthQueryExternalTransportOutcome::TimedOut
            }
            Ok(Ok((
                RailExchangeOutcome::Disconnected,
                Some(LedgerStatus::Acknowledged | LedgerStatus::Completed),
            ))) => WorthQueryExternalTransportOutcome::LostResponse,
            Ok(Ok((RailExchangeOutcome::Disconnected, Some(LedgerStatus::NoRecord))))
            | Ok(Err(())) => WorthQueryExternalTransportOutcome::Disconnected,
            Ok(Ok((RailExchangeOutcome::Disconnected, None))) | Err(_) => {
                WorthQueryExternalTransportOutcome::LostResponse
            }
        }
    }
}
