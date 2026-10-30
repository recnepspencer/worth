//! Result absence is distinct from the schema's child-field presence law.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryResultFieldPresence {
    Required,
    Optional,
    OptionalAspect,
}
impl ApplicationQueryResultFieldPresence {
    pub const fn schema_presence(self) -> crate::application_schema::ApplicationFieldPresence {
        match self {
            Self::Optional => crate::application_schema::ApplicationFieldPresence::Optional,
            Self::Required | Self::OptionalAspect => {
                crate::application_schema::ApplicationFieldPresence::Required
            }
        }
    }
    pub const fn is_optional_result(self) -> bool {
        !matches!(self, Self::Required)
    }
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
            Self::OptionalAspect => "optional-aspect",
        }
    }
}
