pub(crate) use worth_relational_publication::internal_port::publication::cdc::planning::continuity_assessment::disposition;
mod evaluation;
mod failures;
mod normalized_proof;
mod selection;

#[cfg(test)]
mod tests;

pub(crate) use disposition::disposition_for_assessment;
pub(crate) use evaluation::assess_subscriber_continuity;
pub(crate) use selection::select_execution_envelopes;
