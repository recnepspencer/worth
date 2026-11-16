//! Descriptive assessment handoff; decoding neither adopts nor advances the original.
use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedAssessment<D> {
    required: Vec<u8>,
    original: Vec<u8>,
    demand: D,
    maximum_settlement_attempts: Option<std::num::NonZeroUsize>,
}

impl<S, Spec, D> WorthQueryOriginalWorkflowAssessment<S, Spec, D>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    /// Encode occurrence, retained demand and caller restrictions, with no live owner.
    pub fn to_recorded_bytes(&self) -> Result<Vec<u8>, rmp_serde::encode::Error>
    where
        D: serde::Serialize,
    {
        rmp_serde::to_vec(&RecordedAssessment {
            required: self.required.to_recorded_bytes()?,
            original: self.original.to_recorded_bytes()?,
            demand: &self.demand,
            maximum_settlement_attempts: self.controls.maximum_settlement_attempts(),
        })
    }

    /// Fresh workflow preparation, demand equality and Home readmission remain required.
    pub fn from_recorded_bytes(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error>
    where
        D: serde::de::DeserializeOwned,
    {
        let recorded: RecordedAssessment<D> = rmp_serde::from_slice(bytes)?;
        let mut controls = WorthQueryOutputDemandControls::host_policy();
        if let Some(maximum) = recorded.maximum_settlement_attempts {
            controls = controls.settlement_attempts(maximum);
        }
        Ok(Self {
            required: RequiredWorkflowAssessment::from_recorded_bytes(&recorded.required)?,
            original: WorthQueryOriginalOutputDemand::from_recorded_bytes(&recorded.original)?,
            demand: recorded.demand,
            controls,
            marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RecordedAssessment;

    #[test]
    fn descriptor_preserves_demand_and_restrictions_and_refuses_zero_limit() {
        let value = RecordedAssessment {
            required: vec![1],
            original: vec![2],
            demand: 7_u64,
            maximum_settlement_attempts: std::num::NonZeroUsize::new(3),
        };
        let encoded = rmp_serde::to_vec(&value).unwrap();
        let decoded: RecordedAssessment<u64> = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(decoded.demand, 7);
        assert_eq!(decoded.maximum_settlement_attempts.unwrap().get(), 3);
        assert_eq!(decoded.required, [1]);
        assert_eq!(decoded.original, [2]);
        // MessagePack [[], [], 7, 0]: a restriction cannot silently become host policy.
        assert!(
            rmp_serde::from_slice::<RecordedAssessment<u64>>(&[0x94, 0x90, 0x90, 7, 0]).is_err()
        );
        assert!(
            rmp_serde::from_slice::<RecordedAssessment<u64>>(&encoded[..encoded.len() - 1])
                .is_err()
        );
    }
}
