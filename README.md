# db65 - a debugger for cc65
This debugger hosts sim65 cpu engine and emulates the sim65 environment. 

It eads the linker generated dbgfile data

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

## demo
simple demo of use
```
PS C:\work\db65> cat filetest.c
#include <stdio.h>
int main()
{
    int len;
    char buf[100];
    FILE *foo = fopen("foo.txt", "w");
    len = fprintf(foo, "Hello, World!");
    printf("fd=%d len=%d\n", fileno(foo), len);
    fclose(foo);

    foo = fopen("foo.txt", "r");
    printf("fd=%d len=%d\n", fileno(foo), len);
    fgets(buf, 100, foo);
    printf("hello: ");
    fclose(foo);
    fprintf(stderr, "hello stderr\n");
    scanf("%s", buf);
    printf("buf=%s\n", buf);
}
PS C:\work\db65> 
filetest.c:19: Warning: Control reaches end of non-void function [-Wreturn-type]
PS C:\work\db65> C:\Users\paulm\Downloads\db65
>> load filetest
>> ll filetest.sym
>> ls main
0x0f27 .callmain
0x0229 ._main
>> break ._main
>> run
bp #1 ._main
0229:       ldy   #$66      A=00 X=00 Y=04 SP=fd SR=no-BdiCz
>> dis
.__CONSTRUCTOR_TABLE__:
0229:       ldy   #$66
022b:       jsr   .subysp
022e:       lda   #$A3
0230:       ldx   #$17
0232:       jsr   .pushax
0235:       lda   #$B6
0237:       ldx   #$17
0239:       jsr   ._fopen
023c:       jsr   .pushax
023f:       ldy   #$01
>>
0241:       jsr   .ldaxysp
0244:       jsr   .pushax
0247:       lda   #$7F
0249:       ldx   #$17
024b:       jsr   .pushax
024e:       ldy   #$04
0250:       jsr   ._fprintf
0253:       ldy   #$66
0255:       jsr   .staxysp
0258:       lda   #$8D
>> g
fd=3 len=13
fd=4 len=0
hello: hello stderr
f
buf=f
exit
>>

```
## commands
