use super::{BridgeErrorKind, *};
use worth_execution::report::{DenialClass::*, Disposition::*};

impl BridgeErrorKind for BridgeBuildErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::MissingRelationalSource => (
                InvalidInput,
                CorrectInput,
                "bridge.build.missing_relational_source",
            ),
            Self::MissingSignalSink => (
                InvalidInput,
                CorrectInput,
                "bridge.build.missing_signal_sink",
            ),
            Self::MissingMappingRegistrations => (
                InvalidInput,
                CorrectInput,
                "bridge.build.missing_mapping_registrations",
            ),
            Self::MissingSourceAdapter => (
                InvalidInput,
                CorrectInput,
                "bridge.build.missing_source_adapter",
            ),
            Self::DuplicateMappingRegistration => (
                Conflict,
                Final,
                "bridge.build.duplicate_mapping_registration",
            ),
            Self::AmbiguousMappingRegistration => (
                Conflict,
                Final,
                "bridge.build.ambiguous_mapping_registration",
            ),
            Self::DuplicateAspectRegistration => (
                Conflict,
                Final,
                "bridge.build.duplicate_aspect_registration",
            ),
            Self::AmbiguousAspectRegistration => (
                Conflict,
                Final,
                "bridge.build.ambiguous_aspect_registration",
            ),
            Self::DuplicateSourceDeclaration => {
                (Conflict, Final, "bridge.build.duplicate_source_declaration")
            }
            Self::AmbiguousSourceDeclaration => {
                (Conflict, Final, "bridge.build.ambiguous_source_declaration")
            }
            Self::DuplicateStructuralDeclaration => (
                Conflict,
                Final,
                "bridge.build.duplicate_structural_declaration",
            ),
            Self::AmbiguousStructuralDeclaration => (
                Conflict,
                Final,
                "bridge.build.ambiguous_structural_declaration",
            ),
            Self::StructuralComparisonModeMismatch => (
                InvalidInput,
                CorrectInput,
                "bridge.build.structural_comparison_mode_mismatch",
            ),
            Self::SourceCapabilityMismatch => (
                InvalidInput,
                CorrectInput,
                "bridge.build.source_capability_mismatch",
            ),
            Self::BuilderConfigurationConflict => (
                Conflict,
                Final,
                "bridge.build.builder_configuration_conflict",
            ),
            Self::InvalidFineGrainedWideningPolicy => (
                InvalidInput,
                CorrectInput,
                "bridge.build.invalid_fine_grained_widening_policy",
            ),
            Self::AmbiguousSemanticDependencyRegistration => (
                Conflict,
                Final,
                "bridge.build.ambiguous_semantic_dependency_registration",
            ),
            Self::MixedSemanticCorrespondenceSignalGraphs => (
                InvalidInput,
                CorrectInput,
                "bridge.build.mixed_semantic_correspondence_signal_graphs",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge configuration was rejected."
    }
}

impl BridgeErrorKind for BridgeRouteErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::EmptyBulkWorkloadRequest => (
                InvalidInput,
                CorrectInput,
                "bridge.route.empty_bulk_workload_request",
            ),
            Self::RoutePolicyMismatch => (
                InvalidInput,
                CorrectInput,
                "bridge.route.route_policy_mismatch",
            ),
            Self::UnsupportedProducerEnvelope => (
                InvalidInput,
                CorrectInput,
                "bridge.route.unsupported_producer_envelope",
            ),
            Self::MissingMappingRegistration => (
                NotFound,
                CorrectInput,
                "bridge.route.missing_mapping_registration",
            ),
            Self::AmbiguousSliceMappingRegistration => (
                Conflict,
                Final,
                "bridge.route.ambiguous_slice_mapping_registration",
            ),
            Self::UnsupportedTruthPatchScope => (
                InvalidInput,
                CorrectInput,
                "bridge.route.unsupported_truth_patch_scope",
            ),
            Self::UnsupportedTruthDeltaSurface => (
                InvalidInput,
                CorrectInput,
                "bridge.route.unsupported_truth_delta_surface",
            ),
            Self::UnsupportedSubscriptionSlice => (
                InvalidInput,
                CorrectInput,
                "bridge.route.unsupported_subscription_slice",
            ),
            Self::InconsistentNormalizedSurfaceDigest => (
                Invariant,
                Escalate,
                "bridge.route.inconsistent_normalized_surface_digest",
            ),
            Self::InvalidAuthoritativePatchSemantics => (
                Invariant,
                Escalate,
                "bridge.route.invalid_authoritative_patch_semantics",
            ),
            Self::UnsupportedAuthoritativePatchPrecision => (
                InvalidInput,
                CorrectInput,
                "bridge.route.unsupported_authoritative_patch_precision",
            ),
            Self::SliceReadPacketConstructionFailure => (
                Defect,
                Escalate,
                "bridge.route.slice_read_packet_construction_failure",
            ),
            Self::InvalidLoweringContract => (
                Invariant,
                Escalate,
                "bridge.route.invalid_lowering_contract",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not route the request."
    }
}
