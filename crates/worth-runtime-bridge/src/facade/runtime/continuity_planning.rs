use super::*;

impl RuntimeBridge {
    /// Returns the configured continuity lineage source, if one is bound.
    pub fn continuity_lineage_source(&self) -> Option<&dyn ContinuityLineageSource> {
        self.continuity_lineage_source.as_deref()
    }

    /// Plans continuity requests from one retained route record.
    pub fn plan_continuity_requests(
        &self,
        prior_route_record: &BridgeRouteRecord,
    ) -> Result<BridgeEligibleContinuityRequestSet, BridgeContinuityError> {
        let planned = crate::continuity::BridgePlannedContinuityRequestSet::from_route_record(
            prior_route_record,
        )?;
        crate::continuity::BridgeEligibleContinuityRequestSet::from_planned(planned)
    }

    /// Plans continuity requests directly from a planned route before delivery.
    pub fn plan_continuity_requests_from_planned_route(
        &self,
        planned_route: &BridgePlannedRoute,
    ) -> Result<BridgeEligibleContinuityRequestSet, BridgeContinuityError> {
        let planned = crate::continuity::BridgePlannedContinuityRequestSet::from_planned_route(
            planned_route,
        )?;
        crate::continuity::BridgeEligibleContinuityRequestSet::from_planned(planned)
    }

    /// Materializes one historical lineage packet from eligible continuity requests.
    pub fn plan_historical_lineage_packet(
        &self,
        requests: &BridgeEligibleContinuityRequestSet,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineagePacket, BridgeContinuityError> {
        let source = self.continuity_lineage_source().ok_or_else(|| {
            BridgeContinuityError::new(
                BridgeContinuityErrorKind::MissingLineageSource,
                "Bridge historical lineage planning requires a configured continuity lineage source.",
            )
        })?;
        let mut entries = Vec::with_capacity(requests.requests().len());
        for request in requests.requests() {
            let lineage_authority = source
                .historical_lineage(
                    BridgeHistoricalLineageRequest::new(
                        requests.authority_basis().clone(),
                        request.prior_slice().clone(),
                    ),
                    execution,
                )
                .map_err(|error| map_lineage_source_error(request, error))?;
            ensure_lineage_authority_matches_request_set(requests, request, &lineage_authority)?;
            entries.push(crate::continuity::BridgeHistoricalLineagePacketEntry::new(
                request.correlation_id().clone(),
                request.prior_slice().clone(),
                lineage_authority,
            ));
        }
        Ok(crate::continuity::BridgeHistoricalLineagePacket::from_entries(requests, entries))
    }
}

fn map_lineage_source_error(
    request: &BridgePlannedContinuityRequest,
    error: BridgeLineageSourceError,
) -> BridgeContinuityError {
    match error.kind() {
        BridgeLineageSourceErrorKind::ObservationRead(denial) => BridgeContinuityError::new(BridgeContinuityErrorKind::ObservationRead(denial), error.to_string()),
        BridgeLineageSourceErrorKind::UnsupportedContinuityClass => BridgeContinuityError::new(
            BridgeContinuityErrorKind::UnsupportedContinuityClass,
            format!(
                "Bridge continuity correlation `{}` targeted an unsupported continuity class: {error}",
                request.correlation_id().as_str()
            ),
        ),
        BridgeLineageSourceErrorKind::HistoricalResolutionFailure
        | BridgeLineageSourceErrorKind::DuplicateResolvedLineageIdentities
        | BridgeLineageSourceErrorKind::NonCanonicalResolvedLineageIdentities
        | BridgeLineageSourceErrorKind::DuplicateResolvedRecordIdentities
        | BridgeLineageSourceErrorKind::NonCanonicalResolvedRecordIdentities
        | BridgeLineageSourceErrorKind::DuplicateTraversedEventIds
        | BridgeLineageSourceErrorKind::NonCanonicalTraversedEventIds => {
            BridgeContinuityError::new(
                BridgeContinuityErrorKind::HistoricalResolutionFailure,
                format!(
                    "Bridge failed to resolve historical lineage for continuity correlation `{}`: {error}",
                    request.correlation_id().as_str()
                ),
            )
        }
    }
}

fn ensure_lineage_authority_matches_request_set(
    requests: &BridgeEligibleContinuityRequestSet,
    request: &BridgePlannedContinuityRequest,
    lineage_authority: &BridgeHistoricalLineageAuthority,
) -> Result<(), BridgeContinuityError> {
    if lineage_authority.authority_basis() == requests.authority_basis() {
        return Ok(());
    }
    Err(BridgeContinuityError::new(
        BridgeContinuityErrorKind::LineageAuthorityMismatch,
        format!(
            "Bridge historical lineage authority for continuity correlation `{}` did not match the planned branch/snapshot authority basis.",
            request.correlation_id().as_str()
        ),
    ))
}
