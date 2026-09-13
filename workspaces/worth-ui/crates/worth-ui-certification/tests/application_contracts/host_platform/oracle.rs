#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct OracleRect {
    pub identity: u16,
    pub bounds: [u16; 4],
    pub rgba: [u8; 4],
    pub order: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct OracleRectChange {
    pub identity: u16,
    pub previous: OracleRect,
    pub successor: OracleRect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OracleDelta {
    pub changes: Vec<OracleRectChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OracleExpectation {
    pub owner_delta_count: usize,
    pub damage: Vec<[i32; 4]>,
    pub ordered_identities: Vec<u16>,
    pub vacated_damage_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OracleDenial {
    OwnerDeltaDropped,
    DamageWidened,
    PaintOrderChanged,
    VacatedDamageOmitted,
}

pub(super) fn expectation(baseline: &[OracleRect], delta: &OracleDelta) -> OracleExpectation {
    let mut successor = baseline.to_vec();
    let mut damage = Vec::with_capacity(delta.changes.len() * 2);
    for change in &delta.changes {
        let slot = successor
            .iter_mut()
            .find(|row| row.identity == change.identity)
            .expect("world delta identity belongs to the baseline");
        assert_eq!(*slot, change.previous);
        *slot = change.successor;
        damage.extend([
            change.previous.bounds.map(i32::from),
            change.successor.bounds.map(i32::from),
        ]);
    }
    successor.sort_by_key(|row| row.order);
    OracleExpectation {
        owner_delta_count: delta.changes.len(),
        damage,
        ordered_identities: successor.iter().map(|row| row.identity).collect(),
        vacated_damage_count: delta.changes.len(),
    }
}

pub(super) fn removal_expectation(
    baseline: &[OracleRect],
    removed_count: usize,
) -> OracleExpectation {
    let successor = &baseline[removed_count..];
    let damage = baseline[..removed_count]
        .iter()
        .map(|row| row.bounds.map(i32::from))
        .collect::<Vec<_>>();
    OracleExpectation {
        owner_delta_count: removed_count,
        damage,
        ordered_identities: successor.iter().map(|row| row.identity).collect(),
        vacated_damage_count: removed_count,
    }
}

pub(super) fn adjudicate(
    expected: &OracleExpectation,
    candidate: &OracleExpectation,
) -> Result<(), OracleDenial> {
    if candidate.owner_delta_count != expected.owner_delta_count {
        return Err(OracleDenial::OwnerDeltaDropped);
    }
    if candidate.damage != expected.damage {
        return Err(OracleDenial::DamageWidened);
    }
    if candidate.ordered_identities != expected.ordered_identities {
        return Err(OracleDenial::PaintOrderChanged);
    }
    if candidate.vacated_damage_count != expected.vacated_damage_count {
        return Err(OracleDenial::VacatedDamageOmitted);
    }
    Ok(())
}

pub(super) fn ordered_pixel(rows: &[OracleRect], point: [u16; 2]) -> [u8; 4] {
    let mut ordered = rows.to_vec();
    ordered.sort_by_key(|row| row.order);
    ordered
        .into_iter()
        .filter(|row| contains(row.bounds, point))
        .map(|row| row.rgba)
        .next_back()
        .unwrap_or([0, 0, 0, 0])
}

fn contains(bounds: [u16; 4], point: [u16; 2]) -> bool {
    point[0] >= bounds[0]
        && point[1] >= bounds[1]
        && point[0] < bounds[0] + bounds[2]
        && point[1] < bounds[1] + bounds[3]
}
