# Error Handling

Axon uses algebraic types for error handling — no exceptions, no null pointers.
Every possible failure is encoded in the type system.

---

## Option\<T\>

`Option<T>` represents a value that may or may not exist:

```axon
enum Option<T> {
    Some(T),
    None,
}
```

### Using Option

```axon
fn find_index(items: &Vec<String>, target: &String) -> Option<Int32> {
    for i in 0..items.len() {
        if items[i] == target {
            return Some(i);
        }
    }
    None
}

fn main() {
    let names = vec!["Alice", "Bob", "Charlie"];

    match find_index(&names, "Bob") {
        Some(idx) => println("Found at index {}", idx),
        None => println("Not found"),
    }
}
```

### Option Methods

```axon
let val: Option<Int32> = Some(42);

// Unwrap (panics if None)
let x = val.unwrap();              // 42

// Unwrap with default
let y = val.unwrap_or(0);          // 42
let z = None.unwrap_or(0);         // 0

// Map: transform the inner value
let doubled = val.map(|x| x * 2); // Some(84)

// is_some / is_none
if val.is_some() {
    println("Has a value");
}

// and_then: chain optional operations
let result = val
    .map(|x| x + 1)
    .and_then(|x| if x > 0 { Some(x) } else { None });
```

---

## Result\<T, E\>

`Result<T, E>` represents an operation that can succeed (`Ok`) or fail (`Err`):

```axon
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Using Result

```axon
fn parse_int(s: &String) -> Result<Int32, String> {
    // parsing logic...
    if valid {
        Ok(parsed_value)
    } else {
        Err("invalid integer: " + s)
    }
}

fn read_config(path: String) -> Result<Config, IOError> {
    let file = File::open(path)?;       // propagate error
    let contents = file.read_all()?;    // propagate error
    let config = parse_toml(contents)?;
    Ok(config)
}
```

### Result Methods

```axon
let ok: Result<Int32, String> = Ok(42);
let err: Result<Int32, String> = Err("oops");

// Unwrap (panics on Err)
let x = ok.unwrap();           // 42

// Unwrap with default
let y = err.unwrap_or(0);      // 0

// Map the success value
let doubled = ok.map(|x| x * 2);   // Ok(84)

// Map the error
let mapped_err = err.map_err(|e| IOError::new(e));

// is_ok / is_err
if ok.is_ok() {
    println("Success!");
}

// and_then: chain fallible operations
let result = ok
    .and_then(|x| if x > 0 { Ok(x) } else { Err("negative") });
```

---

## Pattern Matching on Errors

Pattern matching is the primary way to handle errors:

```axon
fn process_file(path: String) {
    match File::open(path) {
        Ok(file) => {
            match file.read_all() {
                Ok(data) => println("Read {} bytes", data.len()),
                Err(e) => eprintln("Read error: {}", e),
            }
        }
        Err(e) => eprintln("Open error: {}", e),
    }
}
```

### Matching Specific Error Types

```axon
match load_model("model.axon") {
    Ok(model) => {
        println("Model loaded: {} parameters", model.param_count());
    }
    Err(IOError::NotFound(path)) => {
        eprintln("File not found: {}", path);
    }
    Err(IOError::PermissionDenied(path)) => {
        eprintln("Permission denied: {}", path);
    }
    Err(e) => {
        eprintln("Unexpected error: {}", e);
    }
}
```

---

## The `?` Operator

The `?` operator propagates errors to the caller, reducing boilerplate:

```axon
// Without ?
fn load_data(path: String) -> Result<Vec<Float32>, IOError> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let contents = match file.read_all() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    parse_csv(contents)
}

// With ? — equivalent but cleaner
fn load_data(path: String) -> Result<Vec<Float32>, IOError> {
    let file = File::open(path)?;
    let contents = file.read_all()?;
    parse_csv(contents)
}
```

The `?` operator:

1. If the value is `Ok(v)`, unwraps to `v`
2. If the value is `Err(e)`, returns `Err(e)` from the enclosing function
3. Works on `Option<T>` too — `None` propagates as `None`

### Chaining with `?`

```axon
fn pipeline(path: String) -> Result<Model, Error> {
    let config = load_config(path)?;
    let data = load_dataset(&config.data_path)?;
    let model = build_model(&config)?;
    let trained = train(model, data)?;
    Ok(trained)
}
```

---

## Panic vs Recoverable Errors

### Recoverable Errors

Use `Result<T, E>` for expected failure modes:

```axon
fn connect(host: String) -> Result<Connection, NetworkError> {
    // network errors are expected — caller decides what to do
}
```

### Panics

Use `panic` for unrecoverable programmer errors:

```axon
fn get_element(v: &Vec<Int32>, idx: Int32) -> Int32 {
    if idx < 0 || idx >= v.len() {
        panic("index out of bounds: {} (len: {})", idx, v.len());
    }
    v[idx]
}
```

Panics terminate the program with a stack trace. Use them for:

- Logic errors / violated invariants
- `unwrap()` on `None` or `Err` when failure is truly unexpected
- Debug assertions

### Guidelines

| Situation           | Use                        |
| ------------------- | -------------------------- |
| File not found      | `Result<T, IOError>`       |
| Network timeout     | `Result<T, NetworkError>`  |
| Parse failure       | `Result<T, ParseError>`    |
| Index out of bounds | `panic`                    |
| Division by zero    | `panic`                    |
| Unimplemented code  | `panic("not implemented")` |

---

## Defining Custom Error Types

```axon
enum ModelError {
    LoadFailed(String),
    ShapeMismatch { expected: Vec<Int32>, actual: Vec<Int32> },
    TrainingDiverged,
}

impl Display for ModelError {
    fn to_string(&self) -> String {
        match self {
            ModelError::LoadFailed(path) => format("failed to load: {}", path),
            ModelError::ShapeMismatch { expected, actual } =>
                format("shape mismatch: expected {:?}, got {:?}", expected, actual),
            ModelError::TrainingDiverged => "training diverged (loss = NaN)".to_string(),
        }
    }
}

fn load_and_train(path: String) -> Result<Model, ModelError> {
    let model = load_model(path).map_err(|e| ModelError::LoadFailed(e.to_string()))?;
    train(model)
}
```

---

## Error Handling in ML Code

A realistic training function with comprehensive error handling:

```axon
fn train_model(config: &TrainConfig) -> Result<Model, ModelError> {
    let data = DataLoader::from_csv(&config.data_path)
        .map_err(|e| ModelError::LoadFailed(e.to_string()))?;

    let mut model = NeuralNet::new(config.hidden_size);
    let mut optimizer = Adam::new(model.parameters(), lr: config.learning_rate);

    for epoch in 0..config.epochs {
        let mut epoch_loss = 0.0;

        for batch in &data {
            let (inputs, targets) = batch;
            let predictions = model.forward(inputs);
            let loss = cross_entropy(predictions, targets);

            // Check for divergence
            if loss.item().is_nan() {
                return Err(ModelError::TrainingDiverged);
            }

            epoch_loss += loss.item();
            loss.backward();
            optimizer.step();
            optimizer.zero_grad();
        }

        println("Epoch {}: loss = {:.4}", epoch, epoch_loss / data.len() as Float32);
    }

    Ok(model)
}
```

---

## Summary

| Concept            | Type           | Use Case                           |
| ------------------ | -------------- | ---------------------------------- |
| Missing value      | `Option<T>`    | Lookup, search, optional fields    |
| Fallible operation | `Result<T, E>` | I/O, parsing, network              |
| Error propagation  | `?`            | Clean chaining of fallible calls   |
| Unrecoverable      | `panic(...)`   | Logic errors, invariant violations |
| Pattern matching   | `match`        | Exhaustive error handling          |

---

## See Also

- [Language Tour](language-tour.md) — syntax overview
- [Compiler Errors](../reference/compiler-errors.md) — error code reference
- [Migration from Python](../migration/from-python.md) — Python try/except vs Axon Result
