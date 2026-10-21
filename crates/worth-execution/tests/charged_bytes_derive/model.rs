use worth_execution::CanonicalBits;
use worth_foundational::facade::ChargedBytes;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, ChargedBytes)]
#[charged_bytes(crate = "worth_foundational::facade")]
pub struct ResidentValue {
    pub total: u64,
    pub labels: Vec<String>,
    #[serde(skip)]
    pub resident_notes: Option<Box<Vec<String>>>,
}

impl ResidentValue {
    pub fn empty() -> Self {
        Self {
            total: 0,
            labels: Vec::new(),
            resident_notes: None,
        }
    }

    pub fn with_notes(capacity: usize) -> Self {
        let mut value = Self {
            total: 7,
            labels: vec![String::from("north")],
            resident_notes: None,
        };
        if capacity > 0 {
            let mut note = String::with_capacity(capacity);
            note.push_str("cached");
            let mut notes = Vec::with_capacity(3);
            notes.push(note);
            value.resident_notes = Some(Box::new(notes));
        }
        value
    }
}

impl CanonicalBits for ResidentValue {
    fn canonical_len(&self) -> Option<usize> {
        (self.total, (&self.labels, &self.resident_notes)).canonical_len()
    }

    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        (self.total, (&self.labels, &self.resident_notes)).visit_canonical_bits(visitor)
    }
}

pub fn combine(left: &ResidentValue, right: &ResidentValue) -> ResidentValue {
    let mut labels = left.labels.clone();
    labels.extend(right.labels.iter().cloned());
    let mut notes = Vec::new();
    for value in [left, right] {
        if let Some(value) = &value.resident_notes {
            notes.extend(value.iter().cloned());
        }
    }
    ResidentValue {
        total: left.total + right.total,
        labels,
        resident_notes: (!notes.is_empty()).then(|| Box::new(notes)),
    }
}
