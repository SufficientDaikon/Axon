# Token & Operator Registry

> All locations relative to `h:/programing language/axon/src/`
> Line numbers are approximate — use grep patterns to find exact locations

## Keywords (token.rs TokenKind enum, lines 9-32)

| Axon | TokenKind | lookup_keyword() line | Notes |
|------|-----------|----------------------|-------|
| `fn` | `Fn` | ~249 | Function declaration |
| `let` | `Let` | ~250 | Variable binding |
| `mut` | `Mut` | ~251 | Mutable qualifier |
| `struct` | `Struct` | ~252 | Struct definition |
| `enum` | `Enum` | ~253 | Enum definition |
| `impl` | `Impl` | ~254 | Implementation block |
| `trait` | `Trait` | ~255 | Trait definition |
| `if` | `If` | ~256 | Conditional |
| `else` | `Else` | ~257 | Else branch |
| `while` | `While` | ~258 | While loop |
| `for` | `For` | ~259 | For loop |
| `in` | `In` | ~260 | For-in separator |
| `match` | `Match` | ~261 | Pattern matching |
| `return` | `Return` | ~262 | Return statement |
| `pub` | `Pub` | ~263 | Public visibility |
| `use` | `Use` | ~264 | Use declaration |
| `mod` | `Mod` | ~265 | Module declaration |
| `unsafe` | `Unsafe` | ~266 | Unsafe block |
| `self` | `SelfValue` | ~267 | Self reference |
| `true` | `True` | ~268 | Boolean literal |
| `false` | `False` | ~269 | Boolean literal |
| `as` | `As` | ~270 | Type cast |
| `type` | `Type` | ~271 | Type alias |

## Built-in Type Keywords (token.rs lines 34-58)

| Axon | TokenKind | lookup_keyword() line |
|------|-----------|----------------------|
| `Int8`..`Int64` | `Int8`..`Int64` | ~273-276 |
| `UInt8`..`UInt64` | `UInt8`..`UInt64` | ~277-280 |
| `Float16`..`Float64` | `Float16`..`Float64` | ~281-283 |
| `Bool` | `BoolType` | ~284 |
| `Char` | `CharType` | ~285 |
| `String` | `StringType` | ~286 |
| `Tensor` | `TensorType` | ~287 |
| `Vec` | `VecType` | ~288 |
| `HashMap` | `HashMapType` | ~289 |
| `HashSet` | `HashSetType` | ~290 |
| `Option` | `OptionType` | ~291 |
| `Result` | `ResultType` | ~292 |
| `Mutex` | `MutexType` | ~293 |
| `RwLock` | `RwLockType` | ~294 |
| `Channel` | `ChannelType` | ~295 |
| `Arc` | `ArcType` | ~296 |

## Operators (token.rs lines 60-88, lexer.rs character matching)

| Symbol | TokenKind | 1st Char | Lexer Lines | Parser Role |
|--------|-----------|----------|-------------|-------------|
| `+` | `Plus` | `+` | ~265-272 | binary add, compound `+=` |
| `-` | `Minus` | `-` | ~275-286 | binary sub, unary neg, `->`, `-=` |
| `*` | `Star` | `*` | ~288-296 | binary mul, deref, `*=` |
| `/` | `Slash` | `/` | ~298-306 | binary div, `/=` |
| `@` | `At` | `@` | ~186 | matmul, decorator attrs |
| `%` | `Percent` | `%` | ~308-311 | modulus |
| `==` | `EqEq` | `=` | ~189-193 | equality comparison |
| `!=` | `NotEq` | `!` | ~202-210 | inequality |
| `<` | `Lt` | `<` | ~212-219 | less than, generics open |
| `>` | `Gt` | `>` | ~222-229 | greater than, generics close |
| `<=` | `LtEq` | `<` | ~214-216 | less or equal |
| `>=` | `GtEq` | `>` | ~224-226 | greater or equal |
| `&&` | `AndAnd` | `&` | ~232-236 | logical AND |
| `\|\|` | `OrOr` | `\|` | ~248-253 | logical OR |
| `!` | `Bang` | `!` | ~202-209 | logical NOT, `!=` |
| `&` | `Amp` | `&` | ~232-245 | reference, `&&`, `&mut` |
| `&mut` | `AmpMut` | `&` | ~237-242 | mutable reference |
| `?` | `Question` | `?` | ~313-316 | error propagation |
| `->` | `Arrow` | `-` | ~276-279 | **return type annotation** |
| `=>` | `FatArrow` | `=` | ~194-196 | **match arm body** |
| `::` | `ColonColon` | `:` | ~328-332 | **path separator** |
| `.` | `Dot` | `.` | ~318-325 | field access, method call |
| `..` | `DotDot` | `.` | ~320-322 | range |
| `=` | `Eq` | `=` | ~197-199 | assignment |
| `+=` | `PlusEq` | `+` | ~267-269 | compound assign |
| `-=` | `MinusEq` | `-` | ~280-282 | compound assign |
| `*=` | `StarEq` | `*` | ~290-292 | compound assign |
| `/=` | `SlashEq` | `/` | ~300-302 | compound assign |

## Delimiters (token.rs lines 90-99)

| Symbol | TokenKind | Lexer Line |
|--------|-----------|-----------|
| `(` | `LParen` | ~339 |
| `)` | `RParen` | ~340 |
| `{` | `LBrace` | ~341 |
| `}` | `RBrace` | ~342 |
| `[` | `LBracket` | ~343 |
| `]` | `RBracket` | ~344 |
| `,` | `Comma` | ~345 |
| `;` | `Semicolon` | ~346 |
| `:` | `Colon` | ~333-335 |

## Operator Precedence (parser.rs ~lines 938-946)

Lowest to highest. Each level is a parse function that calls the next:

| Level | Operators | Parse Function |
|-------|-----------|----------------|
| 1 | `\|\|` | `parse_or_expr()` |
| 2 | `&&` | `parse_and_expr()` |
| 3 | `==` `!=` `<` `>` `<=` `>=` | `parse_comparison_expr()` |
| 4 | `..` | `parse_range_expr()` |
| 5 | `+` `-` | `parse_additive_expr()` |
| 6 | `*` `/` `%` `@` | `parse_multiplicative_expr()` |
| 7 | `as` | `parse_cast_expr()` |
| 8 | `-` `!` `&` `&mut` (unary) | `parse_unary_expr()` |
| 9 | `?` `()` `[]` `.` (postfix) | `parse_postfix_expr()` |
