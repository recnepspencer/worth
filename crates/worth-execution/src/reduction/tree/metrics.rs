//! Optional accounting for explicit cost contracts; native reduction carries none.
use super::{ReductionDenial, ReductionRunStop};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReductionAccounting {
    pub structural_visits: u64,
    pub recombined_nodes: u64,
    pub combine_calls: u64,
    pub charged_work: u64,
    pub charged_span: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReductionMetrics {
    accounting: Option<ReductionAccounting>,
}
impl Default for ReductionMetrics {
    fn default() -> Self {
        Self {
            accounting: Some(ReductionAccounting::default()),
        }
    }
}
impl ReductionMetrics {
    pub const fn native() -> Self {
        Self { accounting: None }
    }
    pub const fn has_accounting(self) -> bool {
        self.accounting.is_some()
    }
    pub const fn accounted_metrics(self) -> Option<ReductionAccounting> {
        self.accounting
    }
    pub(crate) fn accounted_mut(&mut self) -> Option<&mut ReductionAccounting> {
        self.accounting.as_mut()
    }
    /// Explicitly supplied accounting metadata, never a native measurement.
    pub const fn from_accounting(accounting: ReductionAccounting) -> Self {
        Self {
            accounting: Some(accounting),
        }
    }
    pub(crate) fn for_mode(accounted: bool) -> Self {
        if accounted {
            Self::default()
        } else {
            Self::native()
        }
    }
    pub(crate) fn add(self, other: Self) -> Result<Self, ReductionRunStop<()>> {
        let (left, right) = match (self.accounting, other.accounting) {
            (None, None) => return Ok(Self::native()),
            (Some(left), Some(right)) => (left, right),
            _ => {
                return Err(ReductionRunStop::Denial(
                    ReductionDenial::AccountingUnavailable,
                ))
            }
        };
        let add = |a: u64, b: u64| {
            a.checked_add(b)
                .ok_or_else(ReductionRunStop::work_counter_overflow)
        };
        Ok(Self::from_accounting(ReductionAccounting {
            structural_visits: add(left.structural_visits, right.structural_visits)?,
            recombined_nodes: add(left.recombined_nodes, right.recombined_nodes)?,
            combine_calls: add(left.combine_calls, right.combine_calls)?,
            charged_work: add(left.charged_work, right.charged_work)?,
            charged_span: add(left.charged_span, right.charged_span)?,
        }))
    }
    pub(super) fn record_visit(&mut self) -> Result<(), ReductionRunStop<()>> {
        let Some(accounting) = self.accounting.as_mut() else {
            return Ok(());
        };
        accounting.structural_visits = accounting
            .structural_visits
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        accounting.charged_work = accounting
            .charged_work
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        accounting.charged_span = accounting
            .charged_span
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        Ok(())
    }
    pub(super) fn record_combine(&mut self) -> Result<(), ReductionRunStop<()>> {
        let Some(accounting) = self.accounting.as_mut() else {
            return Ok(());
        };
        accounting.combine_calls = accounting
            .combine_calls
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        accounting.charged_work = accounting
            .charged_work
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        accounting.charged_span = accounting
            .charged_span
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        Ok(())
    }
    pub(super) fn record_node(&mut self) -> Result<(), ReductionRunStop<()>> {
        let Some(accounting) = self.accounting.as_mut() else {
            return Ok(());
        };
        accounting.recombined_nodes = accounting
            .recombined_nodes
            .checked_add(1)
            .ok_or_else(ReductionRunStop::work_counter_overflow)?;
        Ok(())
    }
}
