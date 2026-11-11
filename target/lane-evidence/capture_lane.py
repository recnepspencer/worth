import subprocess,os,sys,datetime,shlex,re
from pathlib import Path
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
lane=sys.argv[1]
command=sys.argv[2:]
env=os.environ.copy()
env.pop('CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER',None)
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
dirty=bool(subprocess.check_output(['git','status','--short'],text=True).strip())
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
log=Path('target/lane-evidence')/(lane+'.log')
with log.open('w',encoding='utf-8',newline='') as out:
    def write(text):
        out.write(text);out.flush();sys.stdout.write(text);sys.stdout.flush()
    write('command: '+subprocess.list2cmdline(command)+'\nHEAD: '+head+'\ngit_status_short_empty: '+str(not dirty).lower()+'\nstart_utc: '+now()+'\nenvironment: removed stale CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER override\n')
    proc=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,env=env)
    counts=(0,0,0)
    for raw in iter(proc.stdout.readline,b''):
        line=raw.decode('utf-8',errors='replace')
        write(line)
        summary=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',line)
        if summary: counts=tuple(a+int(b) for a,b in zip(counts,summary.groups()))
        if line.startswith('EVIDENCE_TEST_COUNTS: '):
            counts=tuple(int(value) for value in line.split(': ',1)[1].strip().split(','))
    code=proc.wait()
    write('\nend_utc: '+now()+'\nexit_code: '+str(code)+f'\ntests_passed: {counts[0]}\ntests_failed: {counts[1]}\ntests_ignored: {counts[2]}\n')
sys.exit(code)
