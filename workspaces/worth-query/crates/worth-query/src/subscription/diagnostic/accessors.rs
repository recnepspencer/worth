use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::evidence_projection::subscription_evidence_projection;
use super::bundle::{
    DiagnosticAssemblyReceipt, QuerySubscriptionAdmittedDiagnosticBundle,
    QuerySubscriptionDeniedDiagnosticBundle, QuerySubscriptionDiagnosticBundleError,
    QuerySubscriptionDiagnosticBundleWidth, QuerySubscriptionDiagnosticCounters,
    QuerySubscriptionDiagnosticFailure, QuerySubscriptionDiagnosticSemanticLabels,
};
use super::context::QuerySubscriptionDiagnosticSelectionContext;
use super::stage::QuerySubscriptionDiagnosticEvidence;
use super::trace::{QuerySubscriptionDiagnosticStageTrace, QuerySubscriptionDiagnosticTrace};

impl QuerySubscriptionDiagnosticCounters {
    pub fn counter_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(&self.evidence_identity()?))
    }
}

impl QuerySubscriptionDiagnosticBundleWidth {
    pub fn bundle_width_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.bundle_width_identity())
    }
}

impl DiagnosticAssemblyReceipt {
    pub fn assembly_receipt_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.assembly_receipt_identity())
    }
}

impl QuerySubscriptionDiagnosticSemanticLabels {
    pub fn labels_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.labels_identity())
    }
}

impl QuerySubscriptionDiagnosticFailure {
    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }

    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.counter_identity())
    }

    pub fn failure_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.failure_identity())
    }
}

impl QuerySubscriptionDiagnosticEvidence {
    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }

    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.counter_identity())
    }

    pub fn evidence_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl QuerySubscriptionDiagnosticStageTrace {
    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }

    pub fn evidence_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }

    pub fn stage_trace_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.stage_trace_identity())
    }
}

impl QuerySubscriptionDiagnosticTrace {
    pub fn trace_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.trace_identity())
    }
}

impl QuerySubscriptionDiagnosticBundleError {
    pub fn failure_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(self.failure_identity()?))
    }
}

impl QuerySubscriptionDiagnosticSelectionContext {
    pub fn context_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.context_identity())
    }
}

impl QuerySubscriptionAdmittedDiagnosticBundle {
    pub fn support_report_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.support_report_identity)
    }

    pub fn lifecycle_certification_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.lifecycle_certification_identity)
    }

    pub fn bundle_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.bundle_identity)
    }

    pub fn continuation_projection(
        &self,
    ) -> Option<QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>> {
        self.continuation_identity
            .as_ref()
            .map(subscription_evidence_projection)
    }

    pub fn preview_isolation_projection(
        &self,
    ) -> Option<QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>> {
        self.preview_isolation_identity
            .as_ref()
            .map(subscription_evidence_projection)
    }

    pub fn lifecycle_closeout_projection(
        &self,
    ) -> Option<QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>> {
        self.lifecycle_closeout_identity
            .as_ref()
            .map(subscription_evidence_projection)
    }
}

impl QuerySubscriptionDeniedDiagnosticBundle {
    pub fn support_report_projection(
        &self,
    ) -> Option<QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>> {
        self.support_report_identity
            .as_ref()
            .map(subscription_evidence_projection)
    }

    pub fn bundle_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.bundle_identity)
    }
}
