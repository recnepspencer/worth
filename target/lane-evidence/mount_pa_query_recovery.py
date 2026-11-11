from pathlib import Path
import json,re
E=Path('target/lane-evidence');assert 'APPROVE' in (E/'review-query-recovery-source2-verdict.md').read_text()
D=Path('crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication')
cases=[]
for p in sorted((E/'pa-query-recovery-sources').glob('*.rs')):
 assert not (D/p.name).exists(),p.name
 s=p.read_text();(D/p.name).write_text(s);cases+=re.findall(r'#\[test\]\s*fn (\w+)',s)
 parent=D/'mod.rs';parent.write_text(parent.read_text()+f'\nmod {p.stem};\n')
assert set(cases)==set(json.loads((E/'pa-query-recovery-cases.json').read_text())) and len(cases)==4,cases
print('Mounted4publicquery/topologycases; originalroots remain until qualification')
