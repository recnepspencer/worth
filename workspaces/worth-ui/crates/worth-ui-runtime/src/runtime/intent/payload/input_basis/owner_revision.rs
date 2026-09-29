#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIntentInputOwnerRevision {
    Query(UiIntentQueryInputRevision),
    Application(UiIntentApplicationFactRevision),
    Draft(UiIntentDraftInputRevision),
    Expression(UiIntentExpressionInputRevision),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIntentQueryInputRevision {
    field: crate::capability::UiIntentPayloadFieldDescriptor,
    revision: worth_ui_query_binding::UiProjectionInputRevision,
}

/// The revision of one application fact as an Intent payload field read it:
/// the payload field it fed, the fact identity and the fact revision. It
/// proves which fact revision a payload was projected from. Expression
/// operands use `UiIntentApplicationInputRevision` instead, which also names
/// the active generation the fact was read in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIntentApplicationFactRevision {
    field: crate::capability::UiIntentPayloadFieldDescriptor,
    identity: Box<str>,
    revision: u64,
}

/// The expression result one Intent payload field read: the payload field it
/// fed, the expression identity and the outcome revision. A payload stays
/// current only while the expression owner retains that exact outcome
/// revision for the active generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIntentExpressionInputRevision {
    field: crate::capability::UiIntentPayloadFieldDescriptor,
    identity: Box<str>,
    outcome_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentDraftInputRevision {
    field: crate::capability::UiIntentPayloadFieldDescriptor,
    session: crate::runtime::interaction::UiDraftSessionIdentity,
    input_revision: Option<u64>,
    draft_revision: u64,
}

impl UiIntentInputOwnerRevision {
    pub(crate) fn query(
        field: crate::capability::UiIntentPayloadFieldDescriptor,
        revision: worth_ui_query_binding::UiProjectionInputRevision,
    ) -> Self {
        Self::Query(UiIntentQueryInputRevision { field, revision })
    }

    pub(crate) fn application(
        field: crate::capability::UiIntentPayloadFieldDescriptor,
        identity: impl Into<Box<str>>,
        revision: u64,
    ) -> Self {
        Self::Application(UiIntentApplicationFactRevision {
            field,
            identity: identity.into(),
            revision,
        })
    }

    pub(crate) const fn draft(
        field: crate::capability::UiIntentPayloadFieldDescriptor,
        session: crate::runtime::interaction::UiDraftSessionIdentity,
        input_revision: Option<u64>,
        draft_revision: u64,
    ) -> Self {
        Self::Draft(UiIntentDraftInputRevision {
            field,
            session,
            input_revision,
            draft_revision,
        })
    }

    pub(crate) fn expression(
        field: crate::capability::UiIntentPayloadFieldDescriptor,
        result: &crate::runtime::expression::UiExpressionResultReference,
    ) -> Self {
        Self::Expression(UiIntentExpressionInputRevision {
            field,
            identity: result.identity().into(),
            outcome_revision: result.outcome_revision(),
        })
    }
}

impl UiIntentQueryInputRevision {
    pub const fn field(&self) -> crate::capability::UiIntentPayloadFieldDescriptor {
        self.field
    }

    pub const fn revision(&self) -> &worth_ui_query_binding::UiProjectionInputRevision {
        &self.revision
    }
}

impl UiIntentApplicationFactRevision {
    pub const fn field(&self) -> crate::capability::UiIntentPayloadFieldDescriptor {
        self.field
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }
}

impl UiIntentExpressionInputRevision {
    pub const fn field(&self) -> crate::capability::UiIntentPayloadFieldDescriptor {
        self.field
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn outcome_revision(&self) -> u64 {
        self.outcome_revision
    }
}

impl UiIntentDraftInputRevision {
    pub const fn field(self) -> crate::capability::UiIntentPayloadFieldDescriptor {
        self.field
    }

    pub const fn session(self) -> crate::runtime::interaction::UiDraftSessionIdentity {
        self.session
    }

    pub const fn input_revision(self) -> Option<u64> {
        self.input_revision
    }

    pub const fn draft_revision(self) -> u64 {
        self.draft_revision
    }
}
