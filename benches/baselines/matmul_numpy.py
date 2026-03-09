"""NumPy matrix multiply baseline for comparison with Axon runtime."""

import time
import numpy as np


def matmul_benchmark(size: int = 256, iterations: int = 100) -> float:
    """Run matrix multiply benchmark and return average time in seconds."""
    a = np.random.rand(size, size).astype(np.float64)
    b = np.random.rand(size, size).astype(np.float64)

    # Warmup
    _ = a @ b

    total = 0.0
    for _ in range(iterations):
        start = time.perf_counter()
        _ = a @ b
        total += time.perf_counter() - start

    return total / iterations


if __name__ == "__main__":
    for size in [64, 128, 256, 512]:
        avg = matmul_benchmark(size=size, iterations=50)
        print(f"NumPy matmul {size}x{size}: {avg * 1000:.3f} ms")
