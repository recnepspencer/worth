//! Why an authored correction candidate was refused before its native commit; the attempt's
//! provider registration translates it into a commit refusal.
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;

pub(in crate::domain_computation::primary_graph) enum AuthoredCorrectionCandidateDenial {
    Identity(&'static str),
    Interrupted(WorthQueryRequestInterruption),
}
