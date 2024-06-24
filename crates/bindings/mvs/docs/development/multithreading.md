## Is the context thread-safe?

Bocumentation is a little unclear, but in MVS SDK v4.3.0-test.4, Ghidra gives the following disassembly for the `MV_CC_Initialize` function:
```c
void MV_CC_Initialize(void) {
  FUN_18006a1f0((LPCRITICAL_SECTION)&DAT_1800fc100);
  return;
}

undefined8 FUN_18006a1f0(LPCRITICAL_SECTION param_1) {
  char *pcVar1;
  
  EnterCriticalSection(param_1);
    if (*(char *)&param_1[1].DebugInfo == '\0') {
    FUN_18006a5a0((longlong)param_1);
    pcVar1 = FUN_1800290e0();
    FUN_18005fa50(pcVar1);
    FUN_18001be00(pcVar1);
    FUN_18002a8e0();
    if ((DAT_18011cb90 & 1) == 0) {
      DAT_18011cb90 = DAT_18011cb90 | 1;
      FUN_180060590();
      atexit((_func_5014 *)&LAB_1800b0af0);
    }
    if ((DAT_18011cb88 & 1) == 0) {
      DAT_18011cb88 = DAT_18011cb88 | 1;
      FUN_1800343c0();
      atexit((_func_5014 *)&DAT_1800b0a00);
    }
    FUN_1800345e0();
    if ((DAT_18011cf68 & 1) == 0) {
      DAT_18011cf68 = DAT_18011cf68 | 1;
      atexit(FUN_1800b0be0);
    }
    FUN_180047e70();
    FUN_180048690();
    *(undefined *)&param_1[1].DebugInfo = 1;
  }
  LeaveCriticalSection(param_1);
  return 0;
}
```

So looks like the code is guarded by a critical section and this looks good. I didn't delve deep into this code, but looks like it is modifying some global state, so I assume that initializing will onverwrite any previous initialization so - I think - we shall ansure that this function is called only once per process, not per thread.  
Anyway, when creating a context we lock our own mutex, so we sould be good.

## Is the device thread-safe?

I don't know, but we should better assume that it is not, also it should be a good practice not to share device handles between threads in other frameworks so I is better to assume that it is not thread-safe for the sake of consistency and safety.

## Is the device info thread-safe?

Documentation for `MV_CC_EnumDevices` says:
> The memory of storing device list is allocated by the SDK and will be released and applied for during multiple-threaded API calling. It is recommended to avoid the multiple-threaded enumeration. ...

So we use a lock (see `MVSContextInner::enumerate_devices_lock`) to avoid multiple threads enumerating devices. In fact, if two threads enumerate devices at the same time, the second might overwrite the first's result while the first is still reading the result list. The lock is released after the list is read, so we should be good.

Now the question is, can the result elements be shared between threads? I don't see any pointers in the `MV_CC_DEVICE_INFO` struct, so I think it is safe to share the result elements between threads because each copy should be a deep copy of the original. Actually, there is some reserved space in this struct so this might not be entirely true, but I think it is safe to assume that it can be sent between threads.