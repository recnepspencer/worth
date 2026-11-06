use super::super::ApplicationSchemaDeclarationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaContributionDenial {
    InvalidIdentity,
    DuplicateIdentity,
}

impl std::fmt::Display for ApplicationSchemaContributionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application schema contribution denied: {self:?}"
        )
    }
}

impl std::error::Error for ApplicationSchemaContributionDenial {}

impl From<ApplicationSchemaContributionDenial> for ApplicationSchemaDeclarationDenial {
    fn from(denial: ApplicationSchemaContributionDenial) -> Self {
        match denial {
            ApplicationSchemaContributionDenial::InvalidIdentity => {
                Self::InvalidContributionIdentity
            }
            ApplicationSchemaContributionDenial::DuplicateIdentity => {
                Self::DuplicateContributionIdentity
            }
        }
    }
}
