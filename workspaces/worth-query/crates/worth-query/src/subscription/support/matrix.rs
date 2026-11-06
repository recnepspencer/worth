use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::evidence_identities::{support_matrix_identity, support_matrix_row_identity};
use super::super::evidence_projection::subscription_evidence_projection;
use super::super::family::QuerySubscriptionFamily;
use super::profile::{
    QuerySubscriptionActiveLifecycleSupport, QuerySubscriptionLifecycleCloseoutSupport,
    QuerySubscriptionRuntimeBackedSupport, QuerySubscriptionSupportProfile,
};
use super::subject::{
    QuerySubscriptionSupportClass, QuerySubscriptionSupportPosture,
    QuerySubscriptionSupportSubject, SubscriptionFamilyCapabilityDigest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSupportMatrixRow {
    support_class: QuerySubscriptionSupportClass,
    posture: QuerySubscriptionSupportPosture,
    row_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionSupportMatrixRow {
    pub(crate) fn new(
        family: &QuerySubscriptionFamily,
        support_class: QuerySubscriptionSupportClass,
        posture: QuerySubscriptionSupportPosture,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_identity =
            support_matrix_row_identity(family, support_class.as_str(), posture.as_str())?;
        Ok(Self {
            support_class,
            posture,
            row_identity,
        })
    }

    pub fn support_class(&self) -> &QuerySubscriptionSupportClass {
        &self.support_class
    }

    pub fn posture(&self) -> &QuerySubscriptionSupportPosture {
        &self.posture
    }

    pub fn row_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_identity
    }

    pub fn row_projection(&self) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.row_identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSupportMatrix {
    family: QuerySubscriptionFamily,
    capability_digest: SubscriptionFamilyCapabilityDigest,
    rows: Vec<QuerySubscriptionSupportMatrixRow>,
    matrix_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionSupportMatrix {
    pub(crate) fn for_family(
        family: &QuerySubscriptionFamily,
        profile: Option<&QuerySubscriptionSupportProfile>,
        subject: &QuerySubscriptionSupportSubject,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_digest = SubscriptionFamilyCapabilityDigest::for_family(family);
        let coverage = SupportCoverageStage::for_subject(subject);
        let rows = QuerySubscriptionSupportClass::all()
            .into_iter()
            .map(|support_class| {
                QuerySubscriptionSupportMatrixRow::new(
                    family,
                    support_class,
                    posture_for_class(support_class, coverage, profile),
                )
            })
            .collect::<Vec<_>>();
        let row_refs: Vec<&WorthQueryEvidenceIdentity> =
            rows.iter().map(|row| row.row_identity()).collect();
        let matrix_identity =
            support_matrix_identity(family, capability_digest.capability_identity(), row_refs)?;
        Ok(Self {
            family: family.clone(),
            capability_digest,
            rows,
            matrix_identity,
        })
    }

    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub(crate) fn capability_digest(&self) -> &SubscriptionFamilyCapabilityDigest {
        &self.capability_digest
    }

    pub fn rows(&self) -> &[QuerySubscriptionSupportMatrixRow] {
        &self.rows
    }

    pub fn row_for_class(
        &self,
        support_class: QuerySubscriptionSupportClass,
    ) -> Option<&QuerySubscriptionSupportMatrixRow> {
        self.rows
            .iter()
            .find(|row| row.support_class() == &support_class)
    }

    pub fn matrix_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.matrix_identity
    }

    pub fn matrix_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.matrix_identity)
    }
}

fn posture_for_class(
    support_class: QuerySubscriptionSupportClass,
    coverage: SupportCoverageStage,
    profile: Option<&QuerySubscriptionSupportProfile>,
) -> QuerySubscriptionSupportPosture {
    match support_class {
        QuerySubscriptionSupportClass::Declaration => {
            QuerySubscriptionSupportPosture::RuntimeBackedCertified
        }
        QuerySubscriptionSupportClass::Activation => stage_bounded_posture(
            coverage.allows(QuerySubscriptionSupportClass::Activation),
            profile
                .map(|profile| runtime_backed_posture(profile.runtime_backed_support()))
                .unwrap_or(QuerySubscriptionSupportPosture::UncertifiedDenied),
        ),
        QuerySubscriptionSupportClass::ActiveLifecycle
        | QuerySubscriptionSupportClass::Continuation => stage_bounded_posture(
            coverage.allows(support_class),
            profile
                .map(|profile| lifecycle_posture(profile.active_lifecycle_support()))
                .unwrap_or(QuerySubscriptionSupportPosture::UncertifiedDenied),
        ),
        QuerySubscriptionSupportClass::PreviewCloseout => stage_bounded_posture(
            coverage.allows(QuerySubscriptionSupportClass::PreviewCloseout),
            profile
                .map(|profile| closeout_posture(profile.lifecycle_closeout_support()))
                .unwrap_or(QuerySubscriptionSupportPosture::UncertifiedDenied),
        ),
        QuerySubscriptionSupportClass::DurableReplay
        | QuerySubscriptionSupportClass::StoreBackedRestart => {
            QuerySubscriptionSupportPosture::RuntimeBackedDeferred
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SupportCoverageStage {
    DeclarationOnly,
    Activation,
    ActiveLifecycle,
    Continuation,
    PreviewCloseout,
}

impl SupportCoverageStage {
    fn for_subject(subject: &QuerySubscriptionSupportSubject) -> Self {
        match subject.support_class() {
            QuerySubscriptionSupportClass::Declaration
            | QuerySubscriptionSupportClass::DurableReplay
            | QuerySubscriptionSupportClass::StoreBackedRestart => Self::DeclarationOnly,
            QuerySubscriptionSupportClass::Activation => Self::Activation,
            QuerySubscriptionSupportClass::ActiveLifecycle => Self::ActiveLifecycle,
            QuerySubscriptionSupportClass::Continuation => Self::Continuation,
            QuerySubscriptionSupportClass::PreviewCloseout => Self::PreviewCloseout,
        }
    }

    fn allows(&self, support_class: QuerySubscriptionSupportClass) -> bool {
        match self {
            Self::DeclarationOnly => false,
            Self::Activation => matches!(support_class, QuerySubscriptionSupportClass::Activation),
            Self::ActiveLifecycle => matches!(
                support_class,
                QuerySubscriptionSupportClass::Activation
                    | QuerySubscriptionSupportClass::ActiveLifecycle
            ),
            Self::Continuation => matches!(
                support_class,
                QuerySubscriptionSupportClass::Activation
                    | QuerySubscriptionSupportClass::ActiveLifecycle
                    | QuerySubscriptionSupportClass::Continuation
            ),
            Self::PreviewCloseout => matches!(
                support_class,
                QuerySubscriptionSupportClass::Activation
                    | QuerySubscriptionSupportClass::ActiveLifecycle
                    | QuerySubscriptionSupportClass::Continuation
                    | QuerySubscriptionSupportClass::PreviewCloseout
            ),
        }
    }
}

fn stage_bounded_posture(
    phase_proven: bool,
    posture: QuerySubscriptionSupportPosture,
) -> QuerySubscriptionSupportPosture {
    if phase_proven {
        posture
    } else {
        QuerySubscriptionSupportPosture::UncertifiedDenied
    }
}

fn runtime_backed_posture(
    support: &QuerySubscriptionRuntimeBackedSupport,
) -> QuerySubscriptionSupportPosture {
    match support {
        QuerySubscriptionRuntimeBackedSupport::Admitted => {
            QuerySubscriptionSupportPosture::RuntimeBackedCertified
        }
        QuerySubscriptionRuntimeBackedSupport::Denied => {
            QuerySubscriptionSupportPosture::RuntimeBackedDenied
        }
    }
}

fn lifecycle_posture(
    support: &QuerySubscriptionActiveLifecycleSupport,
) -> QuerySubscriptionSupportPosture {
    match support {
        QuerySubscriptionActiveLifecycleSupport::Admitted => {
            QuerySubscriptionSupportPosture::RuntimeBackedCertified
        }
        QuerySubscriptionActiveLifecycleSupport::Denied => {
            QuerySubscriptionSupportPosture::RuntimeBackedDenied
        }
    }
}

fn closeout_posture(
    support: &QuerySubscriptionLifecycleCloseoutSupport,
) -> QuerySubscriptionSupportPosture {
    match support {
        QuerySubscriptionLifecycleCloseoutSupport::Admitted => {
            QuerySubscriptionSupportPosture::RuntimeBackedCertified
        }
        QuerySubscriptionLifecycleCloseoutSupport::Denied => {
            QuerySubscriptionSupportPosture::RuntimeBackedDenied
        }
    }
}
