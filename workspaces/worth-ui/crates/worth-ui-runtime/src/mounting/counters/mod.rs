use worth_ui_host_contract::UiHostPresentationCostReport;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountWorkClass {
    InitialMount,
    SemanticDelta,
    BatchDelta,
    SurfaceOnly,
    UnchangedReuse,
    ComparisonRequired,
    RejectedPreparation,
    RejectedPresentation,
    IndeterminatePresentation,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiMountNamedCounters {
    considered: u64,
    minted: u64,
    reused: u64,
    retired: u64,
    rejected: u64,
    retained: u64,
    presented: u64,
    coalesced: u64,
    overflowed: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountCostReport {
    work_class: UiMountWorkClass,
    initial_mounted_instances: u64,
    changed_mounted_instances: u64,
    index_entries_touched: u64,
    replaced_batch_rows: u64,
    replaced_batch_bytes: u64,
    surface_instance_pairs: u64,
    changed_binding_generations: u64,
    appearance_motion_commands_visited: u64,
    named: UiMountNamedCounters,
    adapter: UiHostPresentationCostReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountStageCounters {
    report: UiMountCostReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountCostOverflow;

impl UiMountStageCounters {
    pub(crate) fn begin(work_class: UiMountWorkClass) -> Self {
        Self {
            report: UiMountCostReport {
                work_class,
                initial_mounted_instances: 0,
                changed_mounted_instances: 0,
                index_entries_touched: 0,
                replaced_batch_rows: 0,
                replaced_batch_bytes: 0,
                surface_instance_pairs: 0,
                changed_binding_generations: 0,
                appearance_motion_commands_visited: 0,
                named: UiMountNamedCounters::default(),
                adapter: UiHostPresentationCostReport::default(),
            },
        }
    }

    pub(crate) fn consider(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.named.considered, count)
    }

    pub(crate) fn mint_initial(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.initial_mounted_instances, count)?;
        add(&mut self.report.named.minted, count)
    }

    pub(crate) fn mint_changed(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.changed_mounted_instances, count)?;
        add(&mut self.report.named.minted, count)
    }

    pub(crate) fn mint_compared(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.named.minted, count)
    }

    pub(crate) fn record_projected_instances(
        &mut self,
        count: usize,
    ) -> Result<(), UiMountCostOverflow> {
        match self.report.work_class {
            UiMountWorkClass::InitialMount => self.mint_initial(count),
            UiMountWorkClass::SemanticDelta | UiMountWorkClass::BatchDelta => {
                self.mint_changed(count)
            }
            UiMountWorkClass::SurfaceOnly
            | UiMountWorkClass::UnchangedReuse
            | UiMountWorkClass::RejectedPreparation
            | UiMountWorkClass::RejectedPresentation
            | UiMountWorkClass::IndeterminatePresentation => {
                add(&mut self.report.named.reused, count)
            }
            UiMountWorkClass::ComparisonRequired => self.mint_compared(count),
        }
    }

    pub(crate) fn touch_indexes(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.index_entries_touched, count)
    }

    pub(crate) fn project_surface_pairs(
        &mut self,
        count: usize,
    ) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.surface_instance_pairs, count)
    }

    pub(crate) fn change_bindings(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.changed_binding_generations, count)
    }

    pub(crate) fn replace_rows<Row>(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.replaced_batch_rows, count)?;
        let bytes = count
            .checked_mul(std::mem::size_of::<Row>())
            .ok_or(UiMountCostOverflow)?;
        add(&mut self.report.replaced_batch_bytes, bytes)
    }

    pub(crate) fn reuse(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.named.reused, count)
    }

    pub(crate) fn retire(&mut self, count: usize) -> Result<(), UiMountCostOverflow> {
        add(&mut self.report.named.retired, count)
    }

    pub(crate) fn coalesce(&mut self, count: u64) -> Result<(), UiMountCostOverflow> {
        add_u64(&mut self.report.named.coalesced, count)
    }

    pub(crate) fn record_overflow(&mut self, overflowed: bool) -> Result<(), UiMountCostOverflow> {
        add_u64(&mut self.report.named.overflowed, u64::from(overflowed))
    }

    pub(crate) fn finish(self) -> UiMountCostReport {
        self.report
    }
}

impl UiMountCostReport {
    pub const fn work_class(self) -> UiMountWorkClass {
        self.work_class
    }

    pub const fn initial_mounted_instances(self) -> u64 {
        self.initial_mounted_instances
    }

    pub const fn changed_mounted_instances(self) -> u64 {
        self.changed_mounted_instances
    }

    pub const fn index_entries_touched(self) -> u64 {
        self.index_entries_touched
    }

    pub const fn replaced_batch_rows(self) -> u64 {
        self.replaced_batch_rows
    }

    pub const fn replaced_batch_bytes(self) -> u64 {
        self.replaced_batch_bytes
    }

    pub const fn surface_instance_pairs(self) -> u64 {
        self.surface_instance_pairs
    }

    pub const fn changed_binding_generations(self) -> u64 {
        self.changed_binding_generations
    }

    pub const fn appearance_motion_commands_visited(self) -> u64 {
        self.appearance_motion_commands_visited
    }

    pub(crate) fn record_appearance_motion_commands_visited(&mut self, count: usize) {
        self.appearance_motion_commands_visited =
            u64::try_from(count).expect("mounted command count fits the u64 cost surface");
    }

    pub const fn adapter(self) -> UiHostPresentationCostReport {
        self.adapter
    }

    pub const fn named(self) -> UiMountNamedCounters {
        self.named
    }

    pub(crate) fn with_adapter(
        mut self,
        adapter: UiHostPresentationCostReport,
    ) -> Result<Self, UiMountCostOverflow> {
        self.adapter = self
            .adapter
            .checked_add(adapter)
            .map_err(|_| UiMountCostOverflow)?;
        add_u64(&mut self.named.presented, adapter.presented_surfaces())?;
        Ok(self)
    }

    pub(crate) fn reclassified(mut self, work_class: UiMountWorkClass) -> Self {
        self.work_class = work_class;
        self
    }

    pub(crate) fn with_retained(mut self, count: usize) -> Result<Self, UiMountCostOverflow> {
        add(&mut self.named.retained, count)?;
        Ok(self)
    }

    pub(crate) fn with_rejected(mut self, count: usize) -> Result<Self, UiMountCostOverflow> {
        add(&mut self.named.rejected, count)?;
        Ok(self)
    }

    pub(crate) fn with_cost_overflow(mut self) -> Result<Self, UiMountCostOverflow> {
        add_u64(&mut self.named.overflowed, 1)?;
        Ok(self)
    }

    pub(crate) fn unchanged_reuse() -> Self {
        let mut report = UiMountStageCounters::begin(UiMountWorkClass::UnchangedReuse).finish();
        report.named.reused = 1;
        report
    }
}

impl UiMountNamedCounters {
    pub const fn considered(self) -> u64 {
        self.considered
    }

    pub const fn minted(self) -> u64 {
        self.minted
    }

    pub const fn reused(self) -> u64 {
        self.reused
    }

    pub const fn retired(self) -> u64 {
        self.retired
    }

    pub const fn rejected(self) -> u64 {
        self.rejected
    }

    pub const fn retained(self) -> u64 {
        self.retained
    }

    pub const fn presented(self) -> u64 {
        self.presented
    }

    pub const fn coalesced(self) -> u64 {
        self.coalesced
    }

    pub const fn overflowed(self) -> u64 {
        self.overflowed
    }
}

fn add(target: &mut u64, count: usize) -> Result<(), UiMountCostOverflow> {
    let count = u64::try_from(count).map_err(|_| UiMountCostOverflow)?;
    add_u64(target, count)?;
    Ok(())
}

fn add_u64(target: &mut u64, count: u64) -> Result<(), UiMountCostOverflow> {
    *target = target.checked_add(count).ok_or(UiMountCostOverflow)?;
    Ok(())
}
