//! Source read support currentness stop vocabulary.
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub enum SourceReadSupportCurrentnessStop {
    PrincipalStale,
    AuthorizationStale,
    Admission(CompanionPreflightStop),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    PrincipalUnavailable,
    PrincipalWorkExhausted,
}
