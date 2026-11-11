from pathlib import Path
import json,subprocess,hashlib
root=Path('target/lane-evidence')
prior=json.loads((root/'funding-final-source-state.json').read_text())['files']
paths=set(prior)
paths.update(subprocess.check_output(['git','diff','--name-only','--','crates','workspaces','tools/boundary-check/config/certification-construction-callers.txt'],text=True,stderr=subprocess.DEVNULL).splitlines())
paths.update(subprocess.check_output(['git','ls-files','--others','--exclude-standard','--','crates','workspaces'],text=True).splitlines())
files={p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in sorted(paths)}
(root/'funding-landing-source-state.json').write_text(json.dumps({'review_base_head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'files':files},indent=2)+'\n')
print('Frozen source/config files:',len(files))
