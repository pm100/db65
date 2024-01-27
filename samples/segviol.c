#include <stdio.h>
#include <stdlib.h>
#include <string.h>
char buff[10];
int main(int argc, char **argv)
{
      int i ;
      int *p;
    // read random location
    i = *(int *)0xdead;
    // write to random location
    *(int *)0xbeef = 0;
  
    p = malloc(20);

    // read unitialized data
    i = p[4];

    strcpy(buff, "Hello");
    printf("buff=%s\n", buff);

}