use super::*;
use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_operation::WorthQueryCreateOutput;
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;

pub struct PriorCycleDecisionMember<Schema: TopologySchemaBinding> {
    pub(super) role: String,
    pub(super) target: WorthQueryInvariantMutationTarget<Schema, Body>,
    pub(super) replacement_y: PositiveLength,
}

pub struct PriorCycleDecision<Schema: TopologySchemaBinding> {
    pub(super) members: Vec<PriorCycleDecisionMember<Schema>>,
}

pub(super) fn observe_prior_cycle<Schema: TopologySchemaBinding>(
    input: &PriorCycleAdjustment,
    reader: &mut DecisionReader<'_, '_, '_, Schema, PriorCycleAdjustmentBinding<Schema>>,
) -> HandlerResult<PriorCycleDecision<Schema>, PriorCycleAdjustmentDenial> {
    let prior = if let Some(suffix) = &input.member_suffix {
        match reader.prior_output_member_if_present::<
            crate::PlanarEditBinding<Schema>,
            crate::PlanarCreatedOutputs<Schema>,
            WorthQueryCreateOutput,
        >(suffix) {
            Ok(Some(identity)) => vec![(format!("created.{suffix}"), identity)],
            Ok(None) => return HandlerResult::DomainDenied(PriorCycleAdjustmentDenial::NoPriorCycle),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
    } else {
        match reader.prior_output_family_if_present::<
        crate::PlanarEditBinding<Schema>,
        crate::PlanarCreatedOutputs<Schema>,
    >() {
        Ok(Some(prior)) => prior.into_iter().map(|member| {
            let role = member.role().to_owned();
            (role, member.into_identity())
        }).collect(),
        Ok(None) => return HandlerResult::DomainDenied(PriorCycleAdjustmentDenial::NoPriorCycle),
        Err(error) => return HandlerResult::ExecutionDenied(error),
        }
    };
    if prior.is_empty() {
        return HandlerResult::DomainDenied(PriorCycleAdjustmentDenial::NoPriorCycle);
    }
    let mut members = Vec::with_capacity(prior.len());
    for (role, identity) in prior {
        let current_y = match reader.field(&identity, PositionY::reference()) {
            Ok(Some(value)) => value,
            Ok(None) => {
                return HandlerResult::DomainDenied(PriorCycleAdjustmentDenial::MissingCoordinate)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let Some(replacement_y) = PositiveLength::get(&current_y)
            .checked_add(PositiveLength::get(&input.offset_y))
            .and_then(PositiveLength::new)
        else {
            return HandlerResult::ExecutionDenied(
                worth_query_host::facade::primary_graph::HandlerExecutionDenial::new(
                    std::io::Error::other(
                        "prior cycle Y offset overflows its declared value binding",
                    ),
                ),
            );
        };
        let target = match reader.mutation_target(&identity) {
            Ok(target) => target,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        members.push(PriorCycleDecisionMember {
            role,
            target,
            replacement_y,
        });
    }
    HandlerResult::Completed(PriorCycleDecision { members })
}
