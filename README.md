# db65 - a debugger for cc65
This debugger hosts sim65 cpu engine and emulates the sim65 environment. 

It understands the PV hooks of sim65

It understands the cc65 parameter stack

# features
- load VICE symbol files
- breakpoints
- next / step
- watch points to trap reads or write to memory locations
- smart stack display
- stack mismatch detection
- reading memory not written to
- TODO - cc65 param stack awareness

# building
Its written in rust so you need rust installed to build

You need a c compiler somewhere , cargo cc / build.rs will find it

then just `cargo build`

# use
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
PS C:\work\db65> cl65 -t sim6502 filetest.c -g -Ln filetest.sym
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
