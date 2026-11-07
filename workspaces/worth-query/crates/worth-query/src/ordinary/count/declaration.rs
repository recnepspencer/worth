use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::ordinary::read::{WorthQueryDeclaredReadIntent, WorthQueryReadNextAction};
use crate::runtime::{
    WorthQueryReadBuilder, WorthQueryReadDenial, WorthQueryReadDenialKind,
    WorthQueryReadGraphFamily,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryCountDeclarationIdentity(String);

impl WorthQueryCountDeclarationIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn from_declared_intent(
        intent: &WorthQueryDeclaredReadIntent,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self(
            worth_query_evidence_identity(WorthQueryEvidenceScope::ReadGraphDigest)
                .field_shape(WorthQueryEvidenceTag::new("stage"), "declared_count")
                .field_value(WorthQueryEvidenceTag::new("source_query"), intent.digest())
                .field_shape(WorthQueryEvidenceTag::new("aggregate"), "count_rows")
                .seal()?
                .as_str()
                .to_string(),
        ))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryCountDeclaration {
    identity: WorthQueryCountDeclarationIdentity,
    intent: WorthQueryDeclaredReadIntent,
}

impl WorthQueryCountDeclaration {
    pub fn identity(&self) -> &WorthQueryCountDeclarationIdentity {
        &self.identity
    }

    pub(crate) fn into_declared_intent(self) -> WorthQueryDeclaredReadIntent {
        self.intent
    }

    fn from_declared_intent(
        intent: WorthQueryDeclaredReadIntent,
    ) -> Result<Self, WorthQueryCountDeclarationStop> {
        if intent.family() != &WorthQueryReadGraphFamily::Collection {
            return Err(WorthQueryCountDeclarationStop::new(
                WorthQueryReadDenial::new(
                    WorthQueryReadDenialKind::AuthoringDenied,
                    "count aggregate declarations require a collection query",
                ),
            ));
        }
        let identity = WorthQueryCountDeclarationIdentity::from_declared_intent(&intent)
            .map_err(|cause| WorthQueryCountDeclarationStop::new(cause.into()))?;
        Ok(Self { identity, intent })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCountDeclarationStop {
    denial: WorthQueryReadDenial,
}

impl WorthQueryCountDeclarationStop {
    pub fn denial(&self) -> &WorthQueryReadDenial {
        &self.denial
    }

    pub fn next_action(&self) -> WorthQueryReadNextAction {
        WorthQueryReadNextAction::ReviseDeclaration
    }

    fn new(denial: WorthQueryReadDenial) -> Self {
        Self { denial }
    }
}

/// Declare a count over one bounded collection or composed collection read.
///
/// Query owns canonicalization, admission, aggregate planning, materialization,
/// and receipt construction. The caller supplies only collection meaning and
/// an authority-bearing read context.
pub fn declare_count(
    author: impl FnOnce(
        WorthQueryReadBuilder<WorthQueryDeclaredReadIntent>,
    ) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial>,
) -> Result<WorthQueryCountDeclaration, WorthQueryCountDeclarationStop> {
    let intent = author(WorthQueryReadBuilder::declaration())
        .map_err(WorthQueryCountDeclarationStop::new)?;
    WorthQueryCountDeclaration::from_declared_intent(intent)
}
