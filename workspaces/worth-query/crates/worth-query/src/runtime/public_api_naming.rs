use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicApiNamingRow {
    concept: String,
    preferred_name: String,
    alternate_names: Vec<String>,
    boundary_crossing: bool,
    naming_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicApiNamingRow {
    pub(crate) fn new(
        concept: impl Into<String>,
        preferred_name: impl Into<String>,
        alternate_names: impl IntoIterator<Item = impl Into<String>>,
        boundary_crossing: bool,
    ) -> Result<Self, IdentityDerivationDenial> {
        let concept = concept.into();
        let preferred_name = preferred_name.into();
        let alternate_names = alternate_names
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let naming_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicApiNamingRow)
                .field_shape(WorthQueryEvidenceTag::new("concept"), concept.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("preferred_name"),
                    preferred_name.as_str(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("boundary_crossing"),
                    boundary_crossing,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("alternate_name"),
                    alternate_names.iter().map(String::as_str),
                )
                .seal()?;
        Ok(Self {
            concept,
            preferred_name,
            alternate_names,
            boundary_crossing,
            naming_identity,
        })
    }

    pub fn concept(&self) -> &str {
        &self.concept
    }

    pub fn preferred_name(&self) -> &str {
        &self.preferred_name
    }

    pub fn alternate_names(&self) -> &[String] {
        &self.alternate_names
    }

    pub fn boundary_crossing(&self) -> bool {
        self.boundary_crossing
    }

    pub fn naming_digest(&self) -> &str {
        self.naming_identity.as_str()
    }

    pub fn naming_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.naming_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicApiNamingContract {
    rows: Vec<WorthQueryRuntimePublicApiNamingRow>,
    preferred_entrypoint_count: usize,
    alternate_name_count: usize,
    boundary_crossing_name_count: usize,
    contract_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicApiNamingContract {
    pub fn standard() -> Result<Self, IdentityDerivationDenial> {
        let rows = vec![
            WorthQueryRuntimePublicApiNamingRow::new(
                "workspace",
                "workspace",
                std::iter::empty::<&str>(),
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "live-view",
                "live_view",
                ["live_view_request", "declare_live_view"],
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "live-view-builder",
                "live_view closure",
                std::iter::empty::<&str>(),
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "computed",
                "computed",
                [
                    "computed_view",
                    "computed_definition",
                    "declare_maintained_derived_view",
                    "declare_derived_view",
                ],
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "computed-builder",
                "computed closure",
                std::iter::empty::<&str>(),
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new("effect", "effect", ["declare_effect"], true)?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "preview",
                "preview",
                ["preview_with_options"],
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "branch",
                "branch",
                ["branch_with_options"],
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "state",
                "state",
                std::iter::empty::<&str>(),
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "inspect",
                "inspect",
                std::iter::empty::<&str>(),
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "read",
                "read",
                std::iter::empty::<&str>(),
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "observe",
                "observe",
                ["drain_patches"],
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "materialize",
                "materialize",
                ["snapshot_rows"],
                true,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "write",
                "write",
                ["WorthQueryWriteCommand"],
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "insert",
                "insert",
                ["WorthQueryWriteCommand::InsertAspects"],
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "update",
                "update",
                [
                    "WorthQueryWriteCommand::UpdateAspect",
                    "WorthQueryWriteCommand::UpdateAspects",
                ],
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "delete",
                "delete",
                ["WorthQueryWriteCommand::Delete"],
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "batch",
                "batch",
                ["workspace.write(...)"],
                false,
            )?,
            WorthQueryRuntimePublicApiNamingRow::new(
                "replay",
                "replay_journal_segment",
                ["WorthQueryJournalReplayRequest"],
                true,
            )?,
        ];
        let preferred_entrypoint_count = rows.len();
        let alternate_name_count = rows.iter().map(|row| row.alternate_names().len()).sum();
        let boundary_crossing_name_count =
            rows.iter().filter(|row| row.boundary_crossing()).count();
        let contract_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicApiNamingContract)
                .field_usize(
                    WorthQueryEvidenceTag::new("preferred_entrypoint_count"),
                    preferred_entrypoint_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("alternate_name_count"),
                    alternate_name_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("boundary_crossing_name_count"),
                    boundary_crossing_name_count,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("row_digest"),
                    rows.iter()
                        .map(WorthQueryRuntimePublicApiNamingRow::naming_digest),
                )
                .seal()?;
        Ok(Self {
            rows,
            preferred_entrypoint_count,
            alternate_name_count,
            boundary_crossing_name_count,
            contract_identity,
        })
    }

    pub fn rows(&self) -> &[WorthQueryRuntimePublicApiNamingRow] {
        &self.rows
    }

    pub fn preferred_name_for(&self, concept: &str) -> Option<&str> {
        self.rows
            .iter()
            .find(|row| row.concept() == concept)
            .map(WorthQueryRuntimePublicApiNamingRow::preferred_name)
    }

    pub fn preferred_entrypoint_count(&self) -> usize {
        self.preferred_entrypoint_count
    }

    pub fn alternate_name_count(&self) -> usize {
        self.alternate_name_count
    }

    pub fn boundary_crossing_name_count(&self) -> usize {
        self.boundary_crossing_name_count
    }

    pub fn contract_digest(&self) -> &str {
        self.contract_identity.as_str()
    }

    pub fn contract_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.contract_identity
    }
}
