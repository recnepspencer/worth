use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod resolved;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::identity::SchemaBasisDigest;
use crate::planning::{ExecutionPlanBundle, PlannedExecutionRoute};
use worth_runtime_bridge::facade::TruthSnapshotIdentity;

pub use resolved::{ResolvedBasisProof, ResolvedSnapshotBasis, SnapshotResolutionReport};
pub use worth_query_declaration::facade::schema_view::{
    QuerySchemaBasisAuthority, QuerySchemaView,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BasisAuthorityFamily {
    Runtime,
    Store,
}

impl BasisAuthorityFamily {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Store => "store",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SnapshotLineageClass {
    CurrentHead,
    ReplayEquivalent,
    FutureExtension,
}

impl SnapshotLineageClass {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::CurrentHead => "current_head",
            Self::ReplayEquivalent => "replay_equivalent",
            Self::FutureExtension => "future_extension",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BasisResolutionMode {
    RuntimeDirect,
    StoreDirect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionBasisIntent {
    authority_family: BasisAuthorityFamily,
    lineage_class: SnapshotLineageClass,
    fallback_allowed: bool,
}

impl ExecutionBasisIntent {
    pub fn new(
        authority_family: BasisAuthorityFamily,
        lineage_class: SnapshotLineageClass,
        fallback_allowed: bool,
    ) -> Self {
        Self {
            authority_family,
            lineage_class,
            fallback_allowed,
        }
    }

    pub fn authority_family(&self) -> &BasisAuthorityFamily {
        &self.authority_family
    }

    pub fn lineage_class(&self) -> &SnapshotLineageClass {
        &self.lineage_class
    }

    pub fn fallback_allowed(&self) -> bool {
        self.fallback_allowed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSnapshotIdentity {
    authority_family: BasisAuthorityFamily,
    workspace_scope: Option<String>,
    snapshot_identity: WorthQueryEvidenceIdentity,
    schema_basis: SchemaBasisDigest,
    lineage_class: SnapshotLineageClass,
}

impl ResolvedSnapshotIdentity {
    pub(crate) fn new(
        authority_family: BasisAuthorityFamily,
        workspace_scope: Option<String>,
        snapshot_identity: WorthQueryEvidenceIdentity,
        schema_basis: SchemaBasisDigest,
        lineage_class: SnapshotLineageClass,
    ) -> Self {
        Self {
            authority_family,
            workspace_scope,
            snapshot_identity,
            schema_basis,
            lineage_class,
        }
    }

    pub fn authority_family(&self) -> &BasisAuthorityFamily {
        &self.authority_family
    }

    pub fn workspace_scope(&self) -> Option<&str> {
        self.workspace_scope.as_deref()
    }

    pub fn snapshot_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_identity
    }

    pub fn schema_basis(&self) -> &SchemaBasisDigest {
        &self.schema_basis
    }

    pub fn lineage_class(&self) -> &SnapshotLineageClass {
        &self.lineage_class
    }

    fn evidence_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(
            worth_query_evidence_identity(WorthQueryEvidenceScope::ResolvedSnapshotBasis)
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_family"),
                    self.authority_family.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot_identity"),
                    &self.snapshot_identity,
                )
                .field_digest(
                    WorthQueryEvidenceTag::new("schema_basis"),
                    self.schema_basis().digest(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("lineage_class"),
                    self.lineage_class.as_str(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("workspace_scope"),
                    self.workspace_scope.as_deref(),
                )
                .seal()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BasisResolutionError {
    UnsupportedBasisKind,
    ResolutionIdentityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BasisPreflightError {
    BasisIntentMismatch,
    PlannedRouteBasisMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPreflightBundle {
    plan: ExecutionPlanBundle,
    basis: ResolvedSnapshotBasis,
    report: SnapshotResolutionReport,
}

impl ExecutionPreflightBundle {
    pub fn plan(&self) -> &ExecutionPlanBundle {
        &self.plan
    }

    pub fn basis(&self) -> &ResolvedSnapshotBasis {
        &self.basis
    }

    pub fn report(&self) -> &SnapshotResolutionReport {
        &self.report
    }
}

pub fn resolve_snapshot_basis(
    intent: ExecutionBasisIntent,
    identity: ResolvedSnapshotIdentity,
    resolution_mode: BasisResolutionMode,
) -> Result<ResolvedSnapshotBasis, BasisResolutionError> {
    if intent.authority_family() != identity.authority_family() {
        return Err(BasisResolutionError::ResolutionIdentityMismatch);
    }

    if intent.lineage_class() != identity.lineage_class() {
        return Err(BasisResolutionError::ResolutionIdentityMismatch);
    }

    let mode_matches_authority = matches!(
        (intent.authority_family(), &resolution_mode),
        (
            BasisAuthorityFamily::Runtime,
            BasisResolutionMode::RuntimeDirect
        ) | (
            BasisAuthorityFamily::Store,
            BasisResolutionMode::StoreDirect
        )
    );
    if !mode_matches_authority {
        return Err(BasisResolutionError::UnsupportedBasisKind);
    }

    Ok(ResolvedSnapshotBasis::new(
        intent,
        identity,
        resolution_mode,
    ))
}

pub(crate) fn resolve_runtime_current_snapshot_basis(
    snapshot_identity: WorthQueryEvidenceIdentity,
    schema_basis: QuerySchemaBasisAuthority,
) -> Result<ResolvedSnapshotBasis, BasisResolutionError> {
    resolve_snapshot_basis(
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Runtime,
            SnapshotLineageClass::CurrentHead,
            false,
        ),
        ResolvedSnapshotIdentity::new(
            BasisAuthorityFamily::Runtime,
            None,
            snapshot_identity,
            schema_basis.into_digest(),
            SnapshotLineageClass::CurrentHead,
        ),
        BasisResolutionMode::RuntimeDirect,
    )
}

pub(crate) fn resolve_runtime_historical_snapshot_basis(
    snapshot_identity: WorthQueryEvidenceIdentity,
    schema_basis: SchemaBasisDigest,
    workspace_scope: impl Into<String>,
) -> Result<ResolvedSnapshotBasis, IdentityDerivationDenial> {
    let reporting_label = snapshot_identity.as_str().to_string();
    let intent = ExecutionBasisIntent::new(
        BasisAuthorityFamily::Runtime,
        SnapshotLineageClass::ReplayEquivalent,
        false,
    );
    let identity = ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        Some(workspace_scope.into()),
        snapshot_identity,
        schema_basis,
        SnapshotLineageClass::ReplayEquivalent,
    );
    Ok(ResolvedSnapshotBasis::new_with_reporting_label(
        intent,
        identity,
        BasisResolutionMode::RuntimeDirect,
        reporting_label,
    )?)
}

pub fn admit_runtime_current_snapshot_basis(
    snapshot_identity: WorthQueryEvidenceIdentity,
    schema_view: QuerySchemaView,
) -> Result<ResolvedSnapshotBasis, BasisResolutionError> {
    resolve_runtime_current_snapshot_basis(snapshot_identity, schema_view.basis_authority())
}

pub fn snapshot_resolution_report(basis: &ResolvedSnapshotBasis) -> SnapshotResolutionReport {
    SnapshotResolutionReport::from_resolved_basis(basis)
}

pub(crate) fn bridge_snapshot_evidence_identity(
    snapshot_identity: &TruthSnapshotIdentity,
) -> Result<WorthQueryEvidenceIdentity, BasisResolutionError> {
    let Some(parts) = snapshot_identity.relational_snapshot_parts() else {
        return Err(BasisResolutionError::UnsupportedBasisKind);
    };
    Ok(
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(parts)
            .evidence_identity(),
    )
}

pub fn preflight_execution_basis(
    plan: ExecutionPlanBundle,
    basis: ResolvedSnapshotBasis,
) -> Result<ExecutionPreflightBundle, BasisPreflightError> {
    if plan.request_context().semantic().basis_intent() != basis.intent() {
        return Err(BasisPreflightError::BasisIntentMismatch);
    }

    let route_matches_basis = match basis.identity().authority_family() {
        BasisAuthorityFamily::Runtime => matches!(
            plan.query().route(),
            PlannedExecutionRoute::RuntimeSnapshotRead
                | PlannedExecutionRoute::RuntimeExpandedSnapshotRead
        ),
        BasisAuthorityFamily::Store => {
            plan.query().route() == &PlannedExecutionRoute::StoreSnapshotRead
        }
    };

    if !route_matches_basis {
        return Err(BasisPreflightError::PlannedRouteBasisMismatch);
    }

    Ok(ExecutionPreflightBundle {
        report: SnapshotResolutionReport::from_resolved_basis(&basis),
        plan,
        basis,
    })
}
