#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv)
{

    char *small_str;
    char *big_str;
    small_str = malloc(10);
    
    // write off the end
    strcpy(small_str, "Hello World!");

    // invalid_free

    free(small_str+1);

    big_str = realloc(small_str, 20);
    strcpy(big_str, "Hello World");
    
    // leak
}