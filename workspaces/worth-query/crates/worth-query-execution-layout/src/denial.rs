//! Typed refusal vocabulary for lowering application and platform schema layouts.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySchemaLayoutDenialKind {
    ContentAdmissionRejected,
    InvalidSchemaMember,
    RelationalSchemaRejected,
    UnsupportedManagedSource,
    SourceProgramIdentityExhausted,
    OrderedPredecessorMembershipRequired,
    OrderedPredecessorMustBeToOne,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySchemaLayoutDenial {
    kind: WorthQuerySchemaLayoutDenialKind,
    subject: String,
    content_cause: Option<Box<worth_relational::facade::content::RelationalContentDenial>>,
}

impl WorthQuerySchemaLayoutDenial {
    pub(crate) fn new(kind: WorthQuerySchemaLayoutDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
            content_cause: None,
        }
    }

    pub(crate) fn with_content_cause(
        mut self,
        cause: worth_relational::facade::content::RelationalContentDenial,
    ) -> Self {
        self.content_cause = Some(Box::new(cause));
        self
    }

    pub const fn kind(&self) -> WorthQuerySchemaLayoutDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQuerySchemaLayoutDenialKind,
        String,
        Option<worth_relational::facade::content::RelationalContentDenial>,
    ) {
        (self.kind, self.subject, self.content_cause.map(|c| *c))
    }
}

impl std::fmt::Display for WorthQuerySchemaLayoutDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "primary graph schema layout denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQuerySchemaLayoutDenial {}
