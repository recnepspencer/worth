use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiQueryIdentityReportingProjection(Arc<str>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiQueryObservationReportingProjection {
    query_world: UiQueryIdentityReportingProjection,
    binding: UiQueryIdentityReportingProjection,
    source_generation: UiQueryIdentityReportingProjection,
    result_generation: UiQueryIdentityReportingProjection,
}

impl UiQueryIdentityReportingProjection {
    pub(crate) fn from_query_reporting_text(projection: &str) -> Self {
        Self(Arc::from(projection))
    }

    pub fn from_query_identity(
        identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    ) -> Self {
        Self(Arc::from(identity.terminal_projection_for_reporting()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_arc(self) -> Arc<str> {
        self.0
    }
}

impl UiQueryObservationReportingProjection {
    pub fn from_observation(observation: &crate::UiProjectionObservation) -> Self {
        let core = match observation {
            crate::UiProjectionObservation::Scalar(observation) => observation.fact().core(),
            crate::UiProjectionObservation::ApplicationScalar(observation) => {
                let query = observation.fact().query_receipt().inspect();
                let basis = query.basis();
                return Self {
                    query_world: UiQueryIdentityReportingProjection::from_query_reporting_text(
                        &format!(
                            "application:{}:{}",
                            basis.runtime_instance(),
                            basis.branch()
                        ),
                    ),
                    binding: UiQueryIdentityReportingProjection::from_query_reporting_text(
                        query.parameter_binding_identity(),
                    ),
                    source_generation:
                        UiQueryIdentityReportingProjection::from_query_reporting_text(&format!(
                            "snapshot:{}",
                            basis.snapshot()
                        )),
                    result_generation:
                        UiQueryIdentityReportingProjection::from_query_reporting_text(&format!(
                            "version:{}",
                            basis.version()
                        )),
                };
            }
            crate::UiProjectionObservation::Collection(observation) => observation.fact().core(),
        };
        Self {
            query_world: UiQueryIdentityReportingProjection::from_query_identity(
                core.query_world_identity(),
            ),
            binding: UiQueryIdentityReportingProjection::from_query_identity(
                core.binding_identity(),
            ),
            source_generation: UiQueryIdentityReportingProjection::from_query_identity(
                core.source_generation_identity(),
            ),
            result_generation: UiQueryIdentityReportingProjection::from_query_identity(
                core.result_generation_identity(),
            ),
        }
    }

    pub fn query_world(&self) -> &str {
        self.query_world.as_str()
    }

    pub fn binding(&self) -> &str {
        self.binding.as_str()
    }

    pub fn source_generation(&self) -> &str {
        self.source_generation.as_str()
    }

    pub fn result_generation(&self) -> &str {
        self.result_generation.as_str()
    }
}

impl crate::UiCollectionProjectionRowReference {
    pub fn reporting_projection(&self) -> UiQueryIdentityReportingProjection {
        UiQueryIdentityReportingProjection::from_query_identity(self.query_identity())
    }
}

impl crate::UiProjectionOptionReference {
    pub fn reporting_projection(&self) -> UiQueryIdentityReportingProjection {
        UiQueryIdentityReportingProjection::from_query_identity(self.query_identity())
    }
}

#[cfg(any(test, feature = "certification-construction"))]
pub(crate) fn scalar_contract_digest_for_reporting(
    settled: &crate::application_binding::WorthUiSettledScalarTextProjection,
) -> String {
    settled
        .certification_settled_projection()
        .authority()
        .contract()
        .contract_digest()
        .to_owned()
}
