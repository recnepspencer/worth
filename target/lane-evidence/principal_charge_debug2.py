"""Diagnostic only: first actual WorkCeiling branch, existing dev binary."""
import ctypes as c, ctypes.wintypes as w, json,struct,sys,subprocess
from pathlib import Path
k=c.WinDLL('kernel32',use_last_error=True);d=c.WinDLL('dbghelp',use_last_error=True);P=c.c_void_p
class SI(c.Structure):
 _fields_=[('cb',w.DWORD),('reserved',w.LPWSTR),('desktop',w.LPWSTR),('title',w.LPWSTR),('x',w.DWORD),('y',w.DWORD),('xs',w.DWORD),('ys',w.DWORD),('xc',w.DWORD),('yc',w.DWORD),('fill',w.DWORD),('flags',w.DWORD),('show',w.WORD),('reserved2',w.WORD),('r2',P),('stdin',w.HANDLE),('stdout',w.HANDLE),('stderr',w.HANDLE)]
class PI(c.Structure):_fields_=[('process',w.HANDLE),('thread',w.HANDLE),('pid',w.DWORD),('tid',w.DWORD)]
class Event(c.Structure):_fields_=[('code',w.DWORD),('pid',w.DWORD),('tid',w.DWORD),('padding',w.DWORD),('data',c.c_byte*160)]
class Line(c.Structure):_fields_=[('size',w.DWORD),('key',P),('number',w.DWORD),('file',c.c_char_p),('address',c.c_ulonglong)]
k.CreateProcessW.argtypes=[w.LPCWSTR,w.LPWSTR,P,P,w.BOOL,w.DWORD,P,w.LPCWSTR,c.POINTER(SI),c.POINTER(PI)]
k.OpenThread.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenThread.restype=w.HANDLE
for name in ['GetThreadContext','SetThreadContext']:getattr(k,name).argtypes=[w.HANDLE,P]
k.WaitForDebugEvent.argtypes=[c.POINTER(Event),w.DWORD];k.ContinueDebugEvent.argtypes=[w.DWORD,w.DWORD,w.DWORD]
k.TerminateProcess.argtypes=[w.HANDLE,w.UINT];k.GetStdHandle.argtypes=[w.DWORD];k.GetStdHandle.restype=w.HANDLE;k.CloseHandle.argtypes=[w.HANDLE]
for name in ['ReadProcessMemory','WriteProcessMemory']:getattr(k,name).argtypes=[w.HANDLE,P,P,c.c_size_t,c.POINTER(c.c_size_t)]
k.FlushInstructionCache.argtypes=[w.HANDLE,P,c.c_size_t]
d.SymInitializeW.argtypes=[w.HANDLE,w.LPCWSTR,w.BOOL];d.SymSetOptions.argtypes=[w.DWORD]
d.SymGetLineFromName64.argtypes=[w.HANDLE,c.c_char_p,c.c_char_p,w.DWORD,c.POINTER(w.LONG),c.POINTER(Line)]
d.SymGetLineFromAddr64.argtypes=[w.HANDLE,c.c_ulonglong,c.POINTER(w.DWORD),c.POINTER(Line)]
d.SymFromAddr.argtypes=[w.HANDLE,c.c_ulonglong,c.POINTER(c.c_ulonglong),P]
d.StackWalk64.argtypes=[w.DWORD,w.HANDLE,w.HANDLE,P,P,P,P,P,P]
meta=json.loads(Path(sys.argv[1]).read_text());binary,test=sys.argv[2:]
exe=next(v['binary-path'] for v in meta['rust-binaries'].values() if v['binary-name']==binary)
command=subprocess.list2cmdline([exe,'--exact',test,'--nocapture']);print('DEBUG_COMMAND',command,flush=True)
si=SI();si.cb=c.sizeof(si);si.flags=0x100;si.stdin=k.GetStdHandle(-10&0xffffffff);si.stdout=k.GetStdHandle(-11&0xffffffff);si.stderr=k.GetStdHandle(-12&0xffffffff)
pi=PI();args=c.create_unicode_buffer(command);assert k.CreateProcessW(None,args,None,None,True,2,None,None,c.byref(si),c.byref(pi)),c.get_last_error()
armed=False;hit=False;breakpoints={};exit_code=125
source=Path('crates/worth-execution/src/backend/meter/checkpoint.rs').resolve()
def write(address,byte):
 buf=c.create_string_buffer(byte);count=c.c_size_t();assert k.WriteProcessMemory(pi.process,P(address),buf,1,c.byref(count)),c.get_last_error();assert k.FlushInstructionCache(pi.process,P(address),1)
try:
 while True:
  ev=Event();assert k.WaitForDebugEvent(c.byref(ev),60000),c.get_last_error();status=0x10002
  if ev.code==1:
   raw=bytes(ev.data);code=struct.unpack_from('<I',raw)[0];address=struct.unpack_from('<Q',raw,16)[0]
   if code==0x80000003 and not armed:
    d.SymSetOptions(0x10|0x200|0x4|0x2);assert d.SymInitializeW(pi.process,str(Path(exe).parent),True),c.get_last_error()
    for wanted in [150,58]:
     line=Line();line.size=c.sizeof(line);delta=w.LONG()
     for filename in [str(source),str(source.relative_to(Path.cwd())),source.name]:
      if d.SymGetLineFromName64(pi.process,None,filename.encode('mbcs'),wanted,c.byref(delta),c.byref(line)):break
     else:
      print('NO_BRANCH_LINE',wanted,flush=True);continue
     assert abs(line.number-wanted)<=1,(wanted,line.number,delta.value)
     if line.address not in breakpoints:
      old=c.create_string_buffer(1);count=c.c_size_t();assert k.ReadProcessMemory(pi.process,P(line.address),old,1,c.byref(count));breakpoints[line.address]=old.raw;write(line.address,b'\xcc')
     print('BREAKPOINT',wanted,hex(line.address),line.number,flush=True)
    assert breakpoints, 'PDB has neither WorkCeiling branch line; no test body executed'
    armed=True
   elif code==0x80000003 and address in breakpoints:
    print('FIRST_WORK_CEILING',hex(address),flush=True);hit=True
    thread=k.OpenThread(0x1fffff,False,ev.tid);context=c.create_string_buffer(1232);struct.pack_into('<I',context,48,0x10000b);assert k.GetThreadContext(thread,context)
    resume=c.create_string_buffer(context.raw);struct.pack_into('<Q',resume,248,address)
    frame=c.create_string_buffer(512)
    for off,reg in [(0,248),(32,160),(48,152)]:struct.pack_into('<Q',frame,off,struct.unpack_from('<Q',context,reg)[0]);struct.pack_into('<I',frame,off+12,3)
    for n in range(80):
     pc=struct.unpack_from('<Q',frame,0)[0];sym=c.create_string_buffer(16384);struct.pack_into('<I',sym,0,88);struct.pack_into('<I',sym,80,16000);disp=c.c_ulonglong();name='?'
     if d.SymFromAddr(pi.process,pc,c.byref(disp),sym):name=sym.raw[84:84+struct.unpack_from('<I',sym,76)[0]].decode(errors='replace')
     line=Line();line.size=c.sizeof(line);ld=w.DWORD();where=''
     if d.SymGetLineFromAddr64(pi.process,pc,c.byref(ld),c.byref(line)):where=f'{line.file.decode(errors="replace")}:{line.number}'
     print('FRAME',n,hex(pc),name,where,flush=True)
     if not d.StackWalk64(0x8664,pi.process,thread,frame,context,None,c.cast(d.SymFunctionTableAccess64,P),c.cast(d.SymGetModuleBase64,P),None):break
    for at,old in breakpoints.items():write(at,old)
    breakpoints.clear();assert k.SetThreadContext(thread,resume);k.CloseHandle(thread)
   elif code!=0x80000003:status=0x80010001
  if ev.code==5:
   exit_code=struct.unpack_from('<I',bytes(ev.data))[0];print('DIAGNOSTIC_EXIT',exit_code,'CHARGE_CAPTURED',hit,flush=True);k.ContinueDebugEvent(ev.pid,ev.tid,status);break
  k.ContinueDebugEvent(ev.pid,ev.tid,status)
except BaseException:
 k.TerminateProcess(pi.process,125);raise
raise SystemExit(exit_code if hit else 125)
