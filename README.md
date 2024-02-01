# db65 - a debugger for cc65
This debugger hosts sim65 cpu engine and emulates the sim65 environment. 

sim65 is part of the cc65 toolset - https://github.com/cc65/cc65

It reads the linker generated dbgfile data

## features
- load cc65 toolchain debug info
- c source level debug
- local and register variable aware
- assembler source level debug
- 'raw' binary debug
- breakpoints 
- read / write watchpoints
- next / step
- smart stack display
- extensive error detection (see below)

## error detection  

- unbalanced stack (eg assembler push, not pulled before rts)
- write or read outside linker assigned memory
- write or read outside heap allocated memory
- heap leaks
- invalid or double free/realloc calls
- reading from memory not written to

## precompiled binaries

Are in the releases section here https://github.com/pm100/db65/releases

- db65 is linux gclibc version compiled on latest ubuntu
- musl-db65 is MUSL linux binary, use this if you get glibc errors
- mac-db65 is macos version
- db65.exe is windows 64 bit version
  
## building
Its written in rust so you need rust installed to build

You need a c compiler somewhere , cargo cc / build.rs will find it

then just `cargo build`

## compiling c code

db65 needs the output from `ld65 --dbgfile` with the c code being compiled with `-g`

ie

```
cc65 -r -g  -t sim6502 $demo.c -o demo.s
ca65 -g  -t sim6502 demo.s -o demo.o
ld65 --dbgfile demo.dbg  -t sim6502 demo.o -o demo sim6502.lib 
```

it can also be compiled using cl65

```
cl65 -t sim6502 -g --ld-args --dbgfile --ld-args demo.dbg demo.c
```

but note that this will not have the .s file as cl65 deletes it

For maximum visibilty into the runtime you should have the runtime source compile
tree available. Best is to build cc65 and then compile using the binaries
in that build tree. In that case db65 will find the runtime source as well
as the intermediate .s files for runtime components written in c


## demo
simple demo of use
```
PS C:\work\db65> cat demo.c
#include <stdio.h>
int main()
{
    int len;
    char buf[100];
    FILE *foo = fopen("foo.txt", "w");
    len = fprintf(foo, "Hello, World!");
    fclose(foo);

    foo = fopen("foo.txt", "r");
    fgets(buf, 100, foo);
    printf("buf=%s\n", buf);
    fclose(foo);
}


PS C:\work\db65> C:\Users\paulm\Downloads\db65
db65 sim6502 debugger 0.2.1 (16024e9)
use 'help' to get help for commands and 'about' for more information
>> load demo
Loaded 3099 bytes, cpu=6502
Loading debug info from "demo.dbg"
files: 77
modules: 55
segments: 9
symbols: 920
>> b demo.c:10
>> run
bp #1 demo.c:10
demo.c:10                   foo = fopen("foo.txt", "r");
>> p -i len
13
>> ns
demo.c:11                   fgets(buf, 100, foo);
>> ns
demo.c:12                   printf("buf=%s\n", buf);
>> p -s buf
Hello, World!
>> b __printf
>> g
bp #2 __printf
_printf.s:244                   pha                             ; Save low byte of ap
>> bt
0x043b _printf+0xf3:
0x0cd6 vfprintf.s:141                   lda     ccount
0x0b68 printf.s:74                      ldy     ParamSize
0x029e demo.c:13                    fclose(foo);
0x0215 crt0.s:30                _exit:  pha
>> g
buf=Hello, World!
Exit
>>
```


