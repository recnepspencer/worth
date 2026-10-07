mod installation;

use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionResourceEnvelope,
    WorthQueryExecutionStrategyContract,
};

use crate::admission_digest::hash_parts;

pub trait WorthQueryExecutionCapacityReservation: Send {}

impl<T: Send> WorthQueryExecutionCapacityReservation for T {}

pub trait WorthQueryExecutionCapacityPort: Send + Sync {
    fn capacity_subject_identity(&self) -> &str;

    /// Owner-declared Work and backing for one successful reservation. The
    /// caller admits this before `try_reserve` can change capacity state.
    fn reservation_preflight_cost(&self) -> Option<(u64, u64)>;

    fn try_reserve(&self) -> Option<Box<dyn WorthQueryExecutionCapacityReservation>>;
}

#[derive(Debug)]
pub enum WorthQueryGraphProviderLookupStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

#[derive(Clone)]
pub struct WorthQueryExecutionResourceSupport {
    provider: WorthQueryExecutionProviderFamily,
    access_product: WorthQueryExecutionAccessProductFamily,
    allocator: WorthQueryExecutionAllocatorFamily,
    envelope: WorthQueryExecutionResourceEnvelope,
    capacity: Arc<dyn WorthQueryExecutionCapacityPort>,
    identity: Arc<str>,
}

impl WorthQueryExecutionResourceSupport {
    pub fn new(
        provider: WorthQueryExecutionProviderFamily,
        access_product: WorthQueryExecutionAccessProductFamily,
        allocator: WorthQueryExecutionAllocatorFamily,
        envelope: WorthQueryExecutionResourceEnvelope,
        capacity: Arc<dyn WorthQueryExecutionCapacityPort>,
    ) -> Self {
        let identity = Arc::<str>::from(hash_parts(&[
            "worth_query_execution_resource_support_v1".into(),
            format!("provider:{}", provider.as_str()),
            format!("access:{}", access_product.as_str()),
            format!("allocator:{}", allocator.as_str()),
            format!("capacity:{}", capacity.capacity_subject_identity()),
            format!("mode:{}", envelope.mode().as_str()),
            format!("safe-point:{}", envelope.cancellation_safe_point().as_str()),
            format!(
                "degradation:{}",
                envelope
                    .degradation()
                    .map_or("complete", |degradation| degradation.as_str())
            ),
            format!(
                "partial-effect:{}",
                envelope.partial_effect_posture().as_str()
            ),
            format!(
                "yielded-state:{}",
                envelope.yielded_state_posture().as_str()
            ),
            format!(
                "retained-progress:{}",
                envelope.retained_progress_posture().as_str()
            ),
            format!(
                "scale:{}",
                envelope
                    .scale_ceilings()
                    .iter()
                    .map(|(axis, value)| format!("{}={value}", axis.as_str()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "resources:{}",
                envelope
                    .resource_ceilings()
                    .iter()
                    .map(|(dimension, value)| format!("{}={value}", dimension.as_str()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ]));
        Self {
            provider,
            access_product,
            allocator,
            envelope,
            capacity,
            identity,
        }
    }

    pub fn provider(&self) -> &WorthQueryExecutionProviderFamily {
        &self.provider
    }

    pub fn access_product(&self) -> &WorthQueryExecutionAccessProductFamily {
        &self.access_product
    }

    pub fn allocator(&self) -> &WorthQueryExecutionAllocatorFamily {
        &self.allocator
    }

    pub fn envelope(&self) -> &WorthQueryExecutionResourceEnvelope {
        &self.envelope
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn capacity_subject_identity(&self) -> &str {
        self.capacity.capacity_subject_identity()
    }

    pub(super) fn capacity(&self) -> &Arc<dyn WorthQueryExecutionCapacityPort> {
        &self.capacity
    }

    pub(super) fn has_same_capacity_authority(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.capacity, &other.capacity)
    }

    fn supports(&self, strategy: &WorthQueryExecutionStrategyContract) -> bool {
        let required = strategy.provider_requirements();
        self.provider == *required.provider()
            && self.access_product == *required.access_product()
            && self.allocator == *required.allocator()
            && covers(&self.envelope, strategy.envelope())
    }
}

impl std::fmt::Debug for WorthQueryExecutionResourceSupport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryExecutionResourceSupport")
            .field("provider", &self.provider)
            .field("access_product", &self.access_product)
            .field("allocator", &self.allocator)
            .field("envelope", &self.envelope)
            .field("capacity_subject", &self.capacity_subject_identity())
            .finish()
    }
}

impl PartialEq for WorthQueryExecutionResourceSupport {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider
            && self.access_product == other.access_product
            && self.allocator == other.allocator
            && self.envelope == other.envelope
            && self.capacity_subject_identity() == other.capacity_subject_identity()
            && self.has_same_capacity_authority(other)
    }
}

impl Eq for WorthQueryExecutionResourceSupport {}

fn covers(
    support: &WorthQueryExecutionResourceEnvelope,
    admitted: &WorthQueryExecutionResourceEnvelope,
) -> bool {
    admitted
        .scale_ceilings()
        .iter()
        .all(|(axis, value)| support.admits_scale(axis, value))
        && admitted
            .resource_ceilings()
            .iter()
            .all(|(dimension, value)| value <= support.resource_ceiling(dimension))
        && admitted.mode() == support.mode()
        && admitted.cancellation_safe_point() == support.cancellation_safe_point()
        && admitted.degradation() == support.degradation()
        && admitted.partial_effect_posture() == support.partial_effect_posture()
        && admitted.yielded_state_posture() == support.yielded_state_posture()
        && admitted.retained_progress_posture() == support.retained_progress_posture()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExecutionResourceSupportSnapshot {
    installed: Arc<InstalledResourceSupportSnapshot>,
}

#[derive(Debug, Eq, PartialEq)]
struct InstalledResourceSupportSnapshot {
    executor: WorthQueryExecutionResourceSupport,
    conditional_nodes: Vec<(String, WorthQueryExecutionResourceSupport)>,
    graph_providers: Vec<(String, WorthQueryExecutionResourceSupport)>,
    commit_providers: Vec<(String, WorthQueryExecutionResourceSupport)>,
    parallel_admission: Option<WorthQueryExecutionResourceSupport>,
    identity: Arc<str>,
}

impl WorthQueryExecutionResourceSupportSnapshot {
    pub(crate) fn has_same_installed_authority(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.installed, &other.installed)
    }

    pub fn executor(&self) -> &WorthQueryExecutionResourceSupport {
        &self.installed.executor
    }

    pub fn conditional_nodes(&self) -> &[(String, WorthQueryExecutionResourceSupport)] {
        &self.installed.conditional_nodes
    }

    pub fn graph_providers(&self) -> &[(String, WorthQueryExecutionResourceSupport)] {
        &self.installed.graph_providers
    }

    pub fn graph_provider(&self, role: &str) -> Option<&WorthQueryExecutionResourceSupport> {
        self.graph_provider_admitted(role, &mut |_, _| Ok::<(), std::convert::Infallible>(()))
            .expect("ordinary installed support lookup has no resource refusal")
    }

    pub fn graph_provider_admitted<Stop>(
        &self,
        role: &str,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Option<&WorthQueryExecutionResourceSupport>, WorthQueryGraphProviderLookupStop<Stop>>
    {
        let mut lower = 0;
        let mut upper = self.installed.graph_providers.len();
        while lower < upper {
            // Selected header inspection precedes the text width/read.
            admit(1, 0).map_err(WorthQueryGraphProviderLookupStop::Admission)?;
            let middle = lower + (upper - lower) / 2;
            let candidate = &self.installed.graph_providers[middle].0;
            let work = candidate
                .len()
                .checked_add(role.len())
                .and_then(|width| width.checked_add(1))
                .and_then(|width| u64::try_from(width).ok())
                .ok_or(WorthQueryGraphProviderLookupStop::AccountingOverflow)?;
            admit(work, 0).map_err(WorthQueryGraphProviderLookupStop::Admission)?;
            match candidate.as_str().cmp(role) {
                std::cmp::Ordering::Less => lower = middle + 1,
                std::cmp::Ordering::Greater => upper = middle,
                std::cmp::Ordering::Equal => {
                    return Ok(Some(&self.installed.graph_providers[middle].1))
                }
            }
        }
        Ok(None)
    }

    pub fn commit_providers(&self) -> &[(String, WorthQueryExecutionResourceSupport)] {
        &self.installed.commit_providers
    }

    pub fn parallel_admission(&self) -> Option<&WorthQueryExecutionResourceSupport> {
        self.installed.parallel_admission.as_ref()
    }

    pub(super) fn all_supports(&self) -> impl Iterator<Item = &WorthQueryExecutionResourceSupport> {
        let installed = &self.installed;
        std::iter::once(&installed.executor)
            .chain(
                installed
                    .conditional_nodes
                    .iter()
                    .map(|(_, support)| support),
            )
            .chain(installed.graph_providers.iter().map(|(_, support)| support))
            .chain(
                installed
                    .commit_providers
                    .iter()
                    .map(|(_, support)| support),
            )
            .chain(installed.parallel_admission.iter())
    }

    pub fn identity(&self) -> &str {
        &self.installed.identity
    }

    pub(super) fn supports(&self, strategy: &WorthQueryExecutionStrategyContract) -> bool {
        let installed = &self.installed;
        installed.executor.supports(strategy)
            && installed
                .conditional_nodes
                .iter()
                .all(|(_, support)| support.supports(strategy))
            && installed
                .graph_providers
                .iter()
                .all(|(_, support)| support.supports(strategy))
            && installed
                .commit_providers
                .iter()
                .all(|(_, support)| support.supports(strategy))
            && installed
                .parallel_admission
                .as_ref()
                .is_none_or(|support| support.supports(strategy))
    }

    pub(super) fn first_mismatch(
        &self,
        strategy: &WorthQueryExecutionStrategyContract,
    ) -> Option<(String, &WorthQueryExecutionResourceSupport)> {
        let installed = &self.installed;
        if !installed.executor.supports(strategy) {
            return Some(("executor".into(), &installed.executor));
        }
        if let Some((location, support)) = installed
            .conditional_nodes
            .iter()
            .find(|(_, support)| !support.supports(strategy))
        {
            return Some((format!("conditional node `{location}`"), support));
        }
        if let Some((role, support)) = installed
            .graph_providers
            .iter()
            .find(|(_, support)| !support.supports(strategy))
        {
            return Some((format!("graph role `{role}`"), support));
        }
        if let Some((group, support)) = installed
            .commit_providers
            .iter()
            .find(|(_, support)| !support.supports(strategy))
        {
            return Some((format!("commit group `{group}`"), support));
        }
        installed
            .parallel_admission
            .as_ref()
            .filter(|support| !support.supports(strategy))
            .map(|support| ("parallel admission provider".into(), support))
    }
}
