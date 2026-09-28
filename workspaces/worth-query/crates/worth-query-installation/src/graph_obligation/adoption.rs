use super::{
    WorthQueryInstalledGraphObligationInspection, WorthQueryInstalledGraphObligationKind,
    WorthQueryInstalledGraphObligationOwner, WorthQueryInstalledGraphObligationSubjectKind,
    WorthQueryInstalledGraphObligationTerminalRequirement,
};

/// Why `inspect_installed_graph_obligations` refused to produce adoption
/// evidence. Nothing is registered either way; inspection is read-only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphObligationAdoptionDenialKind {
    /// The consumer name was empty or only whitespace.
    BlankConsumerName,
    /// The inspected obligation rows no longer match the row count recorded in
    /// the installation evidence.
    InstallationEvidenceDrift,
}

/// A typed refusal from `inspect_installed_graph_obligations`: the denial kind
/// to match on and a human-readable message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphObligationAdoptionDenial {
    kind: WorthQueryGraphObligationAdoptionDenialKind,
    message: String,
}

impl WorthQueryGraphObligationAdoptionDenial {
    pub const fn kind(&self) -> WorthQueryGraphObligationAdoptionDenialKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// One installed graph obligation as seen by an adopting consumer: its slot,
/// kind, required owners, and terminal requirement.
///
/// Read from `WorthQueryGraphObligationAdoptionProof::rows`; it is descriptive
/// and grants nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphObligationAdoptionRow {
    slot: u32,
    kind: WorthQueryInstalledGraphObligationKind,
    required_owners: Vec<WorthQueryInstalledGraphObligationOwner>,
    terminal_requirement: WorthQueryInstalledGraphObligationTerminalRequirement,
}

impl WorthQueryGraphObligationAdoptionRow {
    pub const fn slot(&self) -> u32 {
        self.slot
    }

    pub const fn kind(&self) -> WorthQueryInstalledGraphObligationKind {
        self.kind
    }

    pub fn required_owners(&self) -> &[WorthQueryInstalledGraphObligationOwner] {
        &self.required_owners
    }

    pub const fn terminal_requirement(
        &self,
    ) -> WorthQueryInstalledGraphObligationTerminalRequirement {
        self.terminal_requirement
    }
}

/// Read-only evidence that a consumer adopted one installed obligation set.
///
/// The proof deliberately exposes no registration, selection, planning, or
/// execution transition. For example, this does not compile:
///
/// ```compile_fail
/// # use worth_query_installation::facade::WorthQueryGraphObligationAdoptionProof;
/// # fn misuse(proof: WorthQueryGraphObligationAdoptionProof) {
/// let _terminal = proof.execute();
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphObligationAdoptionProof {
    consumer_name: String,
    installed_set_identity: [u8; 32],
    subject_kind: WorthQueryInstalledGraphObligationSubjectKind,
    subject_name: String,
    rows: Vec<WorthQueryGraphObligationAdoptionRow>,
    selector_index_entries: usize,
}

impl WorthQueryGraphObligationAdoptionProof {
    pub fn consumer_name(&self) -> &str {
        &self.consumer_name
    }

    pub const fn installed_set_identity(&self) -> &[u8; 32] {
        &self.installed_set_identity
    }

    pub const fn subject_kind(&self) -> WorthQueryInstalledGraphObligationSubjectKind {
        self.subject_kind
    }

    pub fn subject_name(&self) -> &str {
        &self.subject_name
    }

    pub fn rows(&self) -> &[WorthQueryGraphObligationAdoptionRow] {
        &self.rows
    }

    pub const fn selector_index_entries(&self) -> usize {
        self.selector_index_entries
    }
}

/// Produces read-only evidence that a named consumer adopted the installed
/// graph obligations of one operation or query.
///
/// Pass the inspection from the installed contract's `graph_obligations()`.
/// Refuses a blank consumer name, and refuses when the inspected rows drifted
/// from the installation evidence. The returned proof carries no registration,
/// planning, or execution authority.
pub fn inspect_installed_graph_obligations(
    consumer_name: impl Into<String>,
    inspection: WorthQueryInstalledGraphObligationInspection<'_>,
) -> Result<WorthQueryGraphObligationAdoptionProof, WorthQueryGraphObligationAdoptionDenial> {
    let consumer_name = consumer_name.into();
    let consumer_name = consumer_name.trim();
    if consumer_name.is_empty() {
        return Err(denial(
            WorthQueryGraphObligationAdoptionDenialKind::BlankConsumerName,
            "graph-obligation adoption requires a non-empty consumer name",
        ));
    }

    let installation = inspection.installation_evidence();
    if installation.obligation_rows() != inspection.rows().len() {
        return Err(denial(
            WorthQueryGraphObligationAdoptionDenialKind::InstallationEvidenceDrift,
            "installed graph-obligation row count drifted from its installation evidence",
        ));
    }

    let rows = inspection
        .rows()
        .iter()
        .map(|row| WorthQueryGraphObligationAdoptionRow {
            slot: row.identity().slot(),
            kind: row.kind(),
            required_owners: row.required_owners().to_vec(),
            terminal_requirement: row.terminal_requirement(),
        })
        .collect();
    Ok(WorthQueryGraphObligationAdoptionProof {
        consumer_name: consumer_name.to_owned(),
        installed_set_identity: *inspection.identity().bytes(),
        subject_kind: inspection.subject_kind(),
        subject_name: inspection.subject_name().to_owned(),
        rows,
        selector_index_entries: installation.selector_index_entries(),
    })
}

fn denial(
    kind: WorthQueryGraphObligationAdoptionDenialKind,
    message: &'static str,
) -> WorthQueryGraphObligationAdoptionDenial {
    WorthQueryGraphObligationAdoptionDenial {
        kind,
        message: message.to_owned(),
    }
}
