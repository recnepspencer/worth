use crate::declarative_live::{DeclarativeLiveQueryRequest, DeclarativeLiveViewShape};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::remask_posture::WorthQueryRuntimeRemaskProjection;
use crate::runtime::{WorthQueryMutationTargetCollectionIdentity, WorthQueryRuntimeRemaskPosture};
use crate::subscription::SubscriptionActivationInput;

#[path = "signal_routing_receipt.rs"]
mod signal_routing_receipt;

pub use signal_routing_receipt::SignalInvalidationRoutingReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveViewDeclarationAdmissionReceipt {
    view_name: String,
    target_collection: WorthQueryMutationTargetCollectionIdentity,
    view_shape: DeclarativeLiveViewShape,
    query_projection_count: usize,
    result_field_count: usize,
    predicate_filter_count: usize,
    traversal_count: usize,
    ordering_count: usize,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl LiveViewDeclarationAdmissionReceipt {
    pub(crate) fn from_request(
        view_name: impl Into<String>,
        request: &DeclarativeLiveQueryRequest,
    ) -> Result<Self, IdentityDerivationDenial> {
        let view_name = view_name.into();
        let target_collection = request.target_collection_identity()?;
        let view_shape = request.view_shape().clone();
        let query_projection_count = request.query_projection().len();
        let result_field_count = request.result_fields().len();
        let predicate_filter_count = request.predicate_filters().len();
        let traversal_count = request.traversal().len();
        let ordering_count = request.ordering().len();
        let receipt_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "live-view-declaration-admission-receipt",
        )
        .field_value(WorthQueryEvidenceTag::new("view"), &view_name)
        .field_value(
            WorthQueryEvidenceTag::new("target"),
            target_collection.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("shape"), view_shape.as_str())
        .field_usize(
            WorthQueryEvidenceTag::new("query_projection_count"),
            query_projection_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("result_field_count"),
            result_field_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("predicate_filter_count"),
            predicate_filter_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("traversal_count"),
            traversal_count,
        )
        .field_usize(WorthQueryEvidenceTag::new("ordering_count"), ordering_count)
        .seal()?;

        Ok(Self {
            view_name,
            target_collection,
            view_shape,
            query_projection_count,
            result_field_count,
            predicate_filter_count,
            traversal_count,
            ordering_count,
            receipt_identity,
        })
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn target_collection_for_reporting(&self) -> &str {
        self.target_collection.as_str()
    }

    pub fn view_shape(&self) -> &DeclarativeLiveViewShape {
        &self.view_shape
    }

    pub fn view_shape_for_reporting(&self) -> &str {
        self.view_shape.as_str()
    }

    pub fn query_projection_count(&self) -> usize {
        self.query_projection_count
    }

    pub fn result_field_count(&self) -> usize {
        self.result_field_count
    }

    pub fn predicate_filter_count(&self) -> usize {
        self.predicate_filter_count
    }

    pub fn traversal_count(&self) -> usize {
        self.traversal_count
    }

    pub fn ordering_count(&self) -> usize {
        self.ordering_count
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub(crate) fn drift_from_request(
        &self,
        expected_view_name: &str,
        request: &DeclarativeLiveQueryRequest,
    ) -> Result<Option<String>, IdentityDerivationDenial> {
        if self.view_name() != expected_view_name {
            return Ok(Some(format!(
                "live-view admission receipt view drifted: expected `{expected_view_name}`, found `{}`",
                self.view_name()
            )));
        }
        let request_target_collection = request.target_collection_identity()?;
        if !self
            .target_collection
            .same_target_collection_as(&request_target_collection)
        {
            return Ok(Some(format!(
                "live-view admission receipt target drifted: expected `{}`, found `{}`",
                request_target_collection.as_str(),
                self.target_collection.as_str()
            )));
        }
        if self.view_shape() != request.view_shape()
            || self.query_projection_count() != request.query_projection().len()
            || self.result_field_count() != request.result_fields().len()
            || self.predicate_filter_count() != request.predicate_filters().len()
            || self.traversal_count() != request.traversal().len()
            || self.ordering_count() != request.ordering().len()
        {
            return Ok(Some(
                "live-view admission receipt shape drifted from the declared request".to_string(),
            ));
        }
        Ok(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionActivationReceipt {
    view_name: String,
    activation_identity: WorthQueryEvidenceIdentity,
    query_declaration_identity: WorthQueryEvidenceIdentity,
    bridge_declaration_identity: WorthQueryEvidenceIdentity,
    basis_binding_identity: WorthQueryEvidenceIdentity,
    signal_strategy_identity: WorthQueryEvidenceIdentity,
    support_identity: WorthQueryEvidenceIdentity,
    remask_posture: Option<WorthQueryRuntimeRemaskPosture>,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionActivationReceipt {
    pub(crate) fn from_activation(
        view_name: impl Into<String>,
        activation: &SubscriptionActivationInput,
        support_evidence_identity: WorthQueryEvidenceIdentity,
        remask_projection: Option<WorthQueryRuntimeRemaskProjection>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let view_name = view_name.into();
        let activation_identity = activation.evidence_identity().clone();
        let query_declaration_identity = activation.query_declaration_identity().clone();
        let bridge_declaration_identity = activation.bridge_declaration_identity().clone();
        let basis_binding_identity = activation.basis_binding_identity().clone();
        let signal_strategy_identity = activation.signal_strategy_identity().clone();
        let support_identity =
            subscription_activation_receipt_source_identity("support", &support_evidence_identity)?;
        let remask_posture = remask_projection.map(|projection| {
            WorthQueryRuntimeRemaskPosture::from_activation_projection(
                &projection,
                &support_identity,
                &basis_binding_identity,
            )
        })?;
        let receipt_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "subscription-activation-receipt",
        )
        .field_value(WorthQueryEvidenceTag::new("view"), &view_name)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("activation"),
            &activation_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_declaration"),
            &query_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_declaration"),
            &bridge_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_binding"),
            &basis_binding_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("signal_strategy"),
            &signal_strategy_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("support"), &support_identity)
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("remask"),
            remask_posture
                .as_ref()
                .map(WorthQueryRuntimeRemaskPosture::remask_identity)?,
        )
        .seal()?;

        Ok(Self {
            view_name,
            activation_identity,
            query_declaration_identity,
            bridge_declaration_identity,
            basis_binding_identity,
            signal_strategy_identity,
            support_identity,
            remask_posture,
            receipt_identity,
        })
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn activation_for_reporting(&self) -> &str {
        self.activation_identity.as_str()
    }

    pub fn activation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_identity
    }

    pub fn query_declaration_for_reporting(&self) -> &str {
        self.query_declaration_identity.as_str()
    }

    pub fn query_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_declaration_identity
    }

    pub fn bridge_declaration_for_reporting(&self) -> &str {
        self.bridge_declaration_identity.as_str()
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn basis_binding_for_reporting(&self) -> &str {
        self.basis_binding_identity.as_str()
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn signal_strategy_for_reporting(&self) -> &str {
        self.signal_strategy_identity.as_str()
    }

    pub fn signal_strategy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.signal_strategy_identity
    }

    pub fn support_for_reporting(&self) -> &str {
        self.support_identity.as_str()
    }

    pub fn support_evidence(&self) -> &str {
        self.support_for_reporting()
    }

    pub fn support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_identity
    }

    pub fn remask_posture(&self) -> Option<&WorthQueryRuntimeRemaskPosture> {
        self.remask_posture.as_ref()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub(crate) fn drift_from_activation(
        &self,
        expected_view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Option<String> {
        if self.view_name() != expected_view_name {
            return Some(format!(
                "subscription activation receipt view drifted: expected `{expected_view_name}`, found `{}`",
                self.view_name()
            ));
        }
        if typed_identity_drift(self.activation_identity(), activation.evidence_identity())
            || typed_identity_drift(
                self.query_declaration_identity(),
                activation.query_declaration_identity(),
            )
            || typed_identity_drift(
                self.bridge_declaration_identity(),
                activation.bridge_declaration_identity(),
            )
            || typed_identity_drift(
                self.basis_binding_identity(),
                activation.basis_binding_identity(),
            )
            || typed_identity_drift(
                self.signal_strategy_identity(),
                activation.signal_strategy_identity(),
            )
        {
            return Some(
                "subscription activation receipt drifted from activation input".to_string(),
            );
        }
        None
    }
}

fn typed_identity_drift(
    left: &WorthQueryEvidenceIdentity,
    right: &WorthQueryEvidenceIdentity,
) -> bool {
    !matches!(left.eq_same_scheme(right), Ok(true))
}

fn subscription_activation_receipt_source_identity(
    role: &str,
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_activation_receipt_source_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
            .seal()?,
    )
}

#[cfg(test)]
#[path = "receipts_tests.rs"]
#[cfg(test)]
mod tests;
