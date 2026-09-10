use super::support::*;

pub(super) struct PreviewReceiptInspectionFixture {
    pub(super) receipt: WorthQueryPreviewIntentReceipt,
    pub(super) inspection: WorthQueryPreviewIntentReceiptInspection,
}

pub(super) fn preview_receipt_with_basis<const N: usize>(
    basis_evidence: [&'static str; N],
    intent_name: &str,
    input: WorthQueryIntentInput,
) -> PreviewReceiptInspectionFixture {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(HostilePreviewBasis { basis_evidence })
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(TestIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let mut preview = runtime
        .preview_with_options(
            test_session_label("preview basis delimiter pressure"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview session should be admitted");
    let receipt = preview
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            intent_name,
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            input,
        ))
        .expect("preview intent should be admitted");
    let inspection = runtime
        .inspect_preview_intent_receipt(&receipt)
        .expect("preview receipt inspection should be available");

    PreviewReceiptInspectionFixture {
        receipt,
        inspection,
    }
}

struct HostilePreviewBasis<const N: usize> {
    basis_evidence: [&'static str; N],
}

impl<const N: usize> WorthQueryRuntimePreviewBasisAdapter for HostilePreviewBasis<N> {
    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        Ok(WorthQueryPreviewBasisAdmission::new(
            authority,
            label.clone(),
            effect_policy,
            WorthQueryBasisAdmissionEvidenceRow::rows_from_values(
                self.basis_evidence.iter().copied(),
            ),
        ))
    }
}
