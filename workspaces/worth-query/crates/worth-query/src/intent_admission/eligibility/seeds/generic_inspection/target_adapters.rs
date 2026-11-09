use super::*;

impl<'a, T> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryLiveView<T> {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::live_view(self)
    }
}

impl<'a, T> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryEffectHandle<T> {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::effect(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryWriteReceipt {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::write_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryBatchWriteReceipt {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::batch_write_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryIntentReceipt {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::intent_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryIntentDenialEvidence {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::intent_denial(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryEffectIntentReceipt {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::effect_intent_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a>
    for &'a WorthQueryPreviewHandleBindingEvidence
{
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::preview_binding(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryPreviewOutcome {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::preview_outcome(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryPreviewIntentReceipt {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::preview_intent_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryBranchIntentReceipt {
    fn into_generic_inspection_intent_seed(
        self,
    ) -> Result<WorthQueryGenericInspectionIntentSeed, IdentityDerivationDenial> {
        WorthQueryGenericInspectionIntentSeed::branch_intent_receipt(self)
    }
}
