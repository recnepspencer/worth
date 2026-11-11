"""Read both main-thread overflow paths from the existing compiled executables."""
from pathlib import Path
import subprocess,sys,json,re
r=Path.cwd();e=r/'target/lane-evidence'
s=(e/'stack_probe.py').read_text()
a=s.index('meta=json.loads(');z=s.index("print('DEBUG COMMAND:'",a)
s=s[:a]+'''exe=sys.argv[1]
import subprocess
command=subprocess.list2cmdline([exe])
'''+s[z:]
(e/'head_closure_main_probe.py').write_text(s)
sys.path.insert(0,str(e));from lifecycle_compiled_frames import Frames
for name in ('invariant_reads','required_feature','shared_execution'):
 exe=r/f'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/target/debug/examples/{name}.exe'
 code=subprocess.run([sys.executable,str(e/'run_evidence.py'),'layout-r3-'+name+'-stack-diagnostic',sys.executable,str(e/'head_closure_main_probe.py'),str(exe)]).returncode
 assert code==0
 log=e/('layout-r3-'+name+'-stack-diagnostic.log');s=log.read_text()
 base=int(re.search(r'IMAGE_BASE (0x[0-9a-f]+)',s)[1],16);pe=Frames(str(exe));rows=[]
 for line in s.splitlines():
  if m:=re.match(r'(\d+) (0x[0-9a-f]+) (0x[0-9a-f]+) (.+)',line):
   pc=int(m[2],16);rows.append({'index':int(m[1]),'name':m[4],'pc':m[2],'bytes':pe.frame(pc-base)})
 result={'exe':str(exe),'trace':str(log),'frames':rows,'resolved_subtotal':sum(x['bytes'] or 0 for x in rows),'unresolved':sum(x['bytes'] is None for x in rows)}
 (e/('layout-r3-'+name+'-frames.json')).write_text(json.dumps(result,indent=2))
 print(name,len(rows),'frames','resolved subtotal',result['resolved_subtotal'],'unresolved',result['unresolved'],flush=True)
 for x in rows:
  if (x['bytes'] or 0)>=32768:print(x['bytes'],x['name'],flush=True)
