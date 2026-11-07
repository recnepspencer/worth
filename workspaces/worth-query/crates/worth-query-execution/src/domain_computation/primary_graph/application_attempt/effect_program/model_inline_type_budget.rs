//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationEffectProgram<(), (), (), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/effect_program/model.rs", "WorthQueryApplicationEffectProgram", 128 * 1024);
        owner.field::<WorthQueryCompleteApplicationReadSet<
            (),
            (),
            (),
            (),
            WorthQueryProjectedApplicationMutation,
        >>("read_set");
        owner.field::<Vec<WorthQueryApplicationRealizedEffect>>("effects");
        owner.field::<Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>>("conditional_definition");
        owner.field::<crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture>("effect_posture");
        owner.field::<super::super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate>("output_correspondence");
        owner.field::<bool>("retain_output_demand_observation");
        owner.field::<bool>("retain_client_observation");
        owner.field::<&'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement]>("producer_required_invariants");
        owner.field::<Option<super::super::super::OutputCurrentnessFacts>>(
            "output_currentness_facts",
        );
        owner.field::<worth_relational::facade::content::RelationalContentAttachments>(
            "content_attachments",
        );
    }
}
