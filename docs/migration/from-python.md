# Migrating from Python to Axon

A side-by-side guide for Python developers moving to Axon. Axon will feel
familiar in many ways, but adds static types, ownership, and compile-time
shape checking.

---

## Variables

| Python                    | Axon                 |
| ------------------------- | -------------------- |
| `x = 42`                  | `val x = 42;`        |
| `x = 42` (reassign later) | `var x = 42;`    |
| `x: int = 42`             | `val x: Int32 = 42;` |

```python
# Python
name = "Alice"
age = 30
scores = [95, 87, 92]
```

```axon
// Axon
val name = "Alice";
val age = 30;
val scores = vec![95, 87, 92];
```

Key difference: Axon variables are **immutable by default**. Use `var`
for mutable variables.

---

## Functions

```python
# Python
def add(a: int, b: int) -> int:
    return a + b

def greet(name: str):
    print(f"Hello, {name}!")
```

```axon
// Axon
fn add(a: Int32, b: Int32): Int32 {
    a + b    // implicit return (last expression)
}

fn greet(name: String) {
    println("Hello, {}!", name);
}
```

Differences:

- `fn` instead of `def`
- Curly braces instead of indentation
- Type annotations are required on parameters
- No `return` needed for the last expression

---

## Types

| Python           | Axon                     | Notes          |
| ---------------- | ------------------------ | -------------- |
| `int`            | `Int32` / `Int64`        | Explicit sizes |
| `float`          | `Float32` / `Float64`    | Explicit sizes |
| `bool`           | `Bool`                   |                |
| `str`            | `String`                 |                |
| `list[int]`      | `Vec<Int32>`             |                |
| `dict[str, int]` | `HashMap<String, Int32>` |                |
| `Optional[int]`  | `Option<Int32>`          |                |
| `None`           | `None`                   |                |

---

## Control Flow

### If/Else

```python
# Python
if score >= 90:
    grade = "A"
elif score >= 70:
    grade = "B"
else:
    grade = "C"
```

```axon
// Axon — if is an expression!
val grade = if score >= 90 {
    "A"
} else if score >= 70 {
    "B"
} else {
    "C"
};
```

### Loops

```python
# Python
for i in range(10):
    print(i)

for item in items:
    process(item)

while condition:
    do_work()
```

```axon
// Axon
for i in 0..10 {
    println("{}", i);
}

for item in items {
    process(item);
}

while condition {
    do_work();
}
```

### Pattern Matching

```python
# Python 3.10+
match command:
    case "quit":
        exit()
    case "hello":
        print("Hi!")
    case _:
        print("Unknown")
```

```axon
// Axon
match command {
    "quit" => exit(),
    "hello" => println("Hi!"),
    _ => println("Unknown"),
}
```

---

## Classes → Models + Extend

Python classes map to Axon models with extend blocks:

```python
# Python
class Point:
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def distance(self, other: 'Point') -> float:
        return ((self.x - other.x)**2 + (self.y - other.y)**2)**0.5

    def __str__(self):
        return f"({self.x}, {self.y})"
```

```axon
// Axon
model Point {
    x: Float64,
    y: Float64,
}

extend Point {
    fn new(x: Float64, y: Float64): Point {
        Point { x, y }
    }

    fn distance(&self, other: &Point): Float64 {
        val dx = self.x - other.x;
        val dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

extend Display for Point {
    fn to_string(&self): String {
        format("({}, {})", self.x, self.y)
    }
}
```

Key differences:

- No inheritance — use traits for polymorphism
- `&self` is explicit (immutable borrow)
- Constructors are regular functions (by convention called `new`)

---

## Error Handling

```python
# Python
try:
    f = open("config.toml")
    data = f.read()
    config = parse(data)
except FileNotFoundError:
    print("Config not found")
except ParseError as e:
    print(f"Parse error: {e}")
```

```axon
// Axon
match File.open("config.toml") {
    Ok(file) => {
        match file.read_all() {
            Ok(data) => {
                val config = parse(data);
                println("Loaded config");
            }
            Err(e) => eprintln("Read error: {}", e),
        }
    }
    Err(e) => eprintln("Config not found: {}", e),
}

// Or more concisely with ?
fn load_config(): Result<Config, IOError> {
    val file = File.open("config.toml")?;
    val data = file.read_all()?;
    parse(data)
}
```

---

## NumPy / Tensors

```python
# Python (NumPy)
import numpy as np

a = np.zeros((3, 4))
b = np.random.randn(3, 4)
c = a + b
d = np.dot(a, b.T)
mean = np.mean(c, axis=0)
```

```axon
// Axon
val a = zeros([3, 4]);
val b = randn([3, 4]);
val c = a + b;
val d = a @ b.transpose();
val mean = c.mean(dim: 0);
```

### Key differences from NumPy:

- **Compile-time shape checking** — shape errors caught before runtime
- `@` operator for matrix multiplication (like Python 3.5+, but type-checked)
- No `import` needed — tensors are built-in types
- Shapes are part of the type: `Tensor<Float32, [3, 4]>`

---

## List Comprehensions → Iterators

```python
# Python
squares = [x**2 for x in range(10)]
evens = [x for x in numbers if x % 2 == 0]
```

```axon
// Axon (iterator methods)
val squares: Vec<Int32> = (0..10).map(|x| x * x).collect();
val evens: Vec<Int32> = numbers.iter().filter(|x| x % 2 == 0).collect();
```

---

## Modules

```python
# Python — file: math_utils.py
def square(x):
    return x * x

# main.py
from math_utils import square
```

```axon
// Axon — file: math_utils.axon
pub fn square(x: Float64): Float64 {
    x * x
}

// main.axon
mod math_utils;
use math_utils.square;
```

---

## Quick Reference

| Python               | Axon                                     |
| -------------------- | ---------------------------------------- |
| `print(x)`           | `println("{}", x)`                       |
| `len(x)`             | `x.len()`                                |
| `type(x)`            | Compile-time types (use `:type` in REPL) |
| `isinstance(x, T)`   | Pattern matching                         |
| `None`               | `None` (Option type)                     |
| `raise ValueError()` | `return Err(...)` or `panic(...)`        |
| `assert x > 0`       | `assert(x > 0)`                          |
| `# comment`          | `// comment`                             |
| `"""docstring"""`    | `/// doc comment`                        |
| `pip install`        | `axonc pkg add`                          |
| `python script.py`   | `axonc build script.axon && ./script`    |

---

## See Also

- [Language Tour](../guide/language-tour.md) — full syntax overview
- [Error Handling](../guide/error-handling.md) — Result and Option patterns
- [Migrating from PyTorch](from-pytorch.md) — ML-specific migration
