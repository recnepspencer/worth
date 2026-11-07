"""Native debugger plumbing; no target/product compilation or disk patching."""
import ctypes as c
import ctypes.wintypes as w
import struct
k=c.WinDLL('kernel32',use_last_error=True);d=c.WinDLL('dbghelp',use_last_error=True)
H=c.c_void_p;Q=c.c_ulonglong;P=c.c_void_p
class SI(c.Structure):
 _fields_=[('cb',w.DWORD),('reserved',w.LPWSTR),('desktop',w.LPWSTR),('title',w.LPWSTR),('x',w.DWORD),('y',w.DWORD),('xs',w.DWORD),('ys',w.DWORD),('xc',w.DWORD),('yc',w.DWORD),('fill',w.DWORD),('flags',w.DWORD),('show',w.WORD),('reserved2',w.WORD),('r2',P),('stdin',H),('stdout',H),('stderr',H)]
class PI(c.Structure):_fields_=[('process',H),('thread',H),('pid',w.DWORD),('tid',w.DWORD)]
class Event(c.Structure):_fields_=[('code',w.DWORD),('pid',w.DWORD),('tid',w.DWORD),('padding',w.DWORD),('data',c.c_byte*160)]
class MBI(c.Structure):
 _fields_=[('base',P),('allocation_base',P),('allocation_protect',w.DWORD),('partition',w.WORD),('size',c.c_size_t),('state',w.DWORD),('protect',w.DWORD),('type',w.DWORD)]
def declare(lib,name,args,result=w.BOOL):
 f=getattr(lib,name);f.argtypes=args;f.restype=result;return f
for name,args,ret in [
 ('CreateProcessW',[w.LPCWSTR,w.LPWSTR,P,P,w.BOOL,w.DWORD,P,w.LPCWSTR,c.POINTER(SI),c.POINTER(PI)],w.BOOL),
 ('CreateRemoteThread',[H,P,c.c_size_t,P,P,w.DWORD,c.POINTER(w.DWORD)],H),
 ('GetThreadContext',[H,P],w.BOOL),('SetThreadContext',[H,P],w.BOOL),
 ('ReadProcessMemory',[H,P,P,c.c_size_t,c.POINTER(c.c_size_t)],w.BOOL),
 ('WriteProcessMemory',[H,P,P,c.c_size_t,c.POINTER(c.c_size_t)],w.BOOL),
 ('VirtualAllocEx',[H,P,c.c_size_t,w.DWORD,w.DWORD],P),
 ('VirtualFreeEx',[H,P,c.c_size_t,w.DWORD],w.BOOL),
 ('VirtualProtectEx',[H,P,c.c_size_t,w.DWORD,c.POINTER(w.DWORD)],w.BOOL),
 ('VirtualQueryEx',[H,P,c.POINTER(MBI),c.c_size_t],c.c_size_t),
 ('FlushInstructionCache',[H,P,c.c_size_t],w.BOOL),
 ('SuspendThread',[H],w.DWORD),('ResumeThread',[H],w.DWORD),
 ('WaitForDebugEvent',[c.POINTER(Event),w.DWORD],w.BOOL),
 ('ContinueDebugEvent',[w.DWORD,w.DWORD,w.DWORD],w.BOOL),
 ('TerminateProcess',[H,w.UINT],w.BOOL),('CloseHandle',[H],w.BOOL),
 ('GetStdHandle',[w.DWORD],H)]:declare(k,name,args,ret)
for name,args,ret in [
 ('SymInitializeW',[H,w.LPCWSTR,w.BOOL],w.BOOL),('SymCleanup',[H],w.BOOL),
 ('SymSetOptions',[w.DWORD],w.DWORD),('SymRefreshModuleList',[H],w.BOOL),
 ('SymFromAddr',[H,Q,c.POINTER(Q),P],w.BOOL),
 ('SymFunctionTableAccess64',[H,Q],P),('SymGetModuleBase64',[H,Q],Q),
 ('StackWalk64',[w.DWORD,H,H,P,P,P,P,P,P],w.BOOL)]:declare(d,name,args,ret)
def checked(value):
 if not value:raise c.WinError(c.get_last_error())
 return value
def read(process,address,size):
 b=c.create_string_buffer(size);n=c.c_size_t();checked(k.ReadProcessMemory(process,address,b,size,c.byref(n)));assert n.value==size;return b.raw
def write(process,address,data):
 n=c.c_size_t();checked(k.WriteProcessMemory(process,address,data,len(data),c.byref(n)));assert n.value==len(data)
def context(thread):
 b=c.create_string_buffer(1232);struct.pack_into('<I',b,48,0x10001f);checked(k.GetThreadContext(thread,b));return b
REG={'rax':120,'rcx':128,'rdx':136,'rsp':152,'rbp':160,'rip':248,'dr0':72,'dr1':80,'dr2':88,'dr3':96,'dr6':104,'dr7':112}
def get(ctx,reg):return struct.unpack_from('<Q',ctx,REG[reg])[0]
def setreg(ctx,reg,value):struct.pack_into('<Q',ctx,REG[reg],value)
def resume_flag(ctx):struct.pack_into('<I',ctx,68,struct.unpack_from('<I',ctx,68)[0]|0x10000)
def bp(process,address,byte):
 old=w.DWORD();checked(k.VirtualProtectEx(process,address,len(byte),0x40,c.byref(old)));write(process,address,byte);checked(k.VirtualProtectEx(process,address,len(byte),old.value,c.byref(w.DWORD())));checked(k.FlushInstructionCache(process,address,len(byte)))
def query(process,address):
 m=MBI();checked(k.VirtualQueryEx(process,address,c.byref(m),c.sizeof(m)));return m
