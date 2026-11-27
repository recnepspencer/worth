//! The role-assignment handler: it creates a standing review grant and links
//! the chosen member as its holder. The handler does not check who is
//! asking; the platform refuses the link when the member is the caller.
use worth_query_decl::facade::application_schema::{CanonicalDate, CanonicalTimestamp};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryOperationBodyDenial,
};

use super::grants::{ACTIVE, REVIEW};
use super::schema::*;

pub struct AssignRoleHandler;

type Denial = WorthQueryOperationBodyDenial;

impl OperationHandler<QmsSchema, AssignRoleBinding> for AssignRoleHandler {
    fn decide(
        &self,
        input: &AssignRole,
        reader: &mut DecisionReader<'_, '_, '_, QmsSchema, AssignRoleBinding>,
    ) -> HandlerResult<AssignDecision, Denial> {
        let observed = reader
            .resolve_entity(SiteFacts::site_id(), input.site.clone())
            .and_then(|site| {
                let member = reader.resolve_entity(MemberFacts::member_id(), input.member)?;
                Ok(AssignDecision {
                    site: reader.mutation_target(&site)?,
                    member: reader.mutation_target(&member)?,
                })
            });
        match observed {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &AssignRole,
        decision: AssignDecision,
        writer: &mut CandidateWriter<'_, QmsSchema, AssignRoleBinding>,
    ) -> HandlerResult<(), Denial> {
        let key = WorthQueryApplicationEntityKey::new(input.grant.clone()).expect("grant key");
        let written = writer.projected_entity(&decision.site).and_then(|site| {
            let member = writer.projected_entity(&decision.member)?;
            let grant = writer.create_entity_in_context(&site, RoleGrant::reference(), key)?;
            writer.initialize_field(&grant, RoleGrantFacts::grant_id(), input.grant.clone())?;
            writer.initialize_field(&grant, RoleGrantFacts::role(), REVIEW.to_owned())?;
            writer.initialize_field(&grant, RoleGrantFacts::purpose(), GrantPurpose::Quality)?;
            writer.initialize_field(&grant, RoleGrantFacts::grant_status(), ACTIVE.to_owned())?;
            writer.initialize_field(&grant, RoleGrantFacts::valid_from(), always_from())?;
            writer.initialize_field(&grant, RoleGrantFacts::valid_until(), always_until())?;
            writer.initialize_field(&grant, RoleGrantFacts::first_day(), first_day())?;
            writer.initialize_field(&grant, RoleGrantFacts::last_day(), last_day())?;
            writer.link(
                RoleHolder::reference(),
                format!("{}-holder", input.grant),
                &member,
                &grant,
            )?;
            writer.link(
                GrantSite::reference(),
                format!("{}-site", input.grant),
                &grant,
                &site,
            )
        });
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

pub fn always_from() -> CanonicalTimestamp {
    CanonicalTimestamp {
        micros_since_unix_epoch: i64::MIN,
    }
}

pub fn always_until() -> CanonicalTimestamp {
    CanonicalTimestamp {
        micros_since_unix_epoch: i64::MAX,
    }
}

pub fn first_day() -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch: i32::MIN,
    }
}

pub fn last_day() -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch: i32::MAX,
    }
}
