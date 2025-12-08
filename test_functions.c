/* Sample C code */
#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

void print_hello(void) {
    printf("Hello\n");
}

static int helper(void) {
    return 42;
}

int main(void) {
    int result = add(1, 2);
    print_hello();
    return 0;
}
