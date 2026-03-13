# Tutorial 06: Error Handling

Axon uses **Result** and **Option** types for explicit, type-safe error handling.
No hidden exceptions — every fallible operation returns a value you must handle.

## The Option Type

`Option<T>` represents a value that may or may not exist:

```axon
enum Option<T> {
    Some(T),
    None,
}
```

### Using Option

```axon
fn find_max(data: Tensor<Float32, [_]>): Option<Float32> {
    if data.len() == 0 {
        return None;
    }
    return Some(data.max());
}

fn main() {
    val values = tensor([1.0, 5.0, 3.0, 9.0, 2.0]);

    match find_max(values) {
        Some(max) => println("Max value: {}", max),
        None => println("Empty tensor"),
    }
}
```

### Option Combinators

```axon
val maybe_value: Option<Float64> = Some(42.0);

// unwrap_or: provide a default
val value = maybe_value.unwrap_or(0.0);

// map: transform the inner value
val doubled = maybe_value.map(|x| x * 2.0);

// is_some / is_none: check presence
if maybe_value.is_some() {
    println("Got a value!");
}
```

## The Result Type

`Result<T, E>` represents an operation that can succeed (`Ok`) or fail (`Err`):

```axon
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Using Result

```axon
fn load_model(path: String): Result<Model, String> {
    if !file_exists(path) {
        return Err("Model file not found: " + path);
    }
    val data = read_file(path)?;  // ? propagates errors
    return Ok(parse_model(data));
}

fn main() {
    match load_model("weights.axon") {
        Ok(model) => println("Loaded model with {} params", model.param_count()),
        Err(e) => println("Error: {}", e),
    }
}
```

### The `?` Operator

The `?` operator propagates errors up the call stack automatically:

```axon
fn train_pipeline(config_path: String): Result<Float64, String> {
    val config = load_config(config_path)?;     // returns Err early if this fails
    val data = load_dataset(config.data_path)?;  // same here
    val model = build_model(config)?;             // and here

    val final_loss = train(model, data, config.epochs)?;
    return Ok(final_loss);
}
```

This is equivalent to writing `match` at every step, but much more concise.

### Custom Error Types

Define your own error types for domain-specific errors:

```axon
enum TrainingError {
    DataNotFound(String),
    InvalidShape { expected: Shape, actual: Shape },
    ConvergenceFailure { epoch: Int32, loss: Float64 },
    OutOfMemory,
}

fn train(model: Model, data: Dataset): Result<Model, TrainingError> {
    if data.shape() != model.expected_input_shape() {
        return Err(TrainingError.InvalidShape {
            expected: model.expected_input_shape(),
            actual: data.shape(),
        });
    }
    // ... training logic ...
    return Ok(model);
}
```

## Combining Option and Result

Convert between Option and Result:

```axon
// Option → Result: provide an error message for the None case
val value: Result<Float64, String> = maybe_value.ok_or("value was missing");

// Result → Option: discard the error info
val maybe: Option<Float64> = result.ok();
```

## Panics

For truly unrecoverable errors, use `panic`:

```axon
fn assert_valid_shape(t: Tensor<Float32, [_, _]>) {
    if t.shape()[0] == 0 {
        panic("tensor must have at least one row");
    }
}
```

Panics terminate the program immediately with a source location and message.
Use them for programming errors (invariant violations), not expected failure modes.

## Best Practices

1. **Use Result for recoverable errors** — file I/O, network, parsing
2. **Use Option for missing values** — lookups, optional config fields
3. **Use panic for bugs** — invariant violations, unreachable code
4. **Use `?` operator** — keeps error-handling code concise
5. **Define domain error types** — makes errors self-documenting

## Next Steps

- [Tutorial 05: Models and Enums](05-structs-and-enums.md) — Custom data types
- [Tutorial 01: Hello Tensor](01-hello-tensor.md) — Start from the beginning
