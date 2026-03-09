# Ownership and Borrowing

Axon uses an ownership system inspired by Rust to guarantee memory safety at
compile time — no garbage collector, no dangling pointers, no data races.

---

## The Three Ownership Rules

1. **Every value has exactly one owner.**
2. **When the owner goes out of scope, the value is dropped.**
3. **There can be either one mutable reference OR any number of immutable
   references to a value — never both at the same time.**

---

## Ownership and Move Semantics

By default, assigning a value _moves_ it. The original binding becomes invalid:

```axon
let tensor = randn([1024, 1024]);
let other = tensor;          // tensor is MOVED into other
// println("{}", tensor);    // ERROR[E4001]: use of moved value `tensor`
println("{}", other);        // OK
```

This applies to function calls as well:

```axon
fn consume(t: Tensor<Float32, [3, 3]>) {
    println("{}", t);
}

let data = randn([3, 3]);
consume(data);
// consume(data);   // ERROR[E4001]: use of moved value `data`
```

### Why Moves?

Moves prevent double-free errors and make ownership transfer explicit.
When a large tensor is passed to a function, no implicit copy occurs —
you always know where your data lives.

---

## Borrowing: `&T` and `&mut T`

To use a value without taking ownership, **borrow** it:

### Immutable Borrows (`&T`)

Multiple immutable borrows are allowed simultaneously:

```axon
fn print_shape(t: &Tensor<Float32, [?, 784]>) {
    println("Shape: {}", t.shape);
}

let input = randn([32, 784]);
print_shape(&input);       // borrow, don't move
print_shape(&input);       // still valid — input wasn't moved
```

### Mutable Borrows (`&mut T`)

Only one mutable borrow is allowed at a time, and no immutable borrows may
coexist with it:

```axon
fn scale(t: &mut Tensor<Float32, [3, 3]>, factor: Float32) {
    // modify tensor in place
}

let mut weights = randn([3, 3]);
scale(&mut weights, 2.0);
println("{}", weights);    // OK — mutable borrow has ended
```

### Borrow Conflicts

The compiler rejects overlapping mutable and immutable borrows:

```axon
let mut data = randn([10]);
let r1 = &data;
let r2 = &mut data;       // ERROR[E4003]: cannot borrow `data` as mutable
                           // because it is also borrowed as immutable
println("{}", r1);
```

---

## Lifetimes

Lifetimes ensure that references never outlive the data they point to.
In most cases, the compiler infers lifetimes automatically:

```axon
fn first_element(v: &Vec<Int32>) -> &Int32 {
    &v[0]   // lifetime of return value tied to lifetime of `v`
}
```

When the compiler needs help, you annotate lifetimes explicitly:

```axon
fn longest<'a>(a: &'a String, b: &'a String) -> &'a String {
    if a.len() > b.len() { a } else { b }
}
```

### Dangling Reference Prevention

```axon
fn dangling() -> &String {
    let s = "hello".to_string();
    &s   // ERROR[E4005]: `s` does not live long enough
}        // `s` is dropped here
```

---

## Copy Types vs Move Types

Some small, stack-allocated types implement the `Copy` trait and are
**copied** instead of moved:

| Copy Types                  | Move Types           |
| --------------------------- | -------------------- |
| `Int8` through `Int64`      | `String`             |
| `UInt8` through `UInt64`    | `Vec<T>`             |
| `Float16` through `Float64` | `Tensor<T, S>`       |
| `Bool`, `Char`              | `HashMap<K, V>`      |
| Tuples of Copy types        | Structs (by default) |

```axon
let a: Int32 = 42;
let b = a;          // copy — both a and b are valid
println("{} {}", a, b);

let s = "hello".to_string();
let t = s;          // move — only t is valid
// println("{}", s);   // ERROR
```

### Making Structs Copyable

Derive `Copy` and `Clone` for small value types:

```axon
struct Color: Copy, Clone {
    r: UInt8,
    g: UInt8,
    b: UInt8,
}

let red = Color { r: 255, g: 0, b: 0 };
let also_red = red;   // copy, not move
println("{}", red.r);  // OK
```

---

## Tensor Device-Aware Borrowing

Tensors carry device information (`@cpu` / `@gpu`), and the borrow checker
enforces device-safety rules:

### Rule: No Cross-Device Aliasing

A tensor on the GPU cannot be mutably borrowed while a CPU reference exists:

```axon
let mut t = randn([256, 256]);
let cpu_ref = &t;
let gpu_t = t.to_gpu();      // ERROR[E4007]: cannot move `t` to GPU while
                              // borrowed on CPU
```

### Device Transfer is a Move

Transferring a tensor between devices moves it:

```axon
let cpu_data = randn([1024]);
let gpu_data = cpu_data.to_gpu();    // cpu_data is moved
// println("{}", cpu_data);          // ERROR: use of moved value

let result = gpu_data.to_cpu();      // gpu_data is moved back
println("{}", result);
```

### Safe Pattern: Borrow, Then Transfer

```axon
let mut data = randn([256, 256]);

// Phase 1: work on CPU
let norm = data.mean();
println("Mean: {}", norm);

// Phase 2: transfer to GPU (no outstanding borrows)
let gpu_data = data.to_gpu();
let result = gpu_data @ gpu_data;
```

---

## Ownership in Practice: Training Loop

A real-world example combining ownership patterns:

```axon
struct Trainer {
    model: NeuralNet,
    optimizer: Adam,
}

impl Trainer {
    fn train_epoch(&mut self, data: &DataLoader) -> Float32 {
        let mut total_loss = 0.0;

        for batch in data {
            let (inputs, targets) = batch;

            // model borrowed mutably through self
            let predictions = self.model.forward(inputs);
            let loss = cross_entropy(predictions, targets);

            total_loss += loss.item();

            loss.backward();
            self.optimizer.step();
            self.optimizer.zero_grad();
        }

        total_loss / data.len() as Float32
    }
}
```

Key ownership points:

- `&mut self` — the trainer exclusively owns the model during training
- `data: &DataLoader` — data is borrowed immutably (read-only)
- `loss.backward()` consumes gradient information (move semantics on graph nodes)
- No data races are possible — the type system guarantees it

---

## Summary

| Concept       | Rule                                            |
| ------------- | ----------------------------------------------- |
| Ownership     | Each value has exactly one owner                |
| Move          | Assignment transfers ownership (non-Copy types) |
| Copy          | Small primitives are implicitly copied          |
| `&T`          | Immutable borrow — multiple allowed             |
| `&mut T`      | Mutable borrow — exclusive access               |
| Lifetimes     | References cannot outlive their referent        |
| Device safety | Cross-device aliasing is forbidden              |

---

## See Also

- [Language Tour](language-tour.md) — overview of Axon syntax
- [Tensor Programming](tensors.md) — tensor-specific ownership rules
- [GPU Programming](gpu-programming.md) — device transfer patterns
- [Compiler Errors: E4xxx](../reference/compiler-errors.md) — borrow checker error codes
