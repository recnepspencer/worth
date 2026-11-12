//! Absence of optional diagnostics never describes the fate of an owner effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticRecordAbsence {
    /// No record was retained in the requested live archive.
    NeverRetained,
    ExpiredUnderCountLimit,
    NotRetainedAcrossReopen,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticRecordReadDenial {
    ForeignOwner,
    Absent(DiagnosticRecordAbsence),
}
