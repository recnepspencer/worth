use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum FactDecodeDenial {
    Format(String),
    Retention(StoreDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
}
impl From<String> for FactDecodeDenial {
    fn from(error: String) -> Self {
        Self::Format(error)
    }
}
impl std::fmt::Display for FactDecodeDenial {
    fn fmt(&self, writer: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Format(message) => writer.write_str(message),
            Self::Retention(denial) => std::fmt::Display::fmt(denial, writer),
            Self::Execution(cause) => write!(writer, "fact payload execution refused: {cause:?}"),
        }
    }
}
impl std::error::Error for FactDecodeDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Retention(denial) => Some(denial),
            Self::Format(_) | Self::Execution(_) => None,
        }
    }
}
