//! Fresh Ability observation under the retained request's one preparation meter.

use std::mem::size_of;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::authorization::{
    RelationalAuthorizationBudgetedObservationStop, RelationalAuthorizationObservationAdmission,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathCloneStop,
    RelationalAuthorizationPlanAdmissionStop,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::super::bridge_observation::{
    lower_bridge_observation_admitted, BridgeObservationPreparationStop,
};
use super::super::super::installed_policy::{
    InstalledPolicyAdmissionStop, WorthQueryInstalledAuthorizationPolicy,
};
use super::super::super::{
    WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
use super::super::validation::{denial, validate_decision};
use super::super::WorthQueryConventionalAuthorizationDecisionPermit;
use super::super::WorthQueryConventionalAuthorizationObservation;
use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrimaryGraphApplicationRuntime,
};

#[derive(Debug)]
pub(in crate::domain_computation) enum AdmittedQueryAuthorizationStop {
    Authorization(WorthQueryOperationAuthorizationDenial),
    Preparation(CompanionPreflightStop),
    AccountingOverflow,
}

impl From<WorthQueryOperationAuthorizationDenial> for AdmittedQueryAuthorizationStop {
    fn from(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self::Authorization(denial)
    }
}

struct QueryObservationAdmission<'a>(&'a mut InvalidationEditAdmission);

impl RelationalAuthorizationObservationAdmission for QueryObservationAdmission<'_> {
    type Stop = CompanionPreflightStop;

    fn prepare(&mut self, work: u64, bytes: u64) -> Result<(), Self::Stop> {
        self.0.charge_external_work(work)?;
        self.0.admit_read_scratch(bytes)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn observe_query_authorization_requirement_admitted<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: worth_relational::facade::snapshots::SnapshotHandle,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, AdmittedQueryAuthorizationStop> {
        let observation = WorthQueryConventionalAuthorizationObservation {
            session_identity,
            relational,
            snapshot,
            principal: access.principal(),
            scope_identity: access.scope(),
            binding_identity: query.binding_identity(),
            requirements: std::slice::from_ref(requirement),
        };
        // The one-element decision Vec is retained by the admitted Query plan.
        prepare(
            admission,
            1,
            size_of::<WorthQueryAuthorizationDecisionFact>(),
        )?;
        Ok(vec![observation.observe_one_admitted(
            self,
            requirement,
            admission,
        )?])
    }
}

impl<Schema, Principal, PrincipalIdentity, Scope>
    WorthQueryConventionalAuthorizationObservation<'_, Schema, Principal, PrincipalIdentity, Scope>
where
    Schema: ApplicationSchema,
{
    fn observe_one_admitted(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryAuthorizationDecisionFact, AdmittedQueryAuthorizationStop> {
        self.prepare_static_checks(runtime, requirement, admission)?;
        self.validate_scope(requirement)?;
        let installed = runtime
            .authorization
            .policy_admitted(requirement, |work, bytes| {
                QueryObservationAdmission(admission).prepare(work, bytes)
            })
            .map_err(map_policy_stop)?;
        let rules = installed.bridge_rule_bindings().len();
        prepare(admission, rules, 0)?;
        let mut installed_checks = rules;
        for binding in installed.bridge_rule_bindings() {
            installed_checks = installed_checks
                .checked_add(binding.rule().requirements().len())
                .and_then(|n| n.checked_add(binding.path_indices().len()))
                .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?;
        }
        prepare(admission, installed_checks, 0)?;
        self.validate_installation(runtime, installed, requirement)?;
        let evidence = self.observe_relational_admitted(installed, requirement, admission)?;
        let dependency_identity = *evidence.observation_identity().bytes();
        let bridge_observation = lower_bridge_observation_admitted(
            installed,
            &evidence,
            dependency_identity,
            requirement.policy(),
            runtime.authorization.bridge(),
            |work, bytes| QueryObservationAdmission(admission).prepare(work, bytes),
        )
        .map_err(map_bridge_stop)?;
        let bridge = runtime
            .authorization
            .bridge()
            .evaluate(bridge_observation)
            .map_err(|_| {
                AdmittedQueryAuthorizationStop::Authorization(denial(
                    WorthQueryOperationAuthorizationDenialKind::BridgeEvaluationRejected,
                    requirement.policy(),
                ))
            })?;
        validate_decision(
            runtime.authorization.bridge(),
            &evidence,
            &bridge,
            dependency_identity,
            requirement.policy(),
        )?;
        Ok(
            WorthQueryAuthorizationDecisionFact::from_conventional_observation(
                WorthQueryConventionalAuthorizationDecisionPermit::new(),
                self.session_identity,
                evidence,
                bridge,
            ),
        )
    }

    fn prepare_static_checks(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), AdmittedQueryAuthorizationStop> {
        let installed_count = runtime
            .authorization
            .bridge()
            .correspondence_count()
            .checked_add(1)
            .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?;
        let levels = usize::BITS as usize - installed_count.leading_zeros() as usize;
        let names = requirement
            .ability()
            .len()
            .checked_add(requirement.scope_entity().len())
            .and_then(|n| n.checked_add(requirement.policy().len()))
            .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?;
        // Scope comparison and installed Bridge correspondence match happen
        // before native observation. They compare variable names and rule
        // identity arrays; their refusal also constructs one typed subject.
        let checks = levels
            .checked_mul(11)
            .and_then(|n| n.checked_add(requirement.policy_paths().len()))
            .and_then(|n| n.checked_add(names.checked_mul(2)?))
            .and_then(|n| n.checked_add(requirement.policy().len()))
            .and_then(|n| n.checked_add(3))
            .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?;
        let denial_backing = requirement
            .scope_entity()
            .len()
            .max(requirement.policy().len())
            .checked_add(size_of::<WorthQueryOperationAuthorizationDenialKind>())
            .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?;
        prepare(admission, checks, denial_backing)
    }

    fn observe_relational_admitted(
        &self,
        installed: &WorthQueryInstalledAuthorizationPolicy,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
        AdmittedQueryAuthorizationStop,
    > {
        let branch_bytes = self.snapshot.branch_id().0.len();
        prepare(
            admission,
            branch_bytes
                .checked_add(1)
                .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?,
            branch_bytes,
        )?;
        let path_bytes = installed
            .relational_paths()
            .len()
            .checked_mul(size_of::<
                worth_relational::facade::authorization::RelationalAuthorizationPathPlan,
            >())
            .ok_or(AdmittedQueryAuthorizationStop::AccountingOverflow)?;
        prepare(admission, installed.relational_paths().len(), path_bytes)?;
        let mut paths = Vec::with_capacity(installed.relational_paths().len());
        for path in installed.relational_paths() {
            paths.push(
                path.try_clone_admitted(|work, bytes| {
                    QueryObservationAdmission(admission).prepare(work, bytes)
                })
                .map_err(map_clone_stop)?,
            );
        }
        let plan = RelationalAuthorizationObservationPlan::try_new_owned_admitted(
            self.snapshot.clone(),
            self.principal.principal_entity_id(),
            self.scope_identity.entity_id(),
            installed.principal_kind(),
            installed.scope_kind(),
            paths,
            Vec::new(),
            |work, bytes| QueryObservationAdmission(admission).prepare(work, bytes),
        )
        .map_err(|stop| match stop {
            RelationalAuthorizationPlanAdmissionStop::Admission(stop) => {
                AdmittedQueryAuthorizationStop::Preparation(stop)
            }
            RelationalAuthorizationPlanAdmissionStop::AccountingOverflow => {
                AdmittedQueryAuthorizationStop::AccountingOverflow
            }
            RelationalAuthorizationPlanAdmissionStop::Plan(_) => {
                AdmittedQueryAuthorizationStop::Authorization(denial(
                    WorthQueryOperationAuthorizationDenialKind::InvalidInstalledPolicy,
                    requirement.policy(),
                ))
            }
        })?;
        self.relational
            .observe_authorization_budgeted(plan, &mut QueryObservationAdmission(admission))
            .map_err(|stop| match stop {
                RelationalAuthorizationBudgetedObservationStop::Admission(stop) => {
                    AdmittedQueryAuthorizationStop::Preparation(stop)
                }
                RelationalAuthorizationBudgetedObservationStop::AccountingOverflow => {
                    AdmittedQueryAuthorizationStop::AccountingOverflow
                }
                RelationalAuthorizationBudgetedObservationStop::Native(_)
                | RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired => {
                    AdmittedQueryAuthorizationStop::Authorization(denial(
                        WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected,
                        requirement.policy(),
                    ))
                }
            })
    }
}

fn prepare(
    admission: &mut InvalidationEditAdmission,
    work: usize,
    bytes: usize,
) -> Result<(), AdmittedQueryAuthorizationStop> {
    QueryObservationAdmission(admission)
        .prepare(
            u64::try_from(work).map_err(|_| AdmittedQueryAuthorizationStop::AccountingOverflow)?,
            u64::try_from(bytes).map_err(|_| AdmittedQueryAuthorizationStop::AccountingOverflow)?,
        )
        .map_err(AdmittedQueryAuthorizationStop::Preparation)
}

fn map_policy_stop(
    stop: InstalledPolicyAdmissionStop<CompanionPreflightStop>,
) -> AdmittedQueryAuthorizationStop {
    match stop {
        InstalledPolicyAdmissionStop::Admission(stop) => {
            AdmittedQueryAuthorizationStop::Preparation(stop)
        }
        InstalledPolicyAdmissionStop::AccountingOverflow => {
            AdmittedQueryAuthorizationStop::AccountingOverflow
        }
        InstalledPolicyAdmissionStop::Policy(denial) => {
            AdmittedQueryAuthorizationStop::Authorization(denial)
        }
    }
}

fn map_bridge_stop(
    stop: BridgeObservationPreparationStop<CompanionPreflightStop>,
) -> AdmittedQueryAuthorizationStop {
    match stop {
        BridgeObservationPreparationStop::Admission(stop) => {
            AdmittedQueryAuthorizationStop::Preparation(stop)
        }
        BridgeObservationPreparationStop::AccountingOverflow => {
            AdmittedQueryAuthorizationStop::AccountingOverflow
        }
        BridgeObservationPreparationStop::Observation(denial) => {
            AdmittedQueryAuthorizationStop::Authorization(denial)
        }
    }
}

fn map_clone_stop(
    stop: RelationalAuthorizationPathCloneStop<CompanionPreflightStop>,
) -> AdmittedQueryAuthorizationStop {
    match stop {
        RelationalAuthorizationPathCloneStop::Admission(stop) => {
            AdmittedQueryAuthorizationStop::Preparation(stop)
        }
        RelationalAuthorizationPathCloneStop::AccountingOverflow => {
            AdmittedQueryAuthorizationStop::AccountingOverflow
        }
    }
}
