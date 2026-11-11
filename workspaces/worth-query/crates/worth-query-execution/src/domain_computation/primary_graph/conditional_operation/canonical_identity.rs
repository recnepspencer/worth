use std::sync::Arc;

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

mod temporal_definition;
pub(super) use temporal_definition::prepare_temporal_node_definition;

const IDENTITY_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(16, 64 * 1024) {
        Some(budget) => budget,
        None => panic!("fixed conditional identity budget is valid"),
    };

#[derive(Clone)]
pub(super) struct WorthQueryTemporalBindingIdentity {
    digest: CanonicalDigestId,
    definition: CanonicalDigestId,
    support_identity: Arc<str>,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

#[derive(Clone)]
pub(super) struct WorthQueryTemporalRuntimeBindingIdentity {
    digest: CanonicalDigestId,
    definition: CanonicalDigestId,
    bridge_identity: Arc<str>,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

pub(super) struct TemporalBindingIdentityParts<'a> {
    pub node_authority: &'a str,
    pub node_definition: (CanonicalDigestId, WorthQueryCanonicalWorkEvidence),
    pub clock: &'a str,
    pub source: &'a str,
    pub timeline: &'a str,
    pub query: CanonicalDigestId,
    pub projector: &'a str,
    pub principal_source: &'a str,
    pub invoker: &'a str,
}

pub(super) struct TemporalRuntimeBindingIdentityParts<'a> {
    pub binding: &'a WorthQueryTemporalBindingIdentity,
    pub runtime_authority: u64,
    pub installation_runtime: u64,
    pub installation_generation: u64,
    pub provider: &'a str,
    pub branch: &'a str,
}

impl WorthQueryTemporalBindingIdentity {
    pub(super) fn digest(&self) -> CanonicalDigestId {
        self.digest
    }

    pub(super) fn support_identity(&self) -> &str {
        &self.support_identity
    }

    pub(super) fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

impl WorthQueryTemporalRuntimeBindingIdentity {
    pub(super) fn definition(&self) -> CanonicalDigestId {
        self.definition
    }
    pub(super) fn digest(&self) -> CanonicalDigestId {
        self.digest
    }

    pub(super) fn bridge_identity(&self) -> &Arc<str> {
        &self.bridge_identity
    }

    pub(super) fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

pub(super) fn prepare_temporal_binding_identity(
    parts: TemporalBindingIdentityParts<'_>,
) -> Result<WorthQueryTemporalBindingIdentity, CanonicalDigestDerivationDenial> {
    let mut material = CanonicalIdentityMaterial::new(
        "worth-query.temporal-conditional-binding",
        "worth-query-temporal-conditional-binding-v1",
    );
    material.text("node-authority", parts.node_authority);
    append_temporal_meaning(&mut material, &parts);
    let (digest, work) = material.derive()?;
    let mut stable = CanonicalIdentityMaterial::new(
        "worth-query.temporal-operation-definition",
        "worth-query-temporal-operation-definition-v1",
    );
    stable.digest("node-definition", parts.node_definition.0);
    append_temporal_meaning(&mut stable, &parts);
    let (definition, stable_work) = stable.derive()?;
    Ok(WorthQueryTemporalBindingIdentity {
        support_identity: Arc::from(format!("conditional:{}", digest.render_hex())),
        digest,
        canonical_work: work
            .combine(stable_work)
            .combine(parts.node_definition.1)
            .with_digest_text_materializations(1),
        definition,
    })
}

fn append_temporal_meaning(
    material: &mut CanonicalIdentityMaterial,
    parts: &TemporalBindingIdentityParts<'_>,
) {
    material.text("clock", parts.clock);
    material.text("source", parts.source);
    material.text("timeline", parts.timeline);
    material.digest("query", parts.query);
    material.text("projector", parts.projector);
    material.text("principal-source", parts.principal_source);
    material.text("invoker", parts.invoker);
}

pub(super) fn prepare_temporal_runtime_binding_identity(
    parts: TemporalRuntimeBindingIdentityParts<'_>,
) -> Result<WorthQueryTemporalRuntimeBindingIdentity, CanonicalDigestDerivationDenial> {
    let mut material = CanonicalIdentityMaterial::new(
        "worth-query.temporal-conditional-runtime-binding",
        "worth-query-temporal-conditional-runtime-binding-v1",
    );
    material.digest("binding", parts.binding.digest());
    material.unsigned_u64("runtime-authority", parts.runtime_authority);
    material.unsigned_u64("installation-runtime", parts.installation_runtime);
    material.unsigned_u64("installation-generation", parts.installation_generation);
    material.text("provider", parts.provider);
    material.text("branch", parts.branch);
    let (digest, work) = material.derive()?;
    Ok(WorthQueryTemporalRuntimeBindingIdentity {
        definition: parts.binding.definition,
        bridge_identity: Arc::from(format!("conditional-runtime:{}", digest.render_hex())),
        digest,
        canonical_work: work.with_digest_text_materializations(1),
    })
}

pub(super) struct CanonicalIdentityMaterial {
    domain: CanonicalBasisDomain,
    version: CanonicalizationRuleVersion,
    entries: Vec<CanonicalBasisEntry>,
}

impl CanonicalIdentityMaterial {
    pub(super) fn new(domain: &'static str, version: &'static str) -> Self {
        Self {
            domain: CanonicalBasisDomain::Future(domain),
            version: CanonicalizationRuleVersion::new(version)
                .expect("fixed conditional canonicalization rule is valid"),
            entries: Vec::new(),
        }
    }

    pub(super) fn text(&mut self, locus: &'static str, value: impl Into<String>) {
        self.push(locus, CanonicalBasisValue::ExactText(value.into().into()));
    }

    pub(super) fn digest(&mut self, locus: &'static str, value: CanonicalDigestId) {
        self.push(locus, CanonicalBasisValue::BytesDigest(value));
    }

    pub(super) fn unsigned_u64(&mut self, locus: &'static str, value: u64) {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value.into(),
            },
        );
    }

    pub(super) fn derive(
        self,
    ) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial>
    {
        let basis = prepare_canonical_basis_sequence(self.version, self.domain, self.entries)
            .into_result()
            .expect("conditional identity material is nonempty");
        let admitted = canonicalization()
            .digest()
            .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), IDENTITY_BUDGET)
            .into_result()?;
        let derived = canonicalization().digest().derive(admitted);
        Ok((
            CanonicalDigestId::new(*derived.value().bytes()),
            WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
        ))
    }

    fn push(&mut self, locus: &'static str, value: CanonicalBasisValue) {
        self.entries.push(CanonicalBasisEntry::new(
            self.domain,
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Identity,
            value,
        ));
    }
}
