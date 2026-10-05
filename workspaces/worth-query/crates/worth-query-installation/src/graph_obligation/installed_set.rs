use std::sync::Arc;

use worth_foundational::facade::CanonicalDigestDerivationDenial;
use worth_proof::NonEmpty;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::application_query::WorthQueryInstalledApplicationQueryIdentity;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

use super::identity::derive_set_identity;
use super::selection_index::WorthQueryInstalledGraphObligationSelectionIndex;
use super::{
    WorthQueryInstalledGraphObligation, WorthQueryInstalledGraphObligationContract,
    WorthQueryInstalledGraphObligationIdentity, WorthQueryInstalledGraphObligationKind,
    WorthQueryInstalledGraphObligationResourcePosture,
    WorthQueryInstalledGraphObligationSetIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphObligationSubjectKind {
    ApplicationQuery,
    ApplicationOperation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorthQueryInstalledGraphObligationSubject {
    ApplicationQuery {
        name: String,
        identity: WorthQueryInstalledApplicationQueryIdentity,
    },
    ApplicationOperation {
        name: String,
        input_type: String,
    },
}

impl WorthQueryInstalledGraphObligationSubject {
    fn identity_fields(&self) -> (&'static str, &str, Option<&str>) {
        match self {
            Self::ApplicationQuery { name, .. } => ("application-query", name, None),
            Self::ApplicationOperation { name, input_type } => {
                ("application-operation", name, Some(input_type))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphObligationInstallationEvidence {
    obligation_rows: usize,
    selector_index_entries: usize,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryInstalledGraphObligationInstallationEvidence {
    pub const fn obligation_rows(self) -> usize {
        self.obligation_rows
    }

    pub const fn selector_index_entries(self) -> usize {
        self.selector_index_entries
    }

    pub const fn canonical_work(self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryInstalledGraphObligationLookup<'a> {
    rows: &'a [WorthQueryInstalledGraphObligation],
    selector_index_probes: usize,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl<'a> WorthQueryInstalledGraphObligationLookup<'a> {
    pub const fn rows(self) -> &'a [WorthQueryInstalledGraphObligation] {
        self.rows
    }

    pub const fn selector_index_probes(self) -> usize {
        self.selector_index_probes
    }

    pub const fn canonical_work(self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryInstalledGraphObligationInspection<'a> {
    installed: &'a WorthQueryInstalledGraphObligationSet,
}

impl WorthQueryInstalledGraphObligationInspection<'_> {
    pub const fn identity(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        self.installed.identity()
    }

    pub const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        self.installed.binding_identity()
    }

    pub const fn subject_kind(&self) -> WorthQueryInstalledGraphObligationSubjectKind {
        self.installed.subject_kind()
    }

    pub fn subject_name(&self) -> &str {
        self.installed.subject_name()
    }

    pub fn rows(&self) -> &[WorthQueryInstalledGraphObligation] {
        self.installed.rows()
    }

    pub const fn installation_evidence(
        &self,
    ) -> WorthQueryInstalledGraphObligationInstallationEvidence {
        self.installed.installation_evidence()
    }
}

/// Sealed installed meaning for one exact query or operation.
///
/// ```compile_fail
/// use worth_query_installation::facade::WorthQueryInstalledGraphObligationSet;
///
/// let forged = WorthQueryInstalledGraphObligationSet {
///     identity: todo!(),
///     binding_identity: todo!(),
///     subject: todo!(),
///     rows: todo!(),
///     selection_index: todo!(),
///     installation: todo!(),
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphObligationSet {
    identity: WorthQueryInstalledGraphObligationSetIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    subject: WorthQueryInstalledGraphObligationSubject,
    rows: NonEmpty<WorthQueryInstalledGraphObligation>,
    selection_index: WorthQueryInstalledGraphObligationSelectionIndex,
    installation: WorthQueryInstalledGraphObligationInstallationEvidence,
}

impl WorthQueryInstalledGraphObligationSet {
    pub(super) fn for_query(
        binding_identity: &ApplicationSchemaBindingIdentity,
        name: String,
        query_identity: WorthQueryInstalledApplicationQueryIdentity,
        contracts: Vec<WorthQueryInstalledGraphObligationContract>,
        resources: WorthQueryInstalledGraphObligationResourcePosture,
    ) -> Result<Self, CanonicalDigestDerivationDenial> {
        Self::build(
            binding_identity,
            WorthQueryInstalledGraphObligationSubject::ApplicationQuery {
                name,
                identity: query_identity,
            },
            contracts,
            resources,
        )
    }

    pub(super) fn for_operation(
        binding_identity: &ApplicationSchemaBindingIdentity,
        name: String,
        input_type: String,
        contracts: Vec<WorthQueryInstalledGraphObligationContract>,
        resources: WorthQueryInstalledGraphObligationResourcePosture,
    ) -> Result<Self, CanonicalDigestDerivationDenial> {
        Self::build(
            binding_identity,
            WorthQueryInstalledGraphObligationSubject::ApplicationOperation { name, input_type },
            contracts,
            resources,
        )
    }

    fn build(
        binding_identity: &ApplicationSchemaBindingIdentity,
        subject: WorthQueryInstalledGraphObligationSubject,
        contracts: Vec<WorthQueryInstalledGraphObligationContract>,
        resources: WorthQueryInstalledGraphObligationResourcePosture,
    ) -> Result<Self, CanonicalDigestDerivationDenial> {
        let (subject_kind, subject_name, input_type) = subject.identity_fields();
        let (identity, canonical_work) = derive_set_identity(
            binding_identity,
            subject_kind,
            subject_name,
            input_type,
            &contracts,
            &resources,
        )?;
        let resources = Arc::new(resources);
        let rows = materialize_rows(&identity, contracts, &resources);
        let selection_index =
            WorthQueryInstalledGraphObligationSelectionIndex::build(rows.as_slice());
        Ok(Self {
            identity,
            binding_identity: binding_identity.clone(),
            subject,
            installation: WorthQueryInstalledGraphObligationInstallationEvidence {
                obligation_rows: rows.len(),
                selector_index_entries: WorthQueryInstalledGraphObligationKind::ALL.len(),
                canonical_work,
            },
            rows,
            selection_index,
        })
    }

    pub const fn identity(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.identity
    }

    pub const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub const fn subject_kind(&self) -> WorthQueryInstalledGraphObligationSubjectKind {
        match self.subject {
            WorthQueryInstalledGraphObligationSubject::ApplicationQuery { .. } => {
                WorthQueryInstalledGraphObligationSubjectKind::ApplicationQuery
            }
            WorthQueryInstalledGraphObligationSubject::ApplicationOperation { .. } => {
                WorthQueryInstalledGraphObligationSubjectKind::ApplicationOperation
            }
        }
    }

    pub fn subject_name(&self) -> &str {
        match &self.subject {
            WorthQueryInstalledGraphObligationSubject::ApplicationQuery { name, .. }
            | WorthQueryInstalledGraphObligationSubject::ApplicationOperation { name, .. } => name,
        }
    }

    pub fn rows(&self) -> &[WorthQueryInstalledGraphObligation] {
        self.rows.as_slice()
    }

    pub fn inspect_kind(
        &self,
        kind: WorthQueryInstalledGraphObligationKind,
    ) -> WorthQueryInstalledGraphObligationLookup<'_> {
        WorthQueryInstalledGraphObligationLookup {
            rows: self.selection_index.select(self.rows.as_slice(), kind),
            selector_index_probes: 1,
            canonical_work: WorthQueryCanonicalWorkEvidence::zero(),
        }
    }

    pub const fn installation_evidence(
        &self,
    ) -> WorthQueryInstalledGraphObligationInstallationEvidence {
        self.installation
    }

    pub const fn inspect(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        WorthQueryInstalledGraphObligationInspection { installed: self }
    }

    pub fn application_query_identity(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationQueryIdentity> {
        match &self.subject {
            WorthQueryInstalledGraphObligationSubject::ApplicationQuery { identity, .. } => {
                Some(identity)
            }
            WorthQueryInstalledGraphObligationSubject::ApplicationOperation { .. } => None,
        }
    }
}

fn materialize_rows(
    identity: &WorthQueryInstalledGraphObligationSetIdentity,
    contracts: Vec<WorthQueryInstalledGraphObligationContract>,
    resources: &Arc<WorthQueryInstalledGraphObligationResourcePosture>,
) -> NonEmpty<WorthQueryInstalledGraphObligation> {
    let rows = contracts
        .into_iter()
        .enumerate()
        .map(|(slot, contract)| {
            WorthQueryInstalledGraphObligation::new(
                WorthQueryInstalledGraphObligationIdentity::new(
                    *identity.digest(),
                    u32::try_from(slot).expect("canonical entry budget bounds obligation slots"),
                ),
                contract,
                Arc::clone(resources),
            )
        })
        .collect();
    NonEmpty::try_from_vec(rows)
        .expect("every installed query or operation has at least one graph obligation")
}
