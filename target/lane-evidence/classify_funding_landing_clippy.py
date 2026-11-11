from pathlib import Path
import re,subprocess,json,posixpath
p=Path('target/lane-evidence/funding-landing-root-clippy.log')
rows=[]
for block in re.split(r'(?m)^error: ',p.read_text())[1:]:
    m=re.search(r'\A[^\n]*\n\s*(?:-->|:::)\s+([^\n]+\.rs):(\d+):(\d+)',block)
    if not m: continue
    path=posixpath.normpath(m.group(1).replace('\\','/'))
    lint=re.search(r'index.html#(\w+)',block)
    lifecycle=subprocess.run(['git','diff','--quiet','7d0e5189bc','codex/m3-p3-lifecycle','--',path]).returncode
    master=subprocess.run(['git','diff','--quiet','7d0e5189bc','origin/master','--',path]).returncode
    assert lifecycle in (0,1) and master in (0,1)
    rows.append({'path':path,'line':int(m.group(2)),'lint':lint.group(1) if lint else block.splitlines()[0],'lifecycle_diff_exit':lifecycle,'master_diff_exit':master})
Path('target/lane-evidence/funding-landing-clippy-diagnostics.json').write_text(json.dumps(rows,indent=2)+'\n')
for r in rows: print(r['lifecycle_diff_exit'],r['master_diff_exit'],r['lint'],r['path']+':'+str(r['line']))
print('TOTAL',len(rows),'paths',len(set(r['path'] for r in rows)))
