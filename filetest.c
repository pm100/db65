#include <stdio.h>
int main()
{
    int len;
    char buf[100];
    // FILE *foo = fopen("foo.txt", "w");
    // len = fprintf(foo, "Hello, World!");
    // printf("fd=%d len=%d\n", fileno(foo), len);
    // fclose(foo);

    // foo = fopen("foo.txt", "r");
    // printf("fd=%d len=%d\n", fileno(foo), len);
    // fgets(buf, 100, foo);
    // printf("buf=%s\n", buf);
    // fclose(foo);
    // fprintf(stderr, "hello stderr\n");
    scanf("%s", buf);
    printf("buf=%s\n", buf);
}