# Phase 5: Standard Library — Implementation Specification

## 1. Overview

Phase 5 delivers the **Axon standard library** (`std`): a collection of foundational modules written in Axon (with some `unsafe` runtime intrinsics) that provide data structures, I/O, math, tensor operations, concurrency, and data loading. The stdlib is the bridge between the compiler and user-level code.

### Deliverables

| Module             | Path                  | Purpose                                            |
| ------------------ | --------------------- | -------------------------------------------------- |
| `std::prelude`     | `stdlib/prelude.axon` | Auto-imported core types and traits                |
| `std::io`          | `stdlib/io.axon`      | File I/O, stdin/stdout, buffered readers           |
| `std::fmt`         | `stdlib/fmt.axon`     | String formatting and Display/Debug traits         |
| `std::collections` | `stdlib/collections/` | Vec, HashMap, HashSet, BTreeMap, Queue, Stack      |
| `std::math`        | `stdlib/math.axon`    | Numeric math functions, constants                  |
| `std::tensor`      | `stdlib/tensor/`      | Tensor operations, creation, slicing, broadcasting |
| `std::string`      | `stdlib/string.axon`  | String manipulation, UTF-8 utilities               |
| `std::iter`        | `stdlib/iter.axon`    | Iterator trait and adapters                        |
| `std::option`      | `stdlib/option.axon`  | Option<T> type and methods                         |
| `std::result`      | `stdlib/result.axon`  | Result<T, E> type and methods                      |
| `std::sync`        | `stdlib/sync/`        | Mutex, RwLock, Channel, Arc                        |
| `std::thread`      | `stdlib/thread.axon`  | Thread spawning, joining                           |
| `std::data`        | `stdlib/data/`        | CSV, JSON loaders, data pipeline                   |
| `std::random`      | `stdlib/random.axon`  | RNG, distributions, seeding                        |
| `std::time`        | `stdlib/time.axon`    | Duration, Instant, timing utilities                |
| `std::mem`         | `stdlib/mem.axon`     | Memory utilities, size_of, align_of, swap          |
| `std::convert`     | `stdlib/convert.axon` | From/Into/TryFrom/TryInto traits                   |
| `std::ops`         | `stdlib/ops.axon`     | Operator overloading traits (Add, Sub, Mul, etc.)  |
| `std::device`      | `stdlib/device.axon`  | Device abstraction (CPU, GPU, TPU)                 |

### Dependencies

- Phase 3: Type checker (all stdlib code must type-check)
- Phase 4: Codegen (stdlib compiles to native code)

---

## 2. Core Traits (`std::prelude`, `std::ops`, `std::convert`)

### 2.1 Prelude (auto-imported)

```axon
// Types always available without `use`:
// Option, Some, None, Result, Ok, Err
// Vec, String, HashMap, Tensor
// Bool, Int32, Int64, Float32, Float64, etc.

// Traits always available:
// Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord
// Iterator, Into, From, Default, Drop
```

### 2.2 Operator Traits

```axon
trait Add<Rhs = Self> {
    type Output;
    fn add(&self, rhs: &Rhs) -> Self::Output;
}

trait Sub<Rhs = Self> { type Output; fn sub(&self, rhs: &Rhs) -> Self::Output; }
trait Mul<Rhs = Self> { type Output; fn mul(&self, rhs: &Rhs) -> Self::Output; }
trait Div<Rhs = Self> { type Output; fn div(&self, rhs: &Rhs) -> Self::Output; }
trait Mod<Rhs = Self> { type Output; fn modulo(&self, rhs: &Rhs) -> Self::Output; }
trait Neg           { type Output; fn neg(&self) -> Self::Output; }
trait Not           { type Output; fn not(&self) -> Self::Output; }

// Tensor-specific
trait MatMul<Rhs = Self> {
    type Output;
    fn matmul(&self, rhs: &Rhs) -> Self::Output;  // @ operator
}

// Indexing
trait Index<Idx> {
    type Output;
    fn index(&self, idx: Idx) -> &Self::Output;
}
trait IndexMut<Idx> {
    type Output;
    fn index_mut(&mut self, idx: Idx) -> &mut Self::Output;
}
```

### 2.3 Conversion Traits

```axon
trait From<T> { fn from(value: T) -> Self; }
trait Into<T> { fn into(self) -> T; }
trait TryFrom<T> { type Error; fn try_from(value: T) -> Result<Self, Self::Error>; }
trait TryInto<T> { type Error; fn try_into(self) -> Result<T, Self::Error>; }
```

### 2.4 Display & Debug

```axon
trait Display { fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError>; }
trait Debug   { fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError>; }
```

### 2.5 Clone & Copy

```axon
trait Clone { fn clone(&self) -> Self; }
trait Copy: Clone { }  // Marker trait — compiler-implemented for primitives
```

### 2.6 Iterator

```axon
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;

    // Provided methods (default impls):
    fn map<B>(self, f: fn(Self::Item) -> B) -> Map<Self, B>;
    fn filter(self, f: fn(&Self::Item) -> Bool) -> Filter<Self>;
    fn fold<B>(self, init: B, f: fn(B, Self::Item) -> B) -> B;
    fn collect<C: FromIterator<Self::Item>>(self) -> C;
    fn enumerate(self) -> Enumerate<Self>;
    fn zip<Other: Iterator>(self, other: Other) -> Zip<Self, Other>;
    fn take(self, n: Int64) -> Take<Self>;
    fn skip(self, n: Int64) -> Skip<Self>;
    fn sum(self) -> Self::Item;  // where Self::Item: Add
    fn count(self) -> Int64;
    fn any(self, f: fn(&Self::Item) -> Bool) -> Bool;
    fn all(self, f: fn(&Self::Item) -> Bool) -> Bool;
    fn for_each(self, f: fn(Self::Item));
}
```

---

## 3. Collections (`std::collections`)

### 3.1 Vec<T>

```axon
struct Vec<T> { /* internal: ptr, len, cap */ }

impl<T> Vec<T> {
    fn new() -> Vec<T>;
    fn with_capacity(cap: Int64) -> Vec<T>;
    fn push(&mut self, value: T);
    fn pop(&mut self) -> Option<T>;
    fn len(&self) -> Int64;
    fn is_empty(&self) -> Bool;
    fn get(&self, index: Int64) -> Option<&T>;
    fn iter(&self) -> VecIter<T>;
    fn iter_mut(&mut self) -> VecIterMut<T>;
    fn sort(&mut self);           // where T: Ord
    fn contains(&self, value: &T) -> Bool;  // where T: PartialEq
    fn extend(&mut self, iter: impl Iterator<Item = T>);
    fn clear(&mut self);
    fn remove(&mut self, index: Int64) -> T;
    fn insert(&mut self, index: Int64, value: T);
    fn as_slice(&self) -> &[T];
}
```

### 3.2 HashMap<K, V>

```axon
struct HashMap<K, V> { /* internal: Robin Hood hashing */ }

impl<K: Hash + Eq, V> HashMap<K, V> {
    fn new() -> HashMap<K, V>;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn contains_key(&self, key: &K) -> Bool;
    fn len(&self) -> Int64;
    fn keys(&self) -> KeysIter<K, V>;
    fn values(&self) -> ValuesIter<K, V>;
    fn iter(&self) -> HashMapIter<K, V>;
}
```

### 3.3 HashSet<T>

```axon
struct HashSet<T> { /* wrapper around HashMap<T, ()> */ }

impl<T: Hash + Eq> HashSet<T> {
    fn new() -> HashSet<T>;
    fn insert(&mut self, value: T) -> Bool;
    fn contains(&self, value: &T) -> Bool;
    fn remove(&mut self, value: &T) -> Bool;
    fn len(&self) -> Int64;
    fn union(&self, other: &HashSet<T>) -> HashSet<T>;
    fn intersection(&self, other: &HashSet<T>) -> HashSet<T>;
    fn difference(&self, other: &HashSet<T>) -> HashSet<T>;
}
```

---

## 4. Tensor Operations (`std::tensor`)

### 4.1 Tensor Core

```axon
struct Tensor<DType, Shape> {
    // Internal: data pointer, shape array, strides, device tag
}

impl<DType: Numeric, Shape> Tensor<DType, Shape> {
    // Creation
    fn zeros(shape: Shape) -> Tensor<DType, Shape>;
    fn ones(shape: Shape) -> Tensor<DType, Shape>;
    fn full(shape: Shape, value: DType) -> Tensor<DType, Shape>;
    fn rand(shape: Shape) -> Tensor<DType, Shape>;
    fn randn(shape: Shape) -> Tensor<DType, Shape>;   // Normal distribution
    fn from_vec(data: Vec<DType>, shape: Shape) -> Tensor<DType, Shape>;
    fn arange(start: DType, end: DType, step: DType) -> Tensor<DType, [?]>;
    fn linspace(start: DType, end: DType, count: Int64) -> Tensor<DType, [?]>;
    fn eye(n: Int64) -> Tensor<DType, [?, ?]>;         // Identity matrix

    // Shape operations
    fn shape(&self) -> Vec<Int64>;
    fn ndim(&self) -> Int64;
    fn numel(&self) -> Int64;
    fn reshape(&self, new_shape: NewShape) -> Tensor<DType, NewShape>;
    fn transpose(&self) -> Tensor<DType, TransposedShape>;
    fn permute(&self, dims: Vec<Int64>) -> Tensor<DType, [?]>;
    fn squeeze(&self, dim: Int64) -> Tensor<DType, [?]>;
    fn unsqueeze(&self, dim: Int64) -> Tensor<DType, [?]>;
    fn flatten(&self) -> Tensor<DType, [?]>;
    fn view(&self, shape: Vec<Int64>) -> Tensor<DType, [?]>;

    // Reduction operations
    fn sum(&self) -> DType;
    fn sum_dim(&self, dim: Int64) -> Tensor<DType, [?]>;
    fn mean(&self) -> DType;
    fn mean_dim(&self, dim: Int64) -> Tensor<DType, [?]>;
    fn max(&self) -> DType;
    fn min(&self) -> DType;
    fn argmax(&self, dim: Int64) -> Tensor<Int64, [?]>;
    fn argmin(&self, dim: Int64) -> Tensor<Int64, [?]>;

    // Element-wise math
    fn abs(&self) -> Tensor<DType, Shape>;
    fn sqrt(&self) -> Tensor<DType, Shape>;
    fn exp(&self) -> Tensor<DType, Shape>;
    fn log(&self) -> Tensor<DType, Shape>;
    fn pow(&self, exp: DType) -> Tensor<DType, Shape>;
    fn clamp(&self, min: DType, max: DType) -> Tensor<DType, Shape>;
    fn relu(&self) -> Tensor<DType, Shape>;
    fn sigmoid(&self) -> Tensor<DType, Shape>;
    fn tanh(&self) -> Tensor<DType, Shape>;

    // Device operations
    fn to_cpu(&self) -> Tensor<DType, Shape>;
    fn to_gpu(&self) -> Tensor<DType, Shape>;
    fn device(&self) -> Device;

    // Comparison
    fn eq(&self, other: &Tensor<DType, Shape>) -> Tensor<Bool, Shape>;
    fn lt(&self, other: &Tensor<DType, Shape>) -> Tensor<Bool, Shape>;
    fn gt(&self, other: &Tensor<DType, Shape>) -> Tensor<Bool, Shape>;

    // Data access
    fn item(&self) -> DType;  // Only for scalar tensors
    fn to_vec(&self) -> Vec<DType>;
    fn contiguous(&self) -> Tensor<DType, Shape>;
    fn is_contiguous(&self) -> Bool;
}
```

### 4.2 Linear Algebra

```axon
mod std::tensor::linalg {
    fn matmul<D, M, K, N>(a: &Tensor<D, [M,K]>, b: &Tensor<D, [K,N]>) -> Tensor<D, [M,N]>;
    fn dot<D, N>(a: &Tensor<D, [N]>, b: &Tensor<D, [N]>) -> D;
    fn outer<D, M, N>(a: &Tensor<D, [M]>, b: &Tensor<D, [N]>) -> Tensor<D, [M,N]>;
    fn norm<D, S>(a: &Tensor<D, S>, p: Float64) -> D;
    fn det<D, N>(a: &Tensor<D, [N,N]>) -> D;
    fn inv<D, N>(a: &Tensor<D, [N,N]>) -> Tensor<D, [N,N]>;
    fn svd<D, M, N>(a: &Tensor<D, [M,N]>) -> (Tensor<D, [M,M]>, Tensor<D, [?]>, Tensor<D, [N,N]>);
    fn eig<D, N>(a: &Tensor<D, [N,N]>) -> (Tensor<D, [N]>, Tensor<D, [N,N]>);
    fn solve<D, N, K>(a: &Tensor<D, [N,N]>, b: &Tensor<D, [N,K]>) -> Tensor<D, [N,K]>;
    fn cholesky<D, N>(a: &Tensor<D, [N,N]>) -> Tensor<D, [N,N]>;
    fn qr<D, M, N>(a: &Tensor<D, [M,N]>) -> (Tensor<D, [M,M]>, Tensor<D, [M,N]>);
}
```

---

## 5. Math (`std::math`)

```axon
mod std::math {
    // Constants
    let PI: Float64 = 3.14159265358979323846;
    let E: Float64  = 2.71828182845904523536;
    let TAU: Float64 = 6.28318530717958647692;
    let INF: Float64;
    let NEG_INF: Float64;
    let NAN: Float64;

    // Trigonometric
    fn sin(x: Float64) -> Float64;
    fn cos(x: Float64) -> Float64;
    fn tan(x: Float64) -> Float64;
    fn asin(x: Float64) -> Float64;
    fn acos(x: Float64) -> Float64;
    fn atan(x: Float64) -> Float64;
    fn atan2(y: Float64, x: Float64) -> Float64;

    // Exponential / logarithmic
    fn exp(x: Float64) -> Float64;
    fn log(x: Float64) -> Float64;     // Natural log
    fn log2(x: Float64) -> Float64;
    fn log10(x: Float64) -> Float64;
    fn pow(base: Float64, exp: Float64) -> Float64;
    fn sqrt(x: Float64) -> Float64;
    fn cbrt(x: Float64) -> Float64;

    // Rounding
    fn floor(x: Float64) -> Float64;
    fn ceil(x: Float64) -> Float64;
    fn round(x: Float64) -> Float64;
    fn trunc(x: Float64) -> Float64;

    // Min/Max/Clamp
    fn min<T: PartialOrd>(a: T, b: T) -> T;
    fn max<T: PartialOrd>(a: T, b: T) -> T;
    fn clamp<T: PartialOrd>(val: T, lo: T, hi: T) -> T;
    fn abs<T: Neg + PartialOrd>(x: T) -> T;
}
```

---

## 6. I/O (`std::io`)

```axon
trait Read {
    fn read(&mut self, buf: &mut Vec<UInt8>) -> Result<Int64, IoError>;
    fn read_to_string(&mut self) -> Result<String, IoError>;
}

trait Write {
    fn write(&mut self, buf: &[UInt8]) -> Result<Int64, IoError>;
    fn write_string(&mut self, s: &String) -> Result<Int64, IoError>;
    fn flush(&mut self) -> Result<(), IoError>;
}

struct File { /* internal */ }

impl File {
    fn open(path: &String) -> Result<File, IoError>;
    fn create(path: &String) -> Result<File, IoError>;
}

impl Read for File { ... }
impl Write for File { ... }

fn println(s: &String);
fn print(s: &String);
fn eprintln(s: &String);
fn stdin() -> StdinReader;
fn stdout() -> StdoutWriter;
fn stderr() -> StderrWriter;
```

---

## 7. Concurrency (`std::sync`, `std::thread`)

```axon
// Threading
fn spawn(f: fn() -> T) -> JoinHandle<T>;
struct JoinHandle<T> { fn join(self) -> Result<T, ThreadError>; }

// Synchronization
struct Mutex<T> {
    fn new(value: T) -> Mutex<T>;
    fn lock(&self) -> MutexGuard<T>;
    fn try_lock(&self) -> Option<MutexGuard<T>>;
}

struct RwLock<T> {
    fn new(value: T) -> RwLock<T>;
    fn read(&self) -> ReadGuard<T>;
    fn write(&self) -> WriteGuard<T>;
}

struct Channel<T> {
    fn new() -> (Sender<T>, Receiver<T>);
    fn bounded(cap: Int64) -> (Sender<T>, Receiver<T>);
}

struct Arc<T> {
    fn new(value: T) -> Arc<T>;
    fn clone(&self) -> Arc<T>;
}
```

---

## 8. Data Loading (`std::data`)

```axon
mod std::data {
    // CSV
    fn read_csv(path: &String) -> Result<DataFrame, DataError>;

    // JSON
    fn read_json(path: &String) -> Result<JsonValue, DataError>;
    fn parse_json(s: &String) -> Result<JsonValue, DataError>;

    // Data pipeline for ML
    struct DataLoader<T> {
        fn new(dataset: &Dataset<T>, batch_size: Int64, shuffle: Bool) -> DataLoader<T>;
        fn iter(&self) -> DataLoaderIter<T>;
    }

    trait Dataset<T> {
        fn len(&self) -> Int64;
        fn get(&self, index: Int64) -> T;
    }

    struct DataFrame {
        fn column(&self, name: &String) -> Result<Series, DataError>;
        fn shape(&self) -> (Int64, Int64);
        fn head(&self, n: Int64) -> DataFrame;
        fn to_tensor<D: Numeric>(&self) -> Result<Tensor<D, [?, ?]>, DataError>;
    }
}
```

---

## 9. Device Abstraction (`std::device`)

```axon
enum Device {
    Cpu,
    Gpu(Int32),     // GPU index
    Tpu(Int32),     // TPU index
}

impl Device {
    fn is_available(device: Device) -> Bool;
    fn count() -> Int32;           // Number of GPUs/accelerators
    fn current() -> Device;
    fn set_default(device: Device);
    fn synchronize();
    fn memory_allocated() -> Int64;
    fn memory_reserved() -> Int64;
}
```

---

## 10. Task Breakdown

### Phase 5a: Core Traits & Prelude

- [ ] T138 Define operator traits (Add, Sub, Mul, Div, MatMul, Index) — `stdlib/ops.axon`
- [ ] T139 Define conversion traits (From, Into, TryFrom, TryInto) — `stdlib/convert.axon`
- [ ] T140 Define Display, Debug, Clone, Copy, Default, Drop — `stdlib/prelude.axon`
- [ ] T141 Define Iterator trait with default methods — `stdlib/iter.axon`
- [ ] T142 Implement operator traits for all primitives — `stdlib/ops.axon`

### Phase 5b: Collections

- [ ] T143 Implement Vec<T> — `stdlib/collections/vec.axon`
- [ ] T144 Implement HashMap<K, V> — `stdlib/collections/hashmap.axon`
- [ ] T145 Implement HashSet<T> — `stdlib/collections/hashset.axon`
- [ ] T146 Implement Option<T> and Result<T, E> with methods — `stdlib/option.axon`, `stdlib/result.axon`
- [ ] T147 Implement String with UTF-8 support — `stdlib/string.axon`

### Phase 5c: Tensor Operations

- [ ] T148 Implement Tensor creation functions (zeros, ones, rand, etc.) — `stdlib/tensor/create.axon`
- [ ] T149 Implement shape operations (reshape, transpose, permute) — `stdlib/tensor/shape.axon`
- [ ] T150 Implement reduction operations (sum, mean, max, argmax) — `stdlib/tensor/reduce.axon`
- [ ] T151 Implement element-wise math (abs, sqrt, exp, log, relu) — `stdlib/tensor/math.axon`
- [ ] T152 Implement linear algebra (matmul, dot, inv, svd) — `stdlib/tensor/linalg.axon`
- [ ] T153 Implement device transfer (to_cpu, to_gpu) — `stdlib/tensor/device.axon`

### Phase 5d: Math & I/O

- [ ] T154 Implement std::math functions — `stdlib/math.axon`
- [ ] T155 Implement Read/Write traits and File — `stdlib/io.axon`
- [ ] T156 Implement println/print/eprintln — `stdlib/io.axon`
- [ ] T157 Implement string formatting (Display/Debug) — `stdlib/fmt.axon`

### Phase 5e: Concurrency & Data

- [ ] T158 Implement Mutex, RwLock, Arc — `stdlib/sync/`
- [ ] T159 Implement Channel (unbounded + bounded) — `stdlib/sync/channel.axon`
- [ ] T160 Implement thread::spawn and JoinHandle — `stdlib/thread.axon`
- [ ] T161 Implement CSV/JSON loading — `stdlib/data/`
- [ ] T162 Implement DataLoader for ML pipelines — `stdlib/data/loader.axon`
- [ ] T163 Implement Device abstraction — `stdlib/device.axon`
- [ ] T164 Implement random number generation — `stdlib/random.axon`

### Phase 5f: Testing

- [ ] T165 Unit tests for all collection types — `tests/stdlib_tests.rs`
- [ ] T166 Unit tests for tensor operations — `tests/tensor_tests.rs`
- [ ] T167 Unit tests for math functions — `tests/math_tests.rs`
- [ ] T168 Integration tests: complete programs using stdlib — `tests/stdlib_integration.rs`
- [ ] T169 Benchmark: Vec, HashMap, Tensor ops vs. known baselines — `benches/`

---

## 11. Acceptance Criteria

- [ ] All prelude types and traits are auto-imported without `use`
- [ ] Vec, HashMap, HashSet pass correctness tests
- [ ] Tensor operations match NumPy output for reference inputs
- [ ] `@` operator works end-to-end (parsing → type check → codegen → execution)
- [ ] File I/O can read/write text and binary files
- [ ] Thread spawning and Mutex work for basic concurrent programs
- [ ] DataLoader can iterate over batches for ML training loops
- [ ] All stdlib code type-checks and compiles through Phase 3+4
