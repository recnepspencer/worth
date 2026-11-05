use {super::super::HandlerExecutionDenial, super::super::WorthQueryApplicationAttemptDenial, crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial};
use worth_relational::facade::content::RelationalContentDenial;

#[derive(Debug)]
pub enum ContentReadDenial {
    Field(HandlerExecutionDenial),
    Admission(WorthQueryValueDecodeDenial),
    Content(RelationalContentDenial),
    BindingContractMismatch,
    RequiredFieldAbsent,
    UnavailableBasis,
}
impl From<WorthQueryValueDecodeDenial> for ContentReadDenial {
    fn from(cause: WorthQueryValueDecodeDenial) -> Self {
        match cause {
            WorthQueryValueDecodeDenial::Content(cause) => Self::Content(cause),
            WorthQueryValueDecodeDenial::BindingContractMismatch => Self::BindingContractMismatch,
            WorthQueryValueDecodeDenial::RequiredContentFieldAbsent => Self::RequiredFieldAbsent,
            WorthQueryValueDecodeDenial::ContentBasisUnavailable => Self::UnavailableBasis,
            cause => Self::Admission(cause),
        }
    }
}
#[derive(Debug)]
pub enum ContentWriteDenial {
    Effect(WorthQueryApplicationAttemptDenial),
    Admission(WorthQueryValueDecodeDenial),
    Content(RelationalContentDenial),
}
impl std::fmt::Display for ContentReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "content field read refused: {self:?}")
    }
}
impl std::error::Error for ContentReadDenial {}
impl std::fmt::Display for ContentWriteDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "content field write refused: {self:?}")
    }
}
impl std::error::Error for ContentWriteDenial {}
