use super::super::ResourceRuntimeState;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;

enum CompletionEnvelopeValidation {
    Valid {
        validated: ValidatedCompletionEnvelope,
        in_flight: InFlightResourceRequest,
    },
    Denied {
        class: CompletionDenialClass,
        node: Option<ResourceNodeId>,
    },
}

impl ResourceRuntimeState {
    pub fn admit_resource_completion(
        &mut self,
        raw: RawCompletionEnvelope,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceCompletionAdmissionReport {
        self.admit_resource_completion_with_boundary(raw, telemetry, true)
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn admit_resource_completion_with_boundary(
        &mut self,
        raw: RawCompletionEnvelope,
        mut telemetry: Option<&mut ResourceTelemetry>,
        count_scalar_boundary: bool,
    ) -> ResourceCompletionAdmissionReport {
        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_completion_validation_count += 1;
            telemetry.resource_hot_in_flight_lookup_count += 1;
        }
        match self.validate_completion_envelope(&raw) {
            CompletionEnvelopeValidation::Denied { class, node } => {
                self.deny_completion(&raw, class, node, telemetry, count_scalar_boundary)
            }
            CompletionEnvelopeValidation::Valid {
                validated,
                in_flight,
            } => self.admit_validated_completion(
                validated,
                in_flight,
                telemetry,
                count_scalar_boundary,
            ),
        }
    }

    fn validate_completion_envelope(
        &self,
        raw: &RawCompletionEnvelope,
    ) -> CompletionEnvelopeValidation {
        let Some(in_flight) = self.in_flight_by_request.get(&raw.request_id()).cloned() else {
            if let Some(retained) = self
                .retained_in_flight_history_by_request
                .get(&raw.request_id())
                .cloned()
            {
                return CompletionEnvelopeValidation::Denied {
                    class: self.retained_completion_denial_class(raw, retained.clone()),
                    node: Some(retained.node()),
                };
            }
            if let Some(pruned) = self
                .pruned_in_flight_history_by_request
                .get(&raw.request_id())
                .cloned()
            {
                return CompletionEnvelopeValidation::Denied {
                    class: Self::pruned_completion_denial_class(raw, pruned),
                    node: None,
                };
            }
            return CompletionEnvelopeValidation::Denied {
                class: CompletionDenialClass::UnknownRequest,
                node: None,
            };
        };

        let handle = in_flight.handle();
        if handle.request_id() != raw.request_id()
            || handle.generation() != raw.generation()
            || handle.branch_epoch() != raw.branch_epoch()
            || in_flight.attempt() != raw.attempt()
        {
            return CompletionEnvelopeValidation::Denied {
                class: CompletionDenialClass::Stale,
                node: Some(in_flight.node()),
            };
        }

        let class = match in_flight.status() {
            ResourceInFlightStatus::Superseded => Some(CompletionDenialClass::Superseded),
            ResourceInFlightStatus::Cancelled => Some(CompletionDenialClass::Cancelled),
            ResourceInFlightStatus::Rejected => Some(CompletionDenialClass::Rejected),
            ResourceInFlightStatus::TimedOut => Some(CompletionDenialClass::TimedOut),
            ResourceInFlightStatus::Active
                if in_flight.lifecycle() == ResourceLifecycleClass::Pending =>
            {
                None
            }
            _ => Some(CompletionDenialClass::Retired),
        };
        if let Some(class) = class {
            return CompletionEnvelopeValidation::Denied {
                class,
                node: Some(in_flight.node()),
            };
        }

        let Some(descriptor) = self.descriptors.get(&in_flight.descriptor_id()) else {
            return CompletionEnvelopeValidation::Denied {
                class: CompletionDenialClass::Impossible,
                node: Some(in_flight.node()),
            };
        };
        if descriptor.payload_contract_digest() != raw.payload_contract_digest() {
            return CompletionEnvelopeValidation::Denied {
                class: CompletionDenialClass::Malformed,
                node: Some(in_flight.node()),
            };
        }
        if descriptor
            .max_payload_bytes()
            .is_some_and(|max| raw.payload_byte_len() > max)
        {
            return CompletionEnvelopeValidation::Denied {
                class: CompletionDenialClass::Partial,
                node: Some(in_flight.node()),
            };
        }

        CompletionEnvelopeValidation::Valid {
            validated: ValidatedCompletionEnvelope::new(
                handle,
                raw.attempt(),
                raw.payload_byte_len(),
            ),
            in_flight,
        }
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn admit_validated_completion(
        &mut self,
        validated: ValidatedCompletionEnvelope,
        in_flight: InFlightResourceRequest,
        mut telemetry: Option<&mut ResourceTelemetry>,
        count_scalar_boundary: bool,
    ) -> ResourceCompletionAdmissionReport {
        let lifecycle_ordinal = self.issue_lifecycle_ordinal();
        let completion_ordinal = self.issue_completion_ordinal();
        let transition = ResourceLifecycleTransition::new(
            in_flight.node(),
            ResourceLifecycleClass::Pending,
            ResourceLifecycleClass::Fulfilled,
            ResourceLifecycleTransitionKind::CompletionAdmitted,
            lifecycle_ordinal,
            ResourceOutputContinuity::OutputReplaced,
        );
        let admitted = AdmittedResourceCompletion::new(
            validated.handle(),
            validated.attempt(),
            in_flight.node(),
            in_flight.descriptor_id(),
            completion_ordinal,
            validated.payload_byte_len(),
            transition,
        );

        if count_scalar_boundary && let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_completion_admission_count += 1;
        }
        let performance = ResourceBoundaryPerformanceEnvelope::completion_admission(1, 0, 1)
            .with_density_strategy(ResourceDensityStrategy::scalar_completion());
        let performance = if count_scalar_boundary {
            telemetry
                .map(|telemetry| Self::record_boundary_performance(telemetry, performance))
                .unwrap_or(performance)
        } else {
            performance
        };

        ResourceCompletionAdmissionReport::admitted(admitted, performance)
    }
}
