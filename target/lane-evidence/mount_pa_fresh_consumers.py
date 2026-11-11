from pathlib import Path
import re,json
E=Path('target/lane-evidence');assert 'APPROVE' in (E/'review-fresh-source-verdict.md').read_text()
D=Path('crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication')
cases=[]
for p in sorted((E/'pa-fresh-public-sources').glob('*.rs')):
 destination=D if p.stem=='checkpoint_birth_refusal' else D/'source_initial_inputs'
 assert not (destination/p.name).exists()
 s=p.read_text();(destination/p.name).write_text(s)
 cases+=re.findall(r'#\[test\]\s*fn (\w+)',s)
 parent=destination/'mod.rs';parent.write_text(parent.read_text()+f'\nmod {p.stem};\n')
assert len(cases)==5,cases
(E/'pa-fresh-public-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print('Mounted five approved fresh/birth courts; all originals still mounted')
