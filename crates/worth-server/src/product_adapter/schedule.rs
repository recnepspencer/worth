use crate::{
    WorthServerOperationAuthorityKind, WorthServerOperationConcurrencyClass,
    WorthServerProductOperationSurfaceDenial, WorthServerProductOperationSurfaceDenialCode,
    WorthServerProductSession,
};

use super::WorthServerLoweredProductOperationPlan;

#[derive(Clone, Debug)]
pub struct WorthServerProductSchedulerAdmission {
    scheduler_lane: String,
    concurrency_class: WorthServerOperationConcurrencyClass,
    canonical_digest: String,
}

impl WorthServerProductSchedulerAdmission {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::string;
        string(&self.scheduler_lane).saturating_add(string(&self.canonical_digest))
    }

    fn new(
        scheduler_lane: String,
        concurrency_class: WorthServerOperationConcurrencyClass,
        canonical_digest: String,
    ) -> Self {
        Self {
            scheduler_lane,
            concurrency_class,
            canonical_digest,
        }
    }

    pub fn scheduler_lane(&self) -> &str {
        &self.scheduler_lane
    }

    pub fn concurrency_class(&self) -> WorthServerOperationConcurrencyClass {
        self.concurrency_class.clone()
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

#[derive(Clone, Debug)]
pub struct WorthServerScheduledProductOperation {
    plan: WorthServerLoweredProductOperationPlan,
    scheduler_admission: WorthServerProductSchedulerAdmission,
    admitted_product_session: Option<WorthServerProductSession>,
    canonical_digest: String,
}

impl WorthServerScheduledProductOperation {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::string;
        self.plan
            .owned_allocation_capacity_bytes()
            .saturating_add(self.scheduler_admission.owned_allocation_capacity_bytes())
            .saturating_add(string(&self.canonical_digest))
            .saturating_add(if self.admitted_product_session.is_some() {
                u64::MAX
            } else {
                0
            })
    }

    pub(crate) fn admit(
        plan: WorthServerLoweredProductOperationPlan,
        admitted_product_session: Option<WorthServerProductSession>,
    ) -> Result<Self, WorthServerProductOperationSurfaceDenial> {
        let scheduler_lane = derive_scheduler_lane(&plan)?;
        validate_product_scheduler_admission(&plan, &scheduler_lane)?;
        let canonical_digest = format!(
            "worth-server-scheduled-product-operation-v2|plan={}|lane={}|concurrency={}|product_session={}",
            plan.canonical_digest(),
            scheduler_lane,
            concurrency_label(plan.concurrency_class()),
            admitted_product_session
                .as_ref()
                .map(WorthServerProductSession::canonical_digest)
                .unwrap_or("none"),
        );
        let scheduler_admission = WorthServerProductSchedulerAdmission::new(
            scheduler_lane,
            plan.concurrency_class(),
            canonical_digest.clone(),
        );
        Ok(Self {
            plan,
            scheduler_admission,
            admitted_product_session,
            canonical_digest,
        })
    }

    pub fn plan(&self) -> &WorthServerLoweredProductOperationPlan {
        &self.plan
    }

    pub fn scheduler_admission(&self) -> &WorthServerProductSchedulerAdmission {
        &self.scheduler_admission
    }

    pub fn admitted_product_session(&self) -> Option<&WorthServerProductSession> {
        self.admitted_product_session.as_ref()
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

fn derive_scheduler_lane(
    plan: &WorthServerLoweredProductOperationPlan,
) -> Result<String, WorthServerProductOperationSurfaceDenial> {
    let authority_metadata = plan.operation_admission().authority_metadata();
    match plan
        .operation_admission()
        .authority_footprint()
        .authority_kind()
    {
        WorthServerOperationAuthorityKind::SharedReadOnly
        | WorthServerOperationAuthorityKind::DiagnosticsOnly => Ok("shared-read".to_string()),
        WorthServerOperationAuthorityKind::ProductDraftMutation => {
            let (product_session_identity, draft_scope) =
                authority_metadata.product_draft_scope().ok_or_else(|| {
                    invalid_product_scheduler_contract(
                        "product draft mutations require scheduler-visible draft scope metadata",
                    )
                })?;
            Ok(format!(
                "product-draft:{product_session_identity}:{draft_scope}"
            ))
        }
        WorthServerOperationAuthorityKind::DurableProductMutation => {
            authority_metadata
                .durable_product_mutation_scope()
                .ok_or_else(|| {
                    invalid_product_scheduler_contract(
                        "durable product mutations require scheduler-visible authority scope",
                    )
                })?;
            Ok(format!(
                "durable-product:{}",
                plan.operation_admission()
                    .authority_footprint()
                    .scope()
                    .canonical_digest()
            ))
        }
        WorthServerOperationAuthorityKind::ProductSessionCoordination => {
            let (target, coordination_lane) = authority_metadata
                .product_session_coordination_target()
                .ok_or_else(|| {
                    invalid_product_scheduler_contract(
                        "product session coordination requires scheduler-visible coordination metadata",
                    )
                })?;
            match target {
                crate::WorthServerProductSessionCoordinationTarget::ExistingSession {
                    product_session_identity,
                } => Ok(format!(
                    "product-session:{product_session_identity}:{coordination_lane}"
                )),
                crate::WorthServerProductSessionCoordinationTarget::SessionCreation => Ok(format!(
                    "product-session-create:{}:{coordination_lane}",
                    plan.operation_admission()
                        .authority_footprint()
                        .scope()
                        .canonical_digest()
                )),
            }
        }
        unsupported => Err(invalid_product_scheduler_contract(&format!(
            "product adapter scheduling does not admit authority kind `{}`",
            unsupported.as_str()
        ))),
    }
}

fn validate_product_scheduler_admission(
    plan: &WorthServerLoweredProductOperationPlan,
    scheduler_lane: &str,
) -> Result<(), WorthServerProductOperationSurfaceDenial> {
    let concurrency_class = plan.concurrency_class();
    let expected = if scheduler_lane == "shared-read" {
        WorthServerOperationConcurrencyClass::ConcurrentSharedRead
    } else {
        WorthServerOperationConcurrencyClass::SerializeDeterministically
    };
    if concurrency_class != expected {
        return Err(invalid_product_scheduler_contract(&format!(
            "product scheduler lane `{scheduler_lane}` requires `{}` concurrency, got `{}`",
            concurrency_label(expected),
            concurrency_label(concurrency_class),
        )));
    }
    Ok(())
}

fn concurrency_label(concurrency_class: WorthServerOperationConcurrencyClass) -> &'static str {
    match concurrency_class {
        WorthServerOperationConcurrencyClass::ConcurrentSharedRead => "shared-read",
        WorthServerOperationConcurrencyClass::SerializeDeterministically => {
            "serialize-deterministically"
        }
    }
}

fn invalid_product_scheduler_contract(detail: &str) -> WorthServerProductOperationSurfaceDenial {
    WorthServerProductOperationSurfaceDenial::new(
        WorthServerProductOperationSurfaceDenialCode::InvalidDeclaration,
        detail.to_string(),
    )
}
