"""Prepare, but do not apply, the six outside-area mechanical lint fixes."""
from pathlib import Path
import difflib

changes = [
    ('crates/worth-relational/src/durability/artifact_payload_emission/request.rs',
     '        if let Work::Charge = self.work {\n            if let Err(cause) = account_work(&self.request, bytes.len() as u64) {\n                self.failure = Some(cause);\n                return Err(stopped());\n            }\n        }',
     '        if let Work::Charge = self.work\n            && let Err(cause) = account_work(&self.request, bytes.len() as u64)\n        {\n            self.failure = Some(cause);\n            return Err(stopped());\n        }'),
    ('crates/worth-relational/src/durability/checkpoint_admission/wire/emission.rs',
     '    let description = cut.clone();', '    let description = *cut;'),
    ('crates/worth-relational/src/durability/medium/recovery_source/home_preflight.rs',
     'source.admission_image().cloned().ok_or_else(|| Refusal::Native(',
     'source.admission_image().cloned().ok_or(Refusal::Native('),
    ('crates/worth-relational/src/history/data/required_output_positions/readmission/node.rs',
     '    if let Some(previous) = &context.previous {\n        if family::compare(previous, &route, context)? != std::cmp::Ordering::Less {\n            return Err(Refusal::InvalidProjectedMeaning);\n        }\n    }',
     '    if let Some(previous) = &context.previous\n        && family::compare(previous, &route, context)? != std::cmp::Ordering::Less\n    {\n        return Err(Refusal::InvalidProjectedMeaning);\n    }'),
    ('crates/worth-relational/src/runtime/state/subsystems/durability/checkpoint_selection/source_retirement.rs',
     '        if let Some(admission) = &mut state.admission {\n            if !admission.loan_source_retirement(handoff)? {\n                state.admission_loan = true;\n                return Ok(false);\n            }\n        }',
     '        if let Some(admission) = &mut state.admission\n            && !admission.loan_source_retirement(handoff)?\n        {\n            state.admission_loan = true;\n            return Ok(false);\n        }'),
    ('workspaces/worth-store/crates/worth-store-application/src/world_section/close.rs',
     '            &self\n                .composition',
     '            self\n                .composition'),
]
patch = ''
for name, old, new in changes:
    source = Path(name).read_bytes().decode('utf-8')
    if '\r\n' in source:
        assert source.count('\n') == source.count('\r\n'), name
        old, new = old.replace('\n', '\r\n'), new.replace('\n', '\r\n')
    assert source.count(old) == 1, name
    changed = source.replace(old, new)
    patch += ''.join(difflib.unified_diff(source.splitlines(True), changed.splitlines(True), fromfile='a/' + name, tofile='b/' + name))
Path('target/lane-evidence/proposed-outside-area-clippy.patch').write_bytes(patch.encode('utf-8'))
print('Prepared six exact mechanical fixes without changing any source file.')
