/* C matrix multiply baseline for comparison with Axon runtime.
 * Compile: gcc -O2 -o matmul_c matmul_c.c -lm
 * Run:     ./matmul_c
 */

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define SIZE 256
#define ITERATIONS 100

static double a[SIZE][SIZE];
static double b[SIZE][SIZE];
static double c[SIZE][SIZE];

static void fill_random(double m[SIZE][SIZE]) {
    for (int i = 0; i < SIZE; i++)
        for (int j = 0; j < SIZE; j++)
            m[i][j] = (double)rand() / RAND_MAX;
}

static void matmul(void) {
    for (int i = 0; i < SIZE; i++)
        for (int j = 0; j < SIZE; j++) {
            double sum = 0.0;
            for (int k = 0; k < SIZE; k++)
                sum += a[i][k] * b[k][j];
            c[i][j] = sum;
        }
}

int main(void) {
    srand(42);
    fill_random(a);
    fill_random(b);

    /* Warmup */
    matmul();

    struct timespec start, end;
    double total = 0.0;

    for (int iter = 0; iter < ITERATIONS; iter++) {
        clock_gettime(CLOCK_MONOTONIC, &start);
        matmul();
        clock_gettime(CLOCK_MONOTONIC, &end);
        total += (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    }

    printf("C matmul %dx%d: %.3f ms (avg over %d iterations)\n",
           SIZE, SIZE, (total / ITERATIONS) * 1000.0, ITERATIONS);

    return 0;
}
