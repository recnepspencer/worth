use super::support::{
    WorthQueryDeclarationCapabilityStatus, WorthQueryDeclarationFamilySupportReport,
};
use super::support_checked::{
    worth_query_checked_family_support, WorthQueryDeclarationFamilySupportChecked,
};
use crate::application::{
    worth_query_canonical_declaration, WorthQueryAsyncDeclarationSupport,
    WorthQueryCanonicalDeclarationArtifact, WorthQueryDeclarationCanonicalizationError,
    WorthQueryDeclarationCanonicalizationVersion, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext, WorthQueryTemporalDeclarationSupport,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationCapabilityDenial<
    D: WorthQueryDomainEntryMarker,
    F: WorthQueryDeclarationFamilyMarker<D>,
> {
    support_report: WorthQueryDeclarationFamilySupportReport<D, F>,
}

impl<D: WorthQueryDomainEntryMarker, F: WorthQueryDeclarationFamilyMarker<D>>
    WorthQueryDeclarationCapabilityDenial<D, F>
{
    fn new(support_report: WorthQueryDeclarationFamilySupportReport<D, F>) -> Self {
        Self { support_report }
    }

    pub fn capability_status(&self) -> WorthQueryDeclarationCapabilityStatus {
        self.support_report.declare_status()
    }

    pub fn support_report(&self) -> &WorthQueryDeclarationFamilySupportReport<D, F> {
        &self.support_report
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAsyncDeclarationDenial<
    D: WorthQueryDomainEntryMarker,
    F: WorthQueryDeclarationFamilyMarker<D>,
> {
    support_report: WorthQueryDeclarationFamilySupportReport<D, F>,
    async_support: WorthQueryAsyncDeclarationSupport,
}

impl<D: WorthQueryDomainEntryMarker, F: WorthQueryDeclarationFamilyMarker<D>>
    WorthQueryAsyncDeclarationDenial<D, F>
{
    fn new(
        support_report: WorthQueryDeclarationFamilySupportReport<D, F>,
        async_support: WorthQueryAsyncDeclarationSupport,
    ) -> Self {
        Self {
            support_report,
            async_support,
        }
    }

    pub fn support_report(&self) -> &WorthQueryDeclarationFamilySupportReport<D, F> {
        &self.support_report
    }

    pub fn async_support(&self) -> WorthQueryAsyncDeclarationSupport {
        self.async_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalDeclarationDenial<
    D: WorthQueryDomainEntryMarker,
    F: WorthQueryDeclarationFamilyMarker<D>,
> {
    support_report: WorthQueryDeclarationFamilySupportReport<D, F>,
    temporal_support: WorthQueryTemporalDeclarationSupport,
}

impl<D: WorthQueryDomainEntryMarker, F: WorthQueryDeclarationFamilyMarker<D>>
    WorthQueryTemporalDeclarationDenial<D, F>
{
    fn new(
        support_report: WorthQueryDeclarationFamilySupportReport<D, F>,
        temporal_support: WorthQueryTemporalDeclarationSupport,
    ) -> Self {
        Self {
            support_report,
            temporal_support,
        }
    }

    pub fn support_report(&self) -> &WorthQueryDeclarationFamilySupportReport<D, F> {
        &self.support_report
    }

    pub fn temporal_support(&self) -> WorthQueryTemporalDeclarationSupport {
        self.temporal_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationAdmissionError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationCapabilityDenial<D, I::Family>),
    Unsupported(WorthQueryDeclarationCapabilityDenial<D, I::Family>),
    InvalidContext(WorthQueryDeclarationCapabilityDenial<D, I::Family>),
    AsyncDeferred(WorthQueryAsyncDeclarationDenial<D, I::Family>),
    AsyncUnsupported(WorthQueryAsyncDeclarationDenial<D, I::Family>),
    TemporalDeferred(WorthQueryTemporalDeclarationDenial<D, I::Family>),
    TemporalUnsupported(WorthQueryTemporalDeclarationDenial<D, I::Family>),
    Canonicalization(WorthQueryDeclarationCanonicalizationError),
}

pub enum WorthQueryDeclaredFamilyChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Admitted(WorthQueryCanonicalDeclarationArtifact<D, I>),
    Deferred(WorthQueryDeclarationCapabilityDenial<D, I::Family>),
    Unsupported(WorthQueryDeclarationCapabilityDenial<D, I::Family>),
    InvalidContext(WorthQueryDeclarationCapabilityDenial<D, I::Family>),
    AsyncDeferred(WorthQueryAsyncDeclarationDenial<D, I::Family>),
    AsyncUnsupported(WorthQueryAsyncDeclarationDenial<D, I::Family>),
    TemporalDeferred(WorthQueryTemporalDeclarationDenial<D, I::Family>),
    TemporalUnsupported(WorthQueryTemporalDeclarationDenial<D, I::Family>),
    Canonicalization(WorthQueryDeclarationCanonicalizationError),
}

pub(crate) fn worth_query_checked_family_declaration<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    input: I,
    version: WorthQueryDeclarationCanonicalizationVersion,
) -> WorthQueryDeclaredFamilyChecked<D, I> {
    let support_report = match worth_query_checked_family_support::<D, C, I::Family>(handle) {
        WorthQueryDeclarationFamilySupportChecked::Admitted(report) => report,
        WorthQueryDeclarationFamilySupportChecked::Deferred(report) => {
            return WorthQueryDeclaredFamilyChecked::Deferred(
                WorthQueryDeclarationCapabilityDenial::new(report),
            );
        }
        WorthQueryDeclarationFamilySupportChecked::Unsupported(report) => {
            return WorthQueryDeclaredFamilyChecked::Unsupported(
                WorthQueryDeclarationCapabilityDenial::new(report),
            );
        }
        WorthQueryDeclarationFamilySupportChecked::InvalidContext(report) => {
            return WorthQueryDeclaredFamilyChecked::InvalidContext(
                WorthQueryDeclarationCapabilityDenial::new(report),
            );
        }
    };

    if !input.async_resource_declaration_clauses().is_empty() {
        match I::Family::async_declaration_support() {
            WorthQueryAsyncDeclarationSupport::CanonicalIdentityOnly => {}
            WorthQueryAsyncDeclarationSupport::DeferredDebt => {
                return WorthQueryDeclaredFamilyChecked::AsyncDeferred(
                    WorthQueryAsyncDeclarationDenial::new(
                        support_report,
                        WorthQueryAsyncDeclarationSupport::DeferredDebt,
                    ),
                );
            }
            WorthQueryAsyncDeclarationSupport::Unsupported => {
                return WorthQueryDeclaredFamilyChecked::AsyncUnsupported(
                    WorthQueryAsyncDeclarationDenial::new(
                        support_report,
                        WorthQueryAsyncDeclarationSupport::Unsupported,
                    ),
                );
            }
        }
    }

    if !input.temporal_declaration_clauses().is_empty() {
        match I::Family::temporal_declaration_support() {
            WorthQueryTemporalDeclarationSupport::CanonicalIdentityOnly => {}
            WorthQueryTemporalDeclarationSupport::DeferredDebt => {
                return WorthQueryDeclaredFamilyChecked::TemporalDeferred(
                    WorthQueryTemporalDeclarationDenial::new(
                        support_report,
                        WorthQueryTemporalDeclarationSupport::DeferredDebt,
                    ),
                );
            }
            WorthQueryTemporalDeclarationSupport::Unsupported => {
                return WorthQueryDeclaredFamilyChecked::TemporalUnsupported(
                    WorthQueryTemporalDeclarationDenial::new(
                        support_report,
                        WorthQueryTemporalDeclarationSupport::Unsupported,
                    ),
                );
            }
        }
    }

    match worth_query_canonical_declaration(handle, input, version) {
        Ok(artifact) => WorthQueryDeclaredFamilyChecked::Admitted(artifact),
        Err(error) => WorthQueryDeclaredFamilyChecked::Canonicalization(error),
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationAdmissionError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
