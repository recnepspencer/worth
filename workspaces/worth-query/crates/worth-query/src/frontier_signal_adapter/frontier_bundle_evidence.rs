#[cfg(test)]
use crate::frontier_planning::{
    FrontierSurfaceDigest, SerialFallbackBundleEvidence, SerialFallbackBundleEvidenceError,
    SerialFallbackEvidence,
};
#[cfg(test)]
use worth_signal::facade::adapters::FrontierRouteEvidenceReceipt;

#[cfg(test)]
use super::frontier_admission_evidence::SignalAdmissionEvidenceError;
#[cfg(test)]
use super::frontier_surface_model::SignalFrontierSurfaceEvidence;

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SignalFrontierBundleEvidence {
    bundle_surface_digest: FrontierSurfaceDigest,
    route_evidences: Vec<SerialFallbackEvidence>,
}

#[cfg(test)]
impl SignalFrontierBundleEvidence {
    #[cfg(test)]
    pub(crate) fn from_route_evidences(route_evidences: Vec<SerialFallbackEvidence>) -> Self {
        let mut parts = vec![format!("route_count:{}", route_evidences.len())];
        for (index, route) in route_evidences.iter().enumerate() {
            parts.push(format!("route[{index}].basis:{}", route.basis_digest()));
            parts.push(format!(
                "route[{index}].surface:{}",
                route.surface_digest().as_str()
            ));
            parts.push(format!(
                "route[{index}].drift:{}",
                route.drift_outcome().as_str()
            ));
            parts.push(format!(
                "route[{index}].fallback:{}",
                route.reason().as_str()
            ));
        }

        Self {
            bundle_surface_digest: FrontierSurfaceDigest::from_label(&parts.join("|")),
            route_evidences,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_stage_records(
        basis_digest: &str,
        route_surfaces: &[SignalFrontierSurfaceEvidence],
        route_receipts: &[FrontierRouteEvidenceReceipt],
    ) -> Result<Self, SignalAdmissionEvidenceError> {
        if route_surfaces.len() != route_receipts.len() {
            return Err(SignalAdmissionEvidenceError::RouteCountMismatch {
                surfaces: route_surfaces.len(),
                route_receipts: route_receipts.len(),
            });
        }

        let mut route_evidences = Vec::with_capacity(route_receipts.len());
        for (surface, route_receipt) in route_surfaces.iter().zip(route_receipts.iter()) {
            route_evidences
                .push(surface.to_route_evidence_from_stage_record(basis_digest, route_receipt)?);
        }

        Ok(Self::from_route_evidences(route_evidences))
    }

    #[cfg(test)]
    pub(crate) fn bundle_surface_digest(&self) -> &FrontierSurfaceDigest {
        &self.bundle_surface_digest
    }

    #[cfg(test)]
    pub(crate) fn bind_to_basis(
        &self,
        basis_digest: &str,
    ) -> Result<SerialFallbackBundleEvidence, SerialFallbackBundleEvidenceError> {
        let rebound_routes = self
            .route_evidences
            .iter()
            .map(|route| {
                SerialFallbackEvidence::from_surface(
                    basis_digest.to_string(),
                    route.surface_digest().clone(),
                    route.reason().clone(),
                    route.drift_outcome().clone(),
                )
            })
            .collect();
        SerialFallbackBundleEvidence::from_routes(
            self.bundle_surface_digest.clone(),
            rebound_routes,
        )
    }
}
