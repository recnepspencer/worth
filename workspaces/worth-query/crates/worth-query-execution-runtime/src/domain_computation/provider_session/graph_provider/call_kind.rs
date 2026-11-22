#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphProviderCallKind {
    Observe,
    Project,
    TouchEffect,
    CommitAdmission,
}

impl WorthQueryGraphProviderCallKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Project => "project",
            Self::TouchEffect => "touch-effect",
            Self::CommitAdmission => "commit-admission",
        }
    }
}
