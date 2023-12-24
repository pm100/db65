#include <stdio.h>
int main(){
    FILE * foo = fopen("foo.txt", "w");
    fprintf(foo, "Hello, World!");
}