# db65 - a debugger for cc65
This debugger hosts sim65 cpu engine and emulates the sim65 environment. 

It understands the PV hooks of sim65

It understands the cc65 parameter stack

## features
- load VICE symbol files
- breakpoints
- next / step
- watch points to trap reads or write to memory locations
- smart stack display
- stack mismatch detection
- reading memory not written to
- TODO - cc65 param stack awareness
- arbitray address expression
  
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
## commands

The command line has full searchable command history.
 -  `ctrl-r` searches backwards
 - up arrow scrolls to previous
 - etc...

pressing enter will repeat the last line

### load_code (load)

loads a binary file that was output by ld65 for sim65. It expects to see the sim65 headers

`>> load <file name>`

### symbols (ll)

loads symbol file output by ld65 for VICE. Generated by ld65 switches `-g -Ln <file>`


`>> ll <file name>`

you dont need to load symbols but its much easier with symbols

### list symbols (ls)

List symbols, option matching a substring

`>> ls [str]`

list all symbols containg the substring 'main' - `ls main`


### run

runs the code starting at the address found in the binary file header

### break (b)

Sets a breakpoint

`>> b <addr>`

Examples

`>> b _main`

`>> b $27ba`

### bl

List break points

### bd

delete break points

`>> bd`

deletes all break points

`>> bd 1`

deletes break point #1 (from bl output)

### go

resumes execution after break point

### next 

is 'step over' if it sees a function call it will not break until it returns

### step

one instruction

### fin

run until return from current function call. 

### watch (w)

watch for memory read or write. Will break on the read or write of a location

`>> w -r -w <addr>`

example

`>> w -w sreg`

will break when a write it made to .sreg

wl and wd are the same as bl and bd

### memory (m)

dumps memory

`>> m $1467`

### dis 

Disassembles

`>> dis [addr]`

if no address is given it continues from the last address

### enable (en)

Enables bug traps.

stack check verifies that the stack is correct when a function returns. break if not

memory check will break if a read is made from a memory location that has not been written to

These are both turned off by default

### print (p)

prints values from memory

`>> p -s -i -p <addr>`

Print either an integer (16 bit), a string or a pointer

### back_trace (bt) 

Displays the current 6502 stack

### reg

set register value

`>> reg xr 42`

### write_memory (wm)

Write one byte to memory

`>> wm 0x1234 0x44`

### expr

Expression evaluator. Anywhere that an address can be given you can have an expression

Registers are available they are called .ac, .pc, .xr, .xy., .sr, .sp

The expr command evaluates an expression, this can be useful for inspecting things or just checking expression syntax

An expression starts with '='. You can then have numbers, symbols, parantheses, registers, plus dereference operations

 - `=.xr` returns the x register
 - `=ptr1+0x20` returns that symbol plus 0x20
 - `=@(ptr1)` returns what ptr1 points at
 - `=@ @(ptr1)` returns what the ptr1 at ptr1 points to
 - `=@(ptr1+.xr)` returns what ptr1 + xr points to

`@` is the dereference operation for a word

`@b` is the deref of a byte


 Useful quickies

 `dis =.pc`

 `expr =.xr`

 Note that you may need to enclose the whole expression in quotes so that it doesnt look like command arguments to the line parser

expression evaluator uses the excellent evalexpr crate https://github.com/ISibboI/evalexpr.
The docs https://docs.rs/evalexpr/latest/evalexpr/  show all the operators and functions that can be used.
Of note

- bitand, bitor....
- if
- shl, shr


### quit

