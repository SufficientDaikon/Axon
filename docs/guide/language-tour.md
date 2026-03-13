# Language Tour

A quick tour of Axon's syntax and core features. This guide assumes familiarity
with at least one systems or ML language (Rust, Python, C++).

---

## Variables

Variables are declared with `val`. They are **immutable by default**.

```axon
val x = 42;              // immutable, type inferred as Int32
val y: Float64 = 3.14;   // explicit type annotation
var counter = 0;          // mutable variable
counter += 1;
```

Type inference works across expressions — you rarely need annotations:

```axon
val name = "Axon";            // String
val active = true;            // Bool
val scores = [95, 87, 92];    // Vec<Int32>
```

---

## Functions

Functions are declared with `fn`. Parameters require type annotations;
return types follow `:`.

```axon
fn add(a: Int32, b: Int32): Int32 {
    a + b   // last expression is the return value
}

fn greet(name: String) {
    println("Hello, {}!", name);
}

fn main() {
    val sum = add(3, 4);
    greet("World");
}
```

### Unsafe Functions

Functions performing low-level operations can be marked `unsafe`:

```axon
unsafe fn raw_pointer_access(ptr: *mut Float32): Float32 {
    // low-level memory access
}
```

---

## Basic Types

### Numeric Types

| Type      | Description             | Size    |
| --------- | ----------------------- | ------- |
| `Int8`    | Signed 8-bit integer    | 1 byte  |
| `Int16`   | Signed 16-bit integer   | 2 bytes |
| `Int32`   | Signed 32-bit integer   | 4 bytes |
| `Int64`   | Signed 64-bit integer   | 8 bytes |
| `UInt8`   | Unsigned 8-bit integer  | 1 byte  |
| `UInt16`  | Unsigned 16-bit integer | 2 bytes |
| `UInt32`  | Unsigned 32-bit integer | 4 bytes |
| `UInt64`  | Unsigned 64-bit integer | 8 bytes |
| `Float16` | 16-bit floating point   | 2 bytes |
| `Float32` | 32-bit floating point   | 4 bytes |
| `Float64` | 64-bit floating point   | 8 bytes |

### Other Primitives

| Type     | Description                |
| -------- | -------------------------- |
| `Bool`   | Boolean (`true` / `false`) |
| `Char`   | Unicode scalar value       |
| `String` | UTF-8 encoded string       |

### Integer Literals

```axon
val dec = 42;          // decimal
val hex = 0xFF;        // hexadecimal
val bin = 0b1010;      // binary
val oct = 0o77;        // octal
val sci = 1.5e10;      // scientific notation
```

---

## Control Flow

### If / Else

`if` is an expression — it returns a value:

```axon
val max = if a > b { a } else { b };

if score >= 90 {
    println("Excellent");
} else if score >= 70 {
    println("Good");
} else {
    println("Keep trying");
}
```

### While Loops

```axon
var i = 0;
while i < 10 {
    println("{}", i);
    i += 1;
}
```

### For Loops

```axon
for item in collection {
    println("{}", item);
}

for i in 0..10 {
    println("{}", i);
}
```

### Match Expressions

Pattern matching with exhaustiveness checking:

```axon
match value {
    0 => println("zero"),
    1 => println("one"),
    n => println("other: {}", n),
}

match option_val {
    Some(x) => println("Got {}", x),
    None => println("Nothing"),
}
```

---

## Models

Named product types with fields:

```axon
model Point {
    x: Float64,
    y: Float64,
}

val p = Point { x: 1.0, y: 2.0 };
println("({}, {})", p.x, p.y);
```

### Methods via `extend`

```axon
extend Point {
    fn distance(&self, other: &Point): Float64 {
        val dx = self.x - other.x;
        val dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    fn origin(): Point {
        Point { x: 0.0, y: 0.0 }
    }
}
```

---

## Enums

Sum types with variants that can hold data:

```axon
enum Shape {
    Circle(Float64),
    Rectangle(Float64, Float64),
    Triangle { base: Float64, height: Float64 },
}

fn area(shape: Shape): Float64 {
    match shape {
        Shape.Circle(r) => 3.14159 * r * r,
        Shape.Rectangle(w, h) => w * h,
        Shape.Triangle { base, height } => 0.5 * base * height,
    }
}
```

---

## Traits and Extend Blocks

Traits define shared behavior:

```axon
trait Printable {
    fn to_string(&self): String;
}

extend Printable for Point {
    fn to_string(&self): String {
        format("({}, {})", self.x, self.y)
    }
}
```

### Trait Bounds

```axon
fn print_all<T: Printable>(items: Vec<T>) {
    for item in items {
        println("{}", item.to_string());
    }
}
```

### Supertraits

```axon
trait Drawable: Printable {
    fn draw(&self);
}
```

---

## Generics

Functions, models, and traits can be generic:

```axon
fn max<T: Ord>(a: T, b: T): T {
    if a > b { a } else { b }
}

model Pair<A, B> {
    first: A,
    second: B,
}

extend<A: Display, B: Display> Pair<A, B> {
    fn show(&self) {
        println("({}, {})", self.first, self.second);
    }
}
```

---

## Tensor Types and Shape Annotations

Axon's killer feature — tensors are first-class citizens with compile-time
shape verification:

```axon
// Tensor with known shape
val weights: Tensor<Float32, [784, 256]> = randn([784, 256]);

// Dynamic batch dimension with ?
val input: Tensor<Float32, [?, 784]> = load_batch();

// Matrix multiply — shapes checked at compile time
val output = input @ weights;   // Tensor<Float32, [?, 256]>
```

Shape mismatches are caught before your code ever runs:

```axon
val a: Tensor<Float32, [3, 4]> = randn([3, 4]);
val b: Tensor<Float32, [5, 6]> = randn([5, 6]);
val c = a @ b;   // ERROR[E3001]: shape mismatch — inner dims 4 ≠ 5
```

See the [Tensor Guide](tensors.md) for the full story.

---

## Error Handling

Axon uses `Option<T>` and `Result<T, E>` for safe error handling:

```axon
fn find(haystack: Vec<Int32>, needle: Int32): Option<Int32> {
    for i in 0..haystack.len() {
        if haystack[i] == needle {
            return Some(i);
        }
    }
    None
}

fn read_config(path: String): Result<Config, IOError> {
    val file = File.open(path)?;    // propagate error with ?
    val data = file.read_all()?;
    parse_config(data)
}
```

See [Error Handling](error-handling.md) for patterns and best practices.

---

## Modules and Visibility

Organize code into modules with `mod` and `use`:

```axon
mod math {
    pub fn square(x: Float64): Float64 {
        x * x
    }

    fn internal_helper() {
        // private — not visible outside this module
    }
}

use math.square;

fn main() {
    println("{}", square(4.0));   // 16.0
}
```

See [Modules & Packages](modules-packages.md) for the full module system.

---

## What's Next?

| Topic                  | Guide                                                            |
| ---------------------- | ---------------------------------------------------------------- |
| Ownership & borrowing  | [ownership-borrowing.md](ownership-borrowing.md)                 |
| Tensor programming     | [tensors.md](tensors.md)                                         |
| GPU programming        | [gpu-programming.md](gpu-programming.md)                         |
| Error handling         | [error-handling.md](error-handling.md)                           |
| Modules & packages     | [modules-packages.md](modules-packages.md)                       |
| Build a neural network | [Tutorial: MNIST Classifier](../tutorial/03-mnist-classifier.md) |
