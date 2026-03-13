# Compiler Error Reference

Complete reference for all Axon compiler error codes. Each error includes its
code, description, example code that triggers it, and how to fix it.

---

## Error Code Ranges

| Range       | Category        | Description                            |
| ----------- | --------------- | -------------------------------------- |
| E0001–E0099 | Lexer / Parser  | Syntax errors                          |
| E1001–E1099 | Name Resolution | Undefined or duplicate names           |
| E2001–E2099 | Type Errors     | Type mismatches and inference failures |
| E3001–E3099 | Shape Errors    | Tensor shape mismatches                |
| E4001–E4099 | Borrow Errors   | Ownership and lifetime violations      |
| W5001–W5010 | Lint Warnings   | Style and best-practice warnings       |

---

## E0001–E0099: Lexer / Parser Errors

### E0001: Unexpected Character

```axon
val x = 42$;
//         ^ ERROR[E0001]: unexpected character `$`
```

**Fix**: Remove or replace the invalid character.

### E0002: Unterminated String Literal

```axon
val s = "hello;
//      ^ ERROR[E0002]: unterminated string literal
```

**Fix**: Close the string with a matching `"`.

### E0003: Unterminated Block Comment

```axon
/* this comment never ends
//^ ERROR[E0003]: unterminated block comment
```

**Fix**: Close with `*/`. Nested comments require matching pairs.

### E0010: Expected Token

```axon
fn foo( {
//      ^ ERROR[E0010]: expected `)`, found `{`
```

**Fix**: Add the missing token.

### E0011: Expected Expression

```axon
val x = ;
//      ^ ERROR[E0011]: expected expression, found `;`
```

**Fix**: Provide a value or expression.

### E0012: Expected Type

```axon
val x: = 42;
//     ^ ERROR[E0012]: expected type, found `=`
```

**Fix**: Provide a type annotation after `:`.

### E0020: Invalid Integer Literal

```axon
val x = 0xGG;
//      ^ ERROR[E0020]: invalid hexadecimal literal
```

**Fix**: Use valid digits for the number base (0-9, a-f for hex).

### E0021: Invalid Float Literal

```axon
val x = 1.2.3;
//      ^ ERROR[E0021]: invalid float literal
```

### E0030: Invalid Escape Sequence

```axon
val s = "\q";
//       ^ ERROR[E0030]: unknown escape sequence `\q`
```

**Fix**: Use valid escapes: `\\`, `\n`, `\t`, `\r`, `\"`, `\0`.

### E0040: Duplicate Match Arm

```axon
match x {
    1 => println("one"),
    1 => println("one again"),  // ERROR[E0040]: duplicate match arm
}
```

### E0050: Invalid Pattern

```axon
match value {
    1 + 2 => println("?"),  // ERROR[E0050]: expected pattern, found expression
}
```

---

## E1001–E1099: Name Resolution Errors

### E1001: Undefined Variable

```axon
fn main() {
    println("{}", unknown_var);
//                ^ ERROR[E1001]: undefined variable `unknown_var`
}
```

**Fix**: Declare the variable before use or check for typos.

### E1002: Undefined Function

```axon
fn main() {
    foo();
//  ^ ERROR[E1002]: undefined function `foo`
}
```

### E1003: Undefined Type

```axon
val x: NonExistent = 42;
//     ^ ERROR[E1003]: undefined type `NonExistent`
```

### E1010: Duplicate Definition

```axon
fn foo() {}
fn foo() {}
// ^ ERROR[E1010]: duplicate definition of `foo`
```

### E1011: Duplicate Field

```axon
model Point { x: Int32, x: Int32 }
//                       ^ ERROR[E1011]: duplicate field `x`
```

### E1020: Unresolved Import

```axon
use std.nonexistent.Module;
//  ^ ERROR[E1020]: unresolved import `std.nonexistent`
```

### E1030: Private Item

```axon
mod inner {
    fn secret() {}
}
inner.secret();
// ^ ERROR[E1030]: function `secret` is private
```

**Fix**: Add `pub` to the item or access it from within its module.

---

## E2001–E2099: Type Errors

### E2001: Type Mismatch

```axon
val x: Int32 = "hello";
// ERROR[E2001]: type mismatch — expected `Int32`, found `String`
```

### E2002: Binary Operator Type Error

```axon
val x = "hello" + 42;
// ERROR[E2002]: cannot apply `+` to `String` and `Int32`
```

### E2003: Return Type Mismatch

```axon
fn foo(): Int32 {
    "not an integer"
// ERROR[E2003]: return type mismatch — expected `Int32`, found `String`
}
```

### E2010: Missing Field

```axon
model Point { x: Int32, y: Int32 }
val p = Point { x: 1 };
// ERROR[E2010]: missing field `y` in struct `Point`
```

### E2011: Unknown Field

```axon
model Point { x: Int32, y: Int32 }
val p = Point { x: 1, y: 2, z: 3 };
//                           ^ ERROR[E2011]: unknown field `z` on `Point`
```

### E2020: Trait Not Implemented

```axon
fn print_it<T: Display>(x: T) {}
print_it(SomeStruct {});
// ERROR[E2020]: trait `Display` not implemented for `SomeStruct`
```

**Fix**: Implement the required trait for the type.

### E2021: Ambiguous Method

```axon
// When multiple trait impls provide the same method
value.shared_method();
// ERROR[E2021]: ambiguous method call — candidates from `TraitA` and `TraitB`
```

**Fix**: Use fully qualified syntax: `TraitA.shared_method(&value)`.

### E2030: Cannot Infer Type

```axon
val x = Vec.new();
// ERROR[E2030]: cannot infer type — add a type annotation
```

**Fix**: `val x: Vec<Int32> = Vec.new();`

### E2040: Invalid Cast

```axon
val x = "hello" as Int32;
// ERROR[E2040]: cannot cast `String` to `Int32`
```

---

## E3001–E3099: Shape Errors

### E3001: Matmul Shape Mismatch

```axon
val a: Tensor<Float32, [3, 4]> = randn([3, 4]);
val b: Tensor<Float32, [5, 6]> = randn([5, 6]);
val c = a @ b;
// ERROR[E3001]: matmul shape mismatch — inner dimensions 4 ≠ 5
//   note: left shape [3, 4], right shape [5, 6]
```

**Fix**: Ensure the inner dimensions match: `[M, K] @ [K, N]`.

### E3002: Invalid Reshape

```axon
val t: Tensor<Float32, [2, 3]> = randn([2, 3]);
val r = t.reshape([2, 2]);
// ERROR[E3002]: cannot reshape [2, 3] (6 elements) to [2, 2] (4 elements)
```

**Fix**: Ensure the total number of elements is preserved.

### E3003: Broadcast Incompatible

```axon
val a: Tensor<Float32, [3, 4]> = randn([3, 4]);
val b: Tensor<Float32, [3, 5]> = randn([3, 5]);
val c = a + b;
// ERROR[E3003]: shapes [3, 4] and [3, 5] are not broadcast-compatible
```

### E3010: Invalid Transpose Axes

```axon
val t: Tensor<Float32, [2, 3, 4]> = randn([2, 3, 4]);
val p = t.permute([0, 1, 5]);
// ERROR[E3010]: axis 5 out of range for tensor with 3 dimensions
```

### E3020: Dynamic Shape Required

```axon
// When static shape info is unavailable
// ERROR[E3020]: cannot verify shape statically — consider using `?` for dynamic dims
//   note: runtime shape check will be inserted
```

---

## E4001–E4099: Borrow Errors

### E4001: Use After Move

```axon
val data = randn([100]);
val other = data;
println("{}", data);
// ERROR[E4001]: use of moved value `data`
//   note: `data` was moved on line 2
```

**Fix**: Clone the value or restructure to avoid the move.

### E4002: Borrow of Moved Value

```axon
val s = "hello".to_string();
val t = s;
val r = &s;
// ERROR[E4002]: cannot borrow `s` — value has been moved
```

### E4003: Mutable Borrow Conflict

```axon
var v = vec![1, 2, 3];
val r1 = &v;
val r2 = &mut v;
// ERROR[E4003]: cannot borrow `v` as mutable — also borrowed as immutable
//   note: immutable borrow of `v` occurs on line 2
```

**Fix**: Ensure immutable borrows end before taking a mutable borrow.

### E4004: Multiple Mutable Borrows

```axon
var data = randn([10]);
val a = &mut data;
val b = &mut data;
// ERROR[E4004]: cannot borrow `data` as mutable more than once
```

### E4005: Dangling Reference

```axon
fn dangling(): &String {
    val s = "hello".to_string();
    &s
// ERROR[E4005]: `s` does not live long enough
//   note: borrowed value only lives until end of function
}
```

**Fix**: Return an owned value instead of a reference.

### E4006: Mutability Required

```axon
val data = randn([10]);
scale(&mut data, 2.0);
// ERROR[E4006]: cannot borrow `data` as mutable — declared as immutable
//   help: consider changing to `var data`
```

### E4007: Cross-Device Borrow

```axon
var t = randn([256]);
val cpu_ref = &t;
val gpu_t = t.to_gpu();
// ERROR[E4007]: cannot move `t` to GPU while borrowed on CPU
```

---

## W5001–W5010: Lint Warnings

### W5001: Unused Variable

```axon
val x = 42;
// WARNING[W5001]: unused variable `x`
//   help: prefix with underscore: `_x`
```

### W5002: Unused Import

```axon
use std.math.sin;
// WARNING[W5002]: unused import `sin`
```

### W5003: Dead Code

```axon
fn unused_function() {}
// WARNING[W5003]: function `unused_function` is never called
```

### W5004: Unnecessary Mutability

```axon
var x = 42;
println("{}", x);
// WARNING[W5004]: variable `x` declared as mutable but never mutated
```

### W5005: Shadowed Variable

```axon
val x = 1;
val x = 2;
// WARNING[W5005]: variable `x` shadows previous declaration
```

### W5006: Naming Convention

```axon
fn MyFunction() {}
// WARNING[W5006]: function `MyFunction` should use snake_case
//   help: rename to `my_function`
```

### W5007: Redundant Type Annotation

```axon
val x: Int32 = 42;
// WARNING[W5007]: type annotation is redundant — inferred as `Int32`
```

### W5008: Missing Documentation

```axon
pub fn public_api() {}
// WARNING[W5008]: public item `public_api` is missing documentation
```

---

## Error Output Formats

### Human-Readable (Default)

```
error[E2001]: type mismatch — expected `Int32`, found `String`
  --> src/main.axon:5:15
  help: consider using `parse()` to convert the string
```

### JSON (`--error-format=json`)

```json
{
  "error_code": "E2001",
  "message": "type mismatch — expected `Int32`, found `String`",
  "severity": "error",
  "location": { "file": "src/main.axon", "line": 5, "column": 15 },
  "suggestion": "consider using `parse()` to convert the string"
}
```

---

## See Also

- [CLI Reference](cli-reference.md) — compiler flags including `--error-format`
- [Language Tour](../guide/language-tour.md) — learn correct syntax
- [Ownership & Borrowing](../guide/ownership-borrowing.md) — understand E4xxx errors
