//! Re-validate a genuine recorded edit envelope with its stage list cut short.
use super::*;
use std::sync::Arc;

impl CanonicalCommitEnvelope {
    /// The recorded coupled event with its stage list cut to `len` sources,
    /// through the one header validation every reader runs.
    pub(crate) fn validate_stage_list_cut_for_test(
        &self,
        len: usize,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let event = self
            .coupled_authored_publication
            .as_ref()
            .expect("a recorded edit envelope");
        let CanonicalCoupledAuthoredSources::Stages(sources) = &event.sources else {
            panic!("the court commit records authored stages");
        };
        let participants = event
            .participants
            .iter()
            .map(|part| coupled_authored::CanonicalCoupledParticipant {
                origin: Arc::clone(&part.origin),
                retirement: part.retirement.clone(),
            })
            .collect();
        let cut = CanonicalCoupledAuthoredPublication {
            sources: CanonicalCoupledAuthoredSources::Stages(sources[..len].to_vec()),
            participants,
            live_inputs: Default::default(),
            _decoded_memory: None,
        };
        self.validate_coupled_authored_header(&cut, context)
    }
}
