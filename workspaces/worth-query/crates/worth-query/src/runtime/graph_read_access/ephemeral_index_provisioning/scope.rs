use crate::identity::hash_parts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryEphemeralGraphIndexScopeKind {
    ReadExecution,
    FamilyExecution,
}

impl WorthQueryEphemeralGraphIndexScopeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadExecution => "read_execution",
            Self::FamilyExecution => "family_execution",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEphemeralGraphIndexScope {
    digest: String,
    kind: WorthQueryEphemeralGraphIndexScopeKind,
    admitted_plan_digest: String,
    snapshot_identity_digest: String,
    byte_budget: Option<usize>,
}

impl WorthQueryEphemeralGraphIndexScope {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn kind(&self) -> &WorthQueryEphemeralGraphIndexScopeKind {
        &self.kind
    }

    pub fn admitted_plan_digest(&self) -> &str {
        &self.admitted_plan_digest
    }

    pub fn snapshot_identity_digest(&self) -> &str {
        &self.snapshot_identity_digest
    }

    pub fn byte_budget(&self) -> Option<usize> {
        self.byte_budget
    }

    pub(in crate::runtime::graph_read_access::ephemeral_index_provisioning) fn read_execution(
        admitted_plan_digest: impl Into<String>,
        snapshot_identity_digest: impl Into<String>,
        byte_budget: Option<usize>,
    ) -> Self {
        Self::new(
            WorthQueryEphemeralGraphIndexScopeKind::ReadExecution,
            admitted_plan_digest.into(),
            snapshot_identity_digest.into(),
            byte_budget,
        )
    }

    fn new(
        kind: WorthQueryEphemeralGraphIndexScopeKind,
        admitted_plan_digest: String,
        snapshot_identity_digest: String,
        byte_budget: Option<usize>,
    ) -> Self {
        let digest = hash_parts(&[
            "worth_query_ephemeral_graph_index_scope_v2".to_string(),
            format!("kind:{}", kind.as_str()),
            format!("admitted_plan:{admitted_plan_digest}"),
            format!("snapshot:{snapshot_identity_digest}"),
            format!("byte_budget:{byte_budget:?}"),
        ]);
        Self {
            digest,
            kind,
            admitted_plan_digest,
            snapshot_identity_digest,
            byte_budget,
        }
    }
}
