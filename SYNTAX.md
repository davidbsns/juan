# Juan Syntax
This is the Juan language specification. (Mainly made to implement Juan properly)  
This does NOT help the average programmer or whatever, this is just for me to write everything about Juan and how it works inside to again, implement it well.

## 1. Naming Conventions

Juan diagnoses declarations that do not follow these case styles. Projects may promote the warnings to errors.

* **Modules**: `snake_case`  
* **Types**: `PascalCase`  
* **Functions & Variables**: `snake_case`  
* **Constants**: `SCREAMING_SNAKE_CASE`  
* **Primitives**: `lowercase`

---

## 2. Comments
* `//` — Single-line comment
* `/* ... */` — Multi-line block comment. These may be nested.
* `///` — Outer doc comment. Documents the declaration directly after it.
* `//!` — Inner doc comment. Documents the current module.
* `/** ... */` — Multi-line outer documentation block. Documents the declaration directly after it.

---

## 3. Modules, Packages & Visibility

Every `.juan` file declares exactly one module namespace. Module paths are independent of filenames and directories. A concrete module path may be declared by only one file in a package; its parent namespaces exist virtually and are importable without their own files.

### 3.1 Module Declaration
```juan
module my_module
```

The module declaration must be the first non-comment declaration in the file. Imports must come directly after it, before ordinary declarations.

### 3.2 Virtual Submodules

A dotted module path declares a submodule without requiring a concrete root module.

```juan
// File A
module foo.bar

// File B
module foo.idk
```

### 3.3 Importing Modules

Importing a module or virtual parent imports its namespace tree. Submodules remain qualified and are NOT injected as unqualified local names.

```juan
import foo

foo.bar.do_something()
foo.idk.do_something_else()
```

You can still import one exact submodule directly. The final part of that path becomes the local module name:

```juan
import foo.bar

bar.do_something()
```

You can rename the local module if needed:

```juan
import foo.bar as foo_bar

foo_bar.do_something()
```

#### Collision handling
If two imported modules would get the same local name, it is a compile error. At least one of them must use an explicit alias.

```juan
import mod_a.sub as sub_a
import mod_b.sub as sub_b

sub_a.do_something()
sub_b.do_something_else()
```

Import order never decides which declaration wins.

#### Selective Imports
A selective import directly imports a type, value, operator or macro:

```juan
import { Player } from game.player
import game.player

let player = player.new("Davey", 100.0)
```

Module imports remain qualified while selective imports create local names. Imports are private to the current module and never automatically re-export anything.

Types, values and macros use separate namespaces. An ambiguous use is a compile error that lists every candidate.

### 3.4 Packages & Dependencies

A package manifest lists source roots and dependency aliases.

The standard library owns the `std` root. A dependency's manifest alias becomes its import root:

```juan
import awesome_math.vector
```

Local roots, dependency aliases and `std` may not collide. The package-manifest format is outside Juan's source grammar.

### 3.5 Visibility

Declarations and fields are module-private by default.

* `pub` — Visible from other packages.
* `pub(package)` — Visible to every module in the same package, but not outside it.
* No modifier — Visible only inside the declaring module.

An import never increases the visibility of the declaration it names.

---

## 4. Control Flow

Control-flow constructs are expressions. Blocks use `{ ... }`.

### 4.1 Block Values & Newlines

Juan does not use statement semicolons. A completed newline separates items in a block, and the final bare expression before `}` becomes the block's value. `return` is only needed to return early.

```juan
fn calculate(): i32 {
    let base = 10
    let bonus = 5
    base + bonus
}
```

Earlier bare expressions are evaluated and discarded. Discarding a `Result` without handling it produces a warning; `let _ = expression` explicitly acknowledges dismissal.

A block with no final expression evaluates to `Unit`. A function with no return type must produce `Unit`. To explicitly discard a non-`Unit` result in the final position, use `let _ = expression`.

Newlines inside `()`, `[]`, record types and record values do not finish an expression. Inside a block, a newline separates completed items. A newline also continues after an incomplete token such as an operator, comma, `=`, or `=>`. A line beginning with `|>` continues the previous pipeline, and a line beginning with `.` continues a postfix member or method chain.

```juan
let total = base +
    bonus

let result = input
    |> validate()
    |> transform()
```

Other than the special leading `|>` form, operators must end the line when continuing an expression.

### 4.2 Conditional Expressions

Conditions MUST be `bool`; integers, strings and handles are not implicitly truthy.

```juan
let text = if true {
    "True"
} else {
    "False"
}
```

When an `if` is used as a value it must have an `else`, and all reachable branches must produce compatible types. An `if` used only for its effects may omit the `else` and evaluates to `Unit`.

`else if` forms one conditional chain:

```juan
let text = if score >= 100 {
    "Amazing"
} else if score >= 50 {
    "Good"
} else {
    "Try again"
}
```

### 4.3 Loops & Iteration

#### Infinite Loop (`loop`)
Executes a block indefinitely until broken using `break`. A `loop` may also produce a value using `break <expr>`.

```juan
let result = loop {
    if ready() {
        break calculate_result()
    }
}
```

Every reachable value-carrying `break` in the same loop must produce a compatible type. The repeating loop body itself must produce `Unit`; the value of the entire `loop` expression comes from `break <expr>`.

#### Conditional Loop (`while`)
Repeats a block as long as the boolean condition is `true`.

```juan
while true {
    io.println("Still running")
}
```

`while` and its body must produce `Unit`, and `break` may not carry a value inside it.

#### Iteration (`for`)
Iterates over a range or a collection.

```juan
// 0, 1, 2, 3, 4
for i in 0..5 {
    io.println(i)
}

// 0, 1, 2, 3, 4, 5
for i in 0..=5 {
    io.println(i)
}

for item in inventory {
    item.do_something()
}
```

A range `a..b` excludes `b`; `a..=b` includes it. Ranges advance by one, are empty when the start exceeds the end, and never overflow after their last element. Bounds are evaluated once, left to right.

`for item in collection` borrows the collection and gives read access to each element. `for mut item in collection` requires a mutable collection and gives exclusive access to each element in turn. Neither form consumes the collection. A loop and its body produce `Unit`; `break` cannot carry a value.

```juan
for mut enemy in enemies {
    enemy.health -= damage
}
```

Arrays, buffers and slices iterate by increasing index, maps by insertion order, and pools by occupied slot index. Structural mutation of a collection is forbidden while its iterator is live. Each element borrow ends before advancing; it cannot be retained into a later iteration.

Custom iteration uses public home-module operations `iter(collection): I` or `iter_mut(mut collection): I`, followed by `next(mut iterator: I): Option<T>`. The iterator borrows the collection; a scoped result of `next` borrows the iterator until that result's last use. These operations follow structural requirement resolution from Section 7.7. Owned iteration is explicit through `into_iter(take collection)`. Normal completion, `break`, return and traps destroy the iterator and any unconsumed owned elements.

#### Jump Statements
* `break`: Immediately breaks the current loop and may return a value only when used with `loop`.
* `continue`: Stops the current iteration of the loop and proceeds to the next cycle.
* `return`: Immediately exits the enclosing function or spawned task body and optionally returns a value.

```juan
for i in 0..10 {
    if i == 3 { continue }
    if i == 8 { break }
    io.println(i)
}
```

`break`, `continue` and `return` have the uninhabited type `Never`. `Never` may also be written as an ordinary return type for a function that never returns.

### 4.4 Match

`match` checks patterns from top to bottom and must be exhaustive unless the final reachable arm is `_`.

```juan
let description = match damage_type {
    Normal => "Normal"
    Poison => "Poison"
    Fire => "Fire"
}
```

A multi-statement arm uses a block.

```juan
match state {
    Flying => io.println("Flying")
    Fading { remaining } => {
        log(remaining)
        io.println(remaining)
    }
}
```

There is no arrow after the match value and arms do not use commas. Patterns may contain variants, record fields, literals, bindings and `_`. Every binding position may be prefixed with `mut`; the binding then follows `let mut` rules for the arm it introduces. Record shorthand accepts it in the same way: `Fading { mut remaining }`. When a match is used as a value, all reachable arms must produce compatible types.

```juan
match enemies.get_mut(handle) {
    Some(mut enemy) => enemy.health -= 10
    None => {}
}
```

A `mut` pattern binding makes the binding mutable; it does not turn shared access into exclusive access. Mutation through a `ViewMut` retains that view's existing exclusive access.

Matching an existing place borrows it. Copyable payload bindings copy their values; move-only payload bindings borrow them. A mutable payload borrow requires mutable access to the source. To consume a move-only union, use `match take value`; this consumes the complete scrutinee, transfers selected payloads and destroys any unmatched fields. Matching an owned temporary consumes it. The same rules apply to `if let`, including `if let PATTERN = take value`. The scrutinee is evaluated once.

### 4.5 `if let`

`if let` tests one pattern using the `match` pattern grammar:

```juan
if let Some(mut enemy) = enemies.get_mut(handle) {
    enemy.health -= 10
} else {
    log("stale handle")
}
```

Bindings from the pattern are scoped to the first block. When used as a value, `else` is required and both blocks must produce compatible types. When used for effects, `else` may be omitted and the expression produces `Unit`. `else if let` chains are allowed.

---

## 5. Functions

Juan modules expose functions that the host may call:

```juan
module hello

import std.io

pub fn greet() {
    io.println("Hello, World!")
}
```

Juan has no reserved entry function. `main` is an ordinary name.

Functions are free functions organized into modules. Juan does not have classes, inheritance or hidden instance methods.

Functions have expression bodies or block bodies. A block body's value follows Section 4.1.

```juan
// Expression function
fn foo(): i32 => 8

// Block function
fn foo(): i32 {
    8
}
```

### 5.1 Parameter Access & Ownership

Function signatures explicitly publish how each parameter may be accessed. The compiler still infers the operations performed by the body and verifies that they do not exceed the published contract.

* No modifier (default) — Temporary read access.
* `mut` — Temporary exclusive read/write access that may update the caller's place.
* `take` — Ownership transfers into the function.

```juan
fn inspect(player: Player) {
    io.println(player.name)
}

fn reset(mut player: Player) {
    player.health = 100.0
}

fn upload(take pixels: Buffer<u8>): Texture {
    create_texture(pixels)
}
```

A `mut` argument must be a mutable place. Passing an existing value for read or `mut` access does not consume it. A `take` argument moves a move-only value; a copyable argument supplies an owned copy. A moved place cannot be used again unless reassigned.

Two overlapping places may not receive incompatible access during the same call. Multiple reads are allowed. A write may not overlap another live read or write.

### 5.2 Function Types

Functions, union constructors and closures may be values.

```juan
fn(i32): i32
fn(mut Player, f32)
fn(take Buffer<u8>): Texture
```

Parameter modes are part of the function type. For otherwise identical types:

* A read-only `fn(T)` may be used where `fn(mut T)` is accepted.
* A mutating `fn(mut T)` may NOT be used where `fn(T)` is required.
* `take` is invariant and only matches another `take` parameter.

The static function type controls the call. Calling a value typed `fn(mut Player)` still requires a mutable `Player` even if its current function happens to be read-only.

Union constructors are ordinary noncapturing function values:

```juan
type LoadError = Io(IoError) | Parse(ParseError)

let convert: fn(IoError): LoadError = LoadError.Io
```

### 5.3 Function Value Ownership

Function types do not expose closure lifetimes or environment ownership. Named functions and union constructors are owned function values and may be stored, returned or placed inside `Rc`, `Pool` or another owner. Returning or storing a callable, or passing it to a `take` function parameter, requires an owned value; a read-only callback parameter may accept a borrowed closure for the duration of the call.

A closure that borrows a local place is scoped to that borrow and cannot escape it. An owned closure uses `move fn`.

Every referenced copyable capture is copied into an owned closure. Every referenced move-only capture is transferred into it and becomes unavailable in the outer scope.

The compiler infers invocation access separately from capture ownership:

* `fn(T): R` — Repeated calls with read access to the environment.
* `mut fn(T): R` — Repeated calls requiring exclusive access to the environment.
* `take fn(T): R` — One call consuming the environment.

A read callable may satisfy a mutable or consuming callable type; a mutable callable may satisfy a consuming callable type. The reverse conversions are forbidden. Calling a `mut fn` requires a mutable callable place; calling a `take fn` consumes it. A closure that moves a capture out during invocation is a consuming callable even when its creation uses `move fn`.

```juan
let upload_later: take fn(): Texture = move fn() {
    upload(pixels)
}
let texture = upload_later()
// Calling upload_later again is an error.
```

In function types, attributes precede the optional invocation mode: `@suspend take fn()`. This is distinct from a parameter's access mode: `take body: take fn()` transfers ownership of a callable that can be invoked once.

Owned callable slots conservatively carry runtime environment metadata, are move-only unless a copyable wrapper is used, and cannot assume thread safety from a bare `fn` signature. Converting a named function to such a slot preserves its generation retention. Direct named-function values remain copyable. Read callback parameters may borrow scoped environments but cannot retain them.

### 5.4 Closures & Inferred Captures

An anonymous `fn` expression creates a closure. Parameter types may be inferred when an expected function type is known.

```juan
let double: fn(i32): i32 = fn(value) => value * 2
```

The compiler derives captures from how outer places are used. Reading creates shared scoped access and mutation creates exclusive scoped access. Use `move fn` when the closure must own its environment.

```juan
let mut total = 0

let mut add: mut fn(i32) = fn(value) {
    total += value
}

let prefix = "Player"
let formatter: fn(str): str = move fn(name) {
    prefix + ": " + name
}
```

A borrowed closure may not be returned, stored in an owner or used after any captured place becomes invalid. The diagnostic identifies the capture that prevents escape and suggests `move fn` when ownership transfer is valid.

### 5.5 Inferred Effects & Guarantees

The compiler infers these properties through function bodies, callbacks, generic operations and destructors:

* `alloc` — Allocates heap memory.
* `block` — May block the executing host thread.
* `io` — Reads from or writes to an external system.
* `nondeterminism` — Observes time, randomness or nondeterministic external state.
* `host` — Invokes a host operation not described by a narrower effect.
* `main_thread` — Requires the runtime's designated host thread.
* `suspend` — Contains a script suspension point.

These inferred properties are stored in compiled interfaces and used for safety, scheduling and optimization.

Host bindings publish the same properties in generated interface metadata. An omitted host property is treated conservatively.

Attributes add declared constraints only when an API needs one:

* `@pure` — No externally observable mutation, I/O, blocking, nondeterminism or host operation. A pure function may allocate its result.
* `@no_heap_alloc` — Performs no heap allocation, including through calls and destructors. Arena allocation is permitted.
* `@deterministic` — Produces the same result and observable mutations for the same inputs, independent of time, randomness, scheduling and worker count.
* `@main_thread` — May run only on the runtime's designated host thread.
* `@suspend` — Marks a function type whose callable may suspend.
* `@no_suspend` — Requires a callable not to suspend. This is the default for a bare function type.

```juan
@{pure, no_heap_alloc}
fn add(a: i32, b: i32): i32 => a + b

fn apply(value: i32, operation: @no_heap_alloc fn(i32): i32): i32 {
    operation(value)
}
```

Function constraints are checked transitively. They are the only attributes allowed in function-type position, where they constrain the supplied callable. A callable satisfying more guarantees may be used where fewer are required. A main-thread-restricted callable cannot satisfy an unrestricted function type, while an unrestricted callable may be used in a main-thread-only slot. Refactoring an ordinary function affects a caller only when the caller or callback slot promises a guarantee that the change would violate.

---

## 6. Variables, Constants & Expressions

Bindings may not shadow another binding in the same scope. A nested scope may shadow an outer binding.

```juan
let my_var = 8
let mut counter = 8
```

The inferred type of both bindings is `i32`. Numeric places may be incremented or decremented:

```juan
let mut my_var = 8
my_var--
```

Postfix `++` and `--` are standalone numeric update statements, equivalent to `+= 1` and `-= 1`. Prefix forms and expression uses such as `array[i++]` are errors. They produce no value and evaluate the destination exactly once.

Operands, the call receiver, arguments and record initializers are evaluated left to right in source order. Assignment evaluates the destination place before its right-hand side. Boolean operators short-circuit. A skipped branch performs no evaluation or moves.

Integer division truncates toward zero; remainder has the dividend's sign. Integer division by zero and the minimum signed integer divided by `-1` trap. Shift counts must be non-negative and smaller than the left operand's bit width; otherwise they trap. Signed right shift extends the sign; unsigned right shift fills with zero. Left shift discards shifted-out bits and does not use arithmetic-overflow trapping.

Floating-point operations use the declared IEEE binary32 or binary64 format, including infinities and NaNs. Floating-point division by zero follows IEEE behavior. NaN compares unequal to every value, including itself; ordered comparisons with NaN are false. Implicit reassociation and fused operations that change observable results are forbidden; explicit library operations may request them.

Primitive scalars compare by value, strings by text contents, and handles by full identity. Records and tagged unions have no automatic fieldwise equality unless declared or derived. Derived equality compares corresponding fields and requires equality for every field. Ordering is not synthesized from equality.

Module constants use `const`:

```juan
module config

import std.io

const NAME: str = "Juan"

pub fn print_name() {
    io.println(NAME)
}
```

You may NOT define a mutable variable in global module scope. Constant initializers must be pure compile-time expressions and may not perform I/O, runtime allocation, environment access, clock reads, randomness or mutable-state access.

A brace block creates a value-producing nested scope:

```juan
let hp = {
    let base = 100.0
    let shield = 25.0
    base + shield
}
```

```juan
let result = input_val
    |> step_one(arg1)
    |> step_two()

// Equivalent to: step_two(step_one(input_val, arg1))
```

`|>` is left-associative, evaluates each input exactly once and inserts it as the first argument of the next call. It has lower precedence than every arithmetic, comparison, range and boolean operator.

---

## 7. Types & Declarations

Juan is statically typed and has no undefined or null value.

### 7.1 Nominal Wrappers & Aliases

A nominal type may wrap an existing type:

```juan
type Health = i32

let health = Health(100)
let raw_health = health.inner
```

`Health` and `i32` are different types and never convert implicitly. A scalar wrapper has one synthesized read-only field named `inner`.

`alias` creates a transparent second name for the same type:

```juan
alias EntityIndex = i32
```

### 7.2 Records

Record types use braces:

```juan
type Foo = {
    bar: str
}

let basic_test = Foo { bar = "Test" }
```

A record value always begins with its type name. `{ ... }` without a preceding type is a block.

If a variable has the same name as the field, you may use shorthand:

```juan
let bar = "Test"
let basic_test = Foo { bar }
```

A field can be modified only through a mutable place.

```juan
let basic_test = Foo { bar }
basic_test.bar = "Test2" // Error

let mut basic_test = Foo { bar }
basic_test.bar = "Test2" // Works
```

Fields may declare pure constant defaults using `field: Type = expression`. Construction evaluates supplied fields in written order and fills omitted fields from their defaults. Every other field is required.

Record update uses a final `..base` entry:

```juan
let options = SpawnOptions { speed = 4.0, ..defaults }
```

The base is evaluated once after explicit initializers. Copyable records copy the remaining fields; a move-only base must be transferred with `..take base`, consuming the complete base and destroying replaced fields. Update of a type with a custom destructor is rejected. Construction and update obey field visibility.

A public record may contain private or package-visible fields. If any required field is inaccessible, outside code cannot construct the record directly and must use a constructor function.

```juan
module game.player

pub type Player = {
    pub name: str,
    health: f32
}

pub fn new(name: str, health: f32): Player => Player { name, health }

pub fn health(player: Player): f32 => player.health
```

### 7.3 Copy & Move Classification

Primitive scalars, owned `str`, `Rc<T>`, `Arc<T>`, `Weak<T>`, `ArcWeak<T>`, `Handle<T>`, `Script`, named function values, union constructors and records/unions made entirely from copyable fields are copyable.

`Buffer<T>`, `Map<K, V, S>`, `Pool<T>`, `Cell<T>`, `Arena`, owned closures, resources and records/unions containing a move-only field are move-only. Assignment, storage, capture or argument passing consumes a move-only value only when ownership is transferred through `take` or another owning position.

Partial moves out of record fields are rejected. A moved binding cannot be used until it is reassigned.

The compiler also infers whether a type is trivially destructible as defined in Section 14.1.

### 7.4 UFCS

UFCS provides dot-call syntax for free functions:

```juan
import game.player

let health = plr.health()

// Exactly equivalent to:
let health = player.health(plr)
```

`value.function(args)` resolves to a matching free function in the value type's home module or one explicitly imported into the current module. The value becomes the first argument. Exactly one candidate must match.

Any value-producing expression may be the receiver, including literals, constructors, blocks and function results:

```juan
let shout = "juan".to_uppercase()
let bounded = 120.clamp(0, 100)
let length = Vec3 { x = 1.0, y = 2.0, z = 3.0 }.length()
let name = find_player(id)?.name.trim()
```

The receiver's static type determines resolution. Context constrains an unsuffixed numeric literal before resolution; without such context it uses the default numeric type. Primitive types have standard-library home modules, so their operations use the same UFCS rules as nominal types.

Field access and UFCS calls are separated syntactically:

```juan
player.health       // Always field access
player.health()     // Always UFCS
(player.callback)() // Calls a function stored in a field
```

Parameter modes and guarantees apply through UFCS as they do through a qualified call.

### 7.5 Operator Overloads

Operators are free functions.

```juan
module vec2

pub type Vec2 = {
    pub x: f32,
    pub y: f32
}

pub fn new(x: f32, y: f32): Vec2 => Vec2 { x, y }
pub fn +(a: Vec2, b: Vec2): Vec2 => Vec2 { x = a.x + b.x, y = a.y + b.y }
pub fn *(v: Vec2, s: f32): Vec2 => Vec2 { x = v.x * s, y = v.y * s }

let mut a = vec2.new(3.0, 4.0)
let b = vec2.new(3.0, 4.0)
let c = a + b

a += b + c * 2.0
a *= 0.5
```

Compound assignment always uses the base operator. `a += b` evaluates the place `a` once, calculates `a + b`, and stores the result back. Compound forms cannot be overloaded separately.

An operator overload may only be declared in the module that defines at least one nominal operand type. Import order never resolves overloads; if one best candidate cannot be found, compilation fails and lists the candidates.

Operator precedence cannot be changed. From strongest to weakest:

1. Calls, indexing, field access, UFCS and postfix `?`
2. Prefix `await`, `!`, `~` and `-`
3. `*`, `/` and `%`
4. `+` and `-`
5. `<<` and `>>`
6. Bitwise `&`
7. Bitwise `^`
8. Bitwise `|`
9. `<`, `<=`, `>`, `>=`, `==` and `!=`
10. Boolean `&&`
11. Boolean `||`
12. Ranges `..` and `..=`
13. Pipeline `|>`

Comparison operators do not chain. Assignment is a statement and is not part of the expression precedence table. Boolean short-circuiting, assignment, field access and function calls are not overloadable. `a != b` means `!(a == b)`.

Indexing may be overloadable for nominal containers, but it must produce an ephemeral place governed by the scoped-access rules in Section 8. It never returns a storable reference.

### 7.6 Tagged Unions

Tagged unions define variants with or without fields:

```juan
type DamageType = Normal | Poison | Fire
type State = Flying | Fading { remaining: f32 }
```

Variants are qualified when constructing them outside a context that already knows the union type:

```juan
let state = State.Fading { remaining = 2.0 }
```

Variants may be unqualified inside a `match` over their union type.

### 7.7 Generics & Structural Requirements

Types and functions may be generic.

```juan
pub type Result<T, E> = Ok(T) | Err(E)
pub fn identity<T>(take item: T): T => item
```

Public generic requirements are written explicitly as free-function signatures. The block follows the `where` requirements:

```juan
fn keys_equal<K, S>(strategy: S, a: K, b: K): bool
where
    @pure
    @no_heap_alloc
    fn equals(strategy: S, a: K, b: K): bool
{
    equals(strategy, a, b)
}
```

An expression-bodied generic uses `=>` after the requirement list.

Requirements are satisfied in this order:

1. An exact built-in operation.
2. A matching public operation in the home module of a participating nominal type.
3. Otherwise the requirement is unsatisfied.

Imported extension functions may be called directly or through UFCS but do NOT satisfy public structural requirements.

The generic body is checked against its published requirements. Public generic functions must list every requirement. Private generics may infer requirements, which become part of their internal compiled interface.

Juan has no associated types; use an explicit type parameter:

```juan
fn next<I, T>(mut iterator: I): Option<T>
```

Type parameters may have defaults. Defaulted parameters must follow required parameters, and callers may omit only a trailing run of them. Constant parameters declare their type: `type Batch<T, const N: usize> = { items: Array<T, N> }`.

Structural property requirements use `T: Copyable`, `T: TriviallyDestructible`, `T: Movable` or `T: Shareable` in the `where` list. These are compiler predicates, not user-defined traits. A generic body may assume only its declared properties and operations.

Scope provenance propagates through generic substitution and aggregates. A value containing a scoped field is itself scoped; a mutable borrow is never made copyable by wrapping it. An owning container rejects scoped elements unless its arena rules explicitly allow them. Compiled interfaces preserve return provenance, constraints, invocation modes and effects. Type erasure cannot discard a property that a caller needs to establish safety.

A generic returning ownership must receive ownership or require `Copyable`; read parameters do not authorize moving out. Structural function requirements specify their access, invocation and effect contracts independently of a concrete implementation.

### 7.8 Explicit Conversions

Juan does not implicitly convert between numeric or nominal types. Conversions use named functions.

```juan
let wide = i64.from(small)             // Guaranteed lossless
let checked_value = i32.try_from(large)? // Checked
let truncated = i32.truncating(decimal) // Explicit truncation
let bits = u32.wrapping_from(signed)   // Explicit wrapping
```

There is no general-purpose `as` cast in ordinary Juan code. Unsafe host/layout conversions belong behind checked interoperability functions.

### 7.9 Strings

`str` is immutable UTF-8 text. An owned `str` is atomically reference-counted; an arena `str` is scoped to its arena as described in Section 14.9. Copying an owned `str` shares its contents. String literals use static immutable storage and do not allocate each time they are evaluated.

`str` does not support integer indexing. The standard library provides byte, Unicode-scalar and text iterators.

Repeated string construction uses `StringBuilder`.

Interpolated strings have an `f` prefix:

```juan
let message = f"Player {player.name}: {score}"
```

Each braced expression is evaluated once, left to right, and formatted through the standard formatting operation for its static type. `{{` and `}}` produce literal braces. Plain strings do not interpolate. Interpolation returns an owned `str` and conservatively has the heap-allocation effect; arena formatting remains explicit through `arena.format`.

### 7.10 Unit Values

`ByteSize` and `Duration` are compiler-known nominal value types. Their literals include a unit suffix with no intervening whitespace:

```juan
let arena_size: ByteSize = 1mb
let timeout: Duration = 250ms
let loading_timeout = 1min + 30s
```

Byte-size suffixes are lowercase and use binary multiples:

* `b` — Bytes.
* `kb` — 1,024 bytes.
* `mb` — 1,048,576 bytes.
* `gb` — 1,073,741,824 bytes.

Juan has no bit-size literal suffixes, so `kb` always means kilobytes rather than kilobits.

Duration suffixes are:

* `ns` — Nanoseconds.
* `us` — Microseconds.
* `ms` — Milliseconds.
* `s` — Seconds.
* `min` — Minutes.
* `h` — Hours.

`ByteSize` stores a `u64` count of bytes and `Duration` a `u64` count of nanoseconds. Both are copyable and trivially destructible. Arena capacities must additionally fit the target's addressable allocation size. A unit literal may use a decimal fraction only when scaling it produces an exact whole number of the base unit:

```juan
1.5mb  // Valid: 1,572,864 bytes
0.25ms // Valid: 250,000 nanoseconds
0.1ns  // Error: smaller than one nanosecond
```

Overflow and inexact unit literals are compile errors. Literal scaling uses exact decimal arithmetic, never an intermediate floating-point value.

Values of the same unit type support comparisons, addition and subtraction. Overflow and negative results trap. Multiplication by `u64` returns the unit type with overflow checks. Division by `u64` returns the unit type, truncating toward zero in base units: `1s / 3` is `333_333_333ns`. Division of two values of the same unit type returns an `f64` ratio: `1s / 250ms` is `4.0`. Any zero divisor traps. Other mixed-unit operators are errors.

There are no implicit numeric conversions. `ByteSize.from_bytes(u64)` and `Duration.from_nanos(u64)` construct exact values; `bytes()` and `nanos()` expose their counts. `Duration.seconds_f64()` explicitly converts to approximate seconds for physics and animation math. `Duration.try_from_seconds(f64)` rejects negative, non-finite, out-of-range or fractional-nanosecond results.

---

## 8. Arrays, Buffers, Maps & Scoped Access

### 8.1 Scoped Borrows

`View<T>`, `ViewMut<T>`, `Slice<T>`, `SliceMut<T>` and lock views are scoped borrows. They may be returned from an operation, but cannot outlive the storage they borrow. The compiler infers the one source place from which a returned borrow derives and records that parameter position in the compiled interface. A function whose returned borrow could derive from multiple parameters is rejected.

Borrow scopes end at the last use when possible. A shared borrow may overlap other shared borrows. An exclusive borrow may not overlap any other access to the same storage. A scoped borrow may not be stored globally, placed in an `Rc`, `Arc`, `Pool` or heap container, captured by an owned closure, kept across `await`, or remain live while its owner is moved, resized or destroyed. Sections 14.9 and 14.10 define same-arena storage and the limited suspension exceptions for script-owned storage and helper parameters.

Field access, operators and UFCS automatically operate through views. A generic value may contain a scoped type only while the compiler preserves its source provenance; an unconstrained generic result may not erase or extend that scope.

Indexing may create an ephemeral place for the enclosing expression:

```juan
positions[i].x += velocity.x
```

That place may be read, updated or passed to one immediate call, but it cannot be stored or returned.

### 8.2 Maps

`Map<K, V, S>` is a typed dictionary with deterministic insertion-order iteration. `S` is its key strategy type.

The standard canonical form uses its default strategy and is written as `Map<K, V>`. A custom form keeps the strategy explicit:

```juan
type CaseInsensitive = {}

@pure
@no_heap_alloc
@deterministic
fn equals(strategy: CaseInsensitive, a: str, b: str): bool

@pure
@no_heap_alloc
@deterministic
fn hash(strategy: CaseInsensitive, value: str): u64

let users: Map<str, User, CaseInsensitive> = map.new(CaseInsensitive {})
```

A strategy may contain state such as a randomized hash seed, but all values of one strategy type must define the same key-equivalence relation. Equal keys must produce equal hashes for the same strategy value.

Map access uses:

* `get_copy(key)` — Returns `Option<V>` and is available only when `V` is copyable.
* `get(key)` — Returns `Option<View<V>>`.
* `get_mut(key)` — Returns `Option<ViewMut<V>>` and requires mutable access to the map.
* `remove(key)` — Returns `Option<V>` and transfers the stored value out.

```juan
let score = scores.get_copy("Davey")

let score_view = scores.get("Davey")?
let description = "Score: " + score_view.to_string()

let mut mutable_score = scores.get_mut("Davey")?
mutable_score += 100
```

The map cannot be structurally changed while any entry view is live. A mutable entry view reserves exclusive access to the map until its last use.

Updating an existing key keeps its iteration position. Removing and reinserting it places it at the end.

### 8.3 Buffers & Arrays
* `Array<T, N>` — Fixed-size inline storage. It may live in a local, another type or heap allocation; it is not guaranteed to be on the stack.
* `Buffer<T>` — Move-only, dynamically growing owned heap storage.
* `Slice<T>` — Scoped read access to contiguous elements.
* `SliceMut<T>` — Scoped exclusive read/write access to contiguous elements.

Array literals use `[a, b, c]` and produce `Array<T, N>`. Elements are evaluated left to right, copied or moved into the array, and must share one element type. `[]` requires context for `T`. Repetition uses `[value; N]`, where `N` is a constant `usize`; it evaluates `value` once and requires a copyable element. The semicolon is only a delimiter inside repetition literals, never a statement separator.

```juan
let checkpoints = [start, bridge, finish]
let counters: Array<i32, 8> = [0; 8]
let items = Buffer.from_array([first, second])
```

Indexing uses `usize` indices and bounds-checks access. Reading a copyable element copies it; moving an element out requires a container removal operation. A range index `items[start..end]` yields a read `Slice<T>`, with omitted bounds defaulting to zero and length. Invalid ranges trap. `slice_mut(start..end)` returns an exclusive mutable range; `slice_mut()` selects the whole collection.

A buffer returns scoped slices directly:

```juan
let read_slice = positions.slice()
process(read_slice)

let mut write_slice = positions.slice_mut()
update(write_slice)
```

The buffer cannot resize, move or be destroyed while a slice is live. Buffer bounds are checked unless proven valid. Ownership follows the parameter and return rules in Section 5.1.

---

## 9. Error Handling

Optional values use `Option<T>` and recoverable failures use `Result<T, E>`.

### 9.1 Option<T>

An `Option<T>` is `Some(T)` or `None`. Postfix `?` extracts `Some(value)` or returns `None` from the enclosing `Option`-returning function. It performs no conversion to `Result`. A script body returning `Unit` or `Result` must handle `None` explicitly or convert it with `ok_or(error)` before using `?`.

### 9.2 Result<T, E>

Postfix `?` extracts `Ok(value)`. If it receives `Err(error)`, it immediately returns that exact error type from the current function or spawned task body.

```juan
fn load_player_config(path: str): Result<Config, LoadError> {
    let content = read_file(path)
        .map_error(LoadError.Io)?

    let config = parse_json(content)
        .map_error(LoadError.Parse)?

    Result.Ok(config)
}
```

Juan does not perform hidden error conversions. `map_error` accepts a function value, and tagged-union constructors such as `LoadError.Io` work directly.

Bounds failures, failed runtime safety checks and violated runtime invariants trap. Integer arithmetic overflow and integer division by zero trap in every build profile; explicit `checked_*`, `wrapping_*` and `saturating_*` operations provide alternate behavior.

---

## 10. Resources & Deterministic Destruction

Files, GPU resources, sockets, subscriptions and other external resources use deterministic destruction.

A move-only resource type may designate exactly one destructor using the built-in `@drop` attribute:

```juan
@drop
fn close(take file: File) {
    host.close_file(file.handle)
}
```

The standard `drop(take value)` operation destroys a value immediately. Ownership transfer passes cleanup responsibility to the new owner. Reassigning an initialized owning place first evaluates the replacement, then destroys the old value, then installs the replacement.

A custom `@drop` body runs once before automatic field cleanup and can inspect the value's initialized fields. Its special consuming parameter is not recursively destroyed by the destructor's own exit. Fields are then destroyed in reverse declaration order; the destructor cannot move out or explicitly destroy those fields.

Destructors cannot suspend. If a destructor traps, remaining initialized fields and outer owners are still cleaned up; additional destructor traps are attached to the original failure rather than restarting unwinding. Fatal host termination is outside this cleanup guarantee.

Destruction occurs exactly once in reverse ownership order on normal return, early return, `?` propagation, trap unwinding and script cancellation. Partial initialization and failed construction destroy only the fields that became initialized.

Destructor effects are inferred and stored in the owning type's compiled interface. An `@no_heap_alloc` function may own a local only when every destructor that can run on its exit paths is also heap-allocation-free, or when ownership is transferred before the exit.

Resource-owning values may be stored in ordinary owners, `Rc`, `Arc`, `Pool` and owned closures; they are destroyed when their last owner is destroyed. They may not be stored in an arena. Strong reference cycles are the exception described in Section 14.1.

---

## 11. SIMD Operations

Juan provides fixed-width SIMD vector and mask types as compiler-known primitives. A target without matching hardware instructions must preserve the same behavior using narrower vectors or scalar operations.

### 11.1 Built-in SIMD Types
* `f32x4`, `f32x8`
* `i32x4`, `i32x8`
* `u8x16`, `u8x32`
* `mask32x4`, `mask32x8`, `mask8x16`, `mask8x32`

### 11.2 Construction & Operators

Vector types support element-wise operations with standard arithmetic operators (`+`, `-`, `*`, `/`). Their lane counts are part of their types and never change based on the target machine.

```juan
import std.simd

fn process_quad(a: f32x4, b: f32x4): f32x4 {
    a * b + f32x4.splat(1.0)
}
```

### 11.3 Lane Masking & Selection

Conditional vector logic uses lane masks and selection.

```juan
import std.simd

fn clamp_lower(v: f32x4, min_val: f32): f32x4 {
    let limit = f32x4.splat(min_val)
    let mask: mask32x4 = v.greater_than(limit)
    simd.select(mask, v, limit)
}
```

---

## 12. Attributes & Contracts

Attributes are compiler metadata or obligations attached to an existing declaration. Attributes are NOT automatically macros.

```juan
@attribute_name
@attribute_name(argument)
@attribute_name(key = value)

@{pure, no_heap_alloc, deterministic}
```

An attribute group applies several attributes to the same target. Group members omit their individual `@`, are comma-separated and may use a trailing comma in multiline form:

```juan
@{
    requires(amount >= 0),
    ensures(result >= 0),
    no_heap_alloc,
}
fn calculate(amount: i32): i32 {
    amount
}
```

`@{a, b}` is equivalent to separate `@a` and `@b` prefixes in that source order. Attribute macros inside a group expand in source order. Groups are valid everywhere a single attribute is valid, including function-type position. Empty groups and repeated attributes that do not explicitly support repetition are errors.

Unknown attributes are compile errors.

### 12.1 Built-in Attributes

* `@requires(condition)` — Every caller must establish the precondition.
* `@ensures(condition)` — The condition must hold on every applicable return.
* `@invariant(condition)` — Defines a type or loop invariant.
* `@decreases(expression)` — Supplies a termination measure for recursive or verified code.
* `@trusted(reason)` — Creates an explicitly reviewed verification boundary.
* `@layout(c, align = N)` / `@layout(c, packed)` — Controls eligible record layout as defined in Section 12.2.
* `@drop` — Marks the one deterministic destructor for a resource type.
* `@deprecated(message)` — Produces use-site warnings.
* `@pure` — Requires a function or callable type to have no externally observable effects.
* `@no_heap_alloc` — Requires a function or callable type to perform no heap allocation.
* `@deterministic` — Requires deterministic behavior for equal inputs.
* `@main_thread` — Restricts a function or callable type to the designated host thread.
* `@suspend` — Marks a function type whose callable may suspend.
* `@no_suspend` — Requires a callable not to suspend.
* `@const` — Marks a pure function as valid in constant evaluation.
* `@syntax(kind)` — Declares a syntax macro of the specified kind.
* `@derive(names...)` — Applies imported declarative derives to a type.

Built-in attribute names are reserved and do not require imports. Each built-in defines its legal targets and argument grammar. A package may not shadow one.

Function-constraint attributes may decorate function declarations. `@pure`, `@no_heap_alloc`, `@deterministic`, `@main_thread`, `@suspend` and `@no_suspend` may also appear before a function type or structural function requirement as part of that type.

### 12.2 Record Layout

`@layout(c)` preserves declaration order using the target C ABI's field sizes and alignment. Eligible fields are fixed-width numeric primitives, fixed arrays of eligible elements, and other explicitly laid-out eligible records. Counted values, resources, handles, callable values, scoped values and implicit tagged-union layouts are ineligible.

```juan
@layout(c, align = 16)
type Vertex = {
    position: Array<f32, 3>,
    weight: f32
}

@layout(c, packed)
type Header = {
    kind: u8,
    length: u32
}
```

`align = N` requires a supported positive power of two and raises minimum record alignment. `packed` preserves field order, removes inter-field padding and sets record alignment to one. The modifiers cannot be combined. Nested record layouts remain unchanged. A packed record cannot contain an explicitly over-aligned record, directly or through nested fields.

Padding and trailing size follow the selected representation. Packing does not specify byte order or a wire format. A layout change is an incompatible hot-reload change.

Primitive packed fields support copied reads and writes through compiler-generated unaligned access. Forming a typed view or passing an insufficiently aligned field through ordinary read or `mut` access is rejected; copy it into an aligned local first. There is no implicit copy-back for a mutable argument.

`size_of<T>()` and `align_of<T>()` return constant `ByteSize` values. `offset_of<T>(field)` returns a constant byte offset for an accessible field of an explicitly laid-out record. These operations expose no pointer and do not authorize reading padding bytes.

### 12.3 Contract Semantics

Contract expressions are pure. They may not mutate, allocate runtime objects, perform I/O, read clocks or randomness, or call functions whose effects are incompatible with specification evaluation.

`old(expression)` denotes a value captured before the function body executes. The expression must be specification-pure and copyable without heap allocation; resource owners and live borrows are ineligible. In `@ensures`, `result` names the returned value and is read-only. Postconditions run after return-value evaluation and before local destruction; they do not run on traps or cancellation.

Contracts are checked at runtime unless the compiler proves them. A proven check may be omitted without changing the contract's meaning. Failed checks identify the contract, source location and relevant values.

Runtime quantifiers must have finite bounded domains. Floating-point contracts use Juan's ordinary IEEE behavior, including explicit handling of NaN when relevant.

Declared parameter modes provide the initial frame condition: a function may only modify state reachable through `mut` parameters or explicitly authorized host effects.

---

## 13. Macros

Juan macros are hygienic declarative syntax transformations. They match structured syntax and emit Juan syntax trees. Constant evaluation computes values without transforming syntax. Procedural macros and arbitrary compile-time programs are not part of Juan.

### 13.1 Declaration Macros

A declaration macro uses `fn` with the built-in `@syntax(declaration)` attribute:

```juan
@syntax(declaration)
pub fn event($name: identifier, $fields: field*) => {
    type $name = {
        $fields
    }
}
```

Inside an `@syntax` declaration only:

* `$name` declares or splices a metavariable.
* `identifier`, `type`, `expression`, `pattern`, `field`, `declaration` and `token_tree` are compiler-known syntax categories.
* `*`, `+` and `?` mean repeated, one-or-more and optional captures.
* `=>` separates the matcher from the syntax-tree template.

This is not an executable function and has no runtime function type.

After explicitly importing the macro:

```juan
import { event } from awesome_events.syntax

event Damage {
    source: Entity,
    target: Entity,
    amount: f32
}
```

It expands into an ordinary `Damage` record declaration. Macros extend declaration boundaries; core expression, statement, pattern and type grammar remain closed.

### 13.2 Attribute & Derive Macros

An imported attribute macro receives the parsed declaration it decorates and may validate it or emit additional declarations.

```juan
import { serialize, replicate } from awesome_net.syntax

@serialize
@replicate(rate = 20)
type PlayerSnapshot = {
    position: Vec3,
    health: u16
}
```

`@derive` is a built-in dispatcher for imported declarative derives:

```juan
import { equality, serialization } from std.derive

@derive(equality, serialization)
type Player = {
    name: str,
    health: Health
}
```

Derives emit free functions and declarations.

### 13.3 Parsing Without Executing Macros

Tooling must parse a macro invocation without running user code. At a declaration boundary, an explicitly imported syntax name followed by tokens becomes a generic `SyntaxInvocation` node.

The delimiter scanner respects balanced `()`, `[]` and `{}`. A macro's compiled interface exposes a declarative parsing schema that the editor may inspect without executing expansion code.

If a tool lacks the macro interface, it preserves the invocation's balanced delimiters as an opaque token tree.

### 13.4 Hygiene, Resolution & Expansion

* Macro names are lexically scoped and explicitly imported.
* Literal names created by a template receive definition-site identity.
* Captured names retain call-site identity and source spans.
* Templates cannot request implicit call-site capture.
* Macros may emit declarations and attributes.
* Macros may not emit imports, module declarations or new macro definitions.
* Expansion happens after parsing and macro-import discovery, but before ordinary name resolution and type checking.
* Expansion order is source order with bounded rounds and a finite recursion limit.
* Diagnostics show the invocation, generated failing node and expansion trace.

Attribute macros expand top to bottom. Built-in non-expanding attributes remain attached to the declaration and cannot have their meaning removed by a macro. Generated attributes expand in a later bounded round.

### 13.5 Capability Boundary

Macro expansion has no filesystem, network, environment-variable, clock, randomness, process or arbitrary host access. It may inspect captured syntax and immutable compiler facts such as the target profile.

Equal macro input, imported interfaces and compiler configuration must produce equal syntax trees. Expansion has instruction, memory, output-size and recursion budgets.

---

## 14. Memory, Concurrency & Hot Reload

### 14.1 Ownership, Shared Values & Handles

Juan has no tracing garbage collector. Heap values use deterministic ownership or reference counting. Every value has an inferred ownership class, and destruction never depends on a tracing memory pass.

#### Ownership Classes

Every value is one of:

* **Copyable** — Copied implicitly. Primitive scalars, owned `str`, `Rc<T>`, `Arc<T>`, `Weak<T>`, `ArcWeak<T>`, `Handle<T>`, `Script`, named function values, union constructors and records/unions made entirely from copyable fields.
* **Move-only** — Transferred through `take` or another owning position. `Buffer<T>`, `Map<K, V, S>`, `Pool<T>`, `Cell<T>`, `Arena`, owned closures, resources and records/unions containing a move-only field.
* **Scoped** — Borrows described in Section 8.1, including values allocated in an arena. Never owned; cannot outlive their source.

The ownership class belongs to the value, not only its type name. An ordinary `str` is counted; a `str` produced by an arena is scoped to that arena and owns nothing. A scoped `str` cannot be stored where an owned `str` is required.

#### Trivially Destructible Values

A type is **trivially destructible** when destroying it requires no destructor, reference-count decrement or resource cleanup. Scalars, `ByteSize`, `Duration`, `Handle<T>`, union constructors, SIMD types, fixed arrays of trivial elements and records/unions made only from trivially destructible fields qualify. `str`, `Rc`, `Arc`, `Weak`, `ArcWeak`, `Script`, named function values, containers, closures and resources do not.

A named function value is copyable but retains the bytecode generation that created it. Copying it retains that generation; destroying it releases the reference. Release builds contain one generation and elide this counting. Union constructors contain only stable type and variant identifiers and remain trivial.

This property is stored in compiled interfaces and governs what may live in an arena.

#### Destruction

A move-only value is destroyed exactly once at the end of its owner's scope, in reverse declaration order, on normal return, early return, `?` propagation, trap unwinding and script cancellation. A record destroys its fields in reverse declaration order. `@drop` destructors run as part of this order.

A counted allocation is destroyed when its last strong owner is destroyed. Releasing the last reference to a large structure destroys that structure during the release; hosts that need bounded frame time should defer such releases or use an arena. Deterministic destruction is guaranteed for every value not trapped in a strong reference cycle.

#### Strings

`str` is immutable UTF-8 text. An owned `str` has an atomic reference count. Copying it retains the allocation, and the last handle frees the bytes. String literals are static and are never counted or freed. Owned strings are shareable across tasks.

#### Shared Values

`Rc<T>` is a counted, immutable value confined to one runtime thread. `Arc<T>` is its cross-task form and requires `T` to be shareable and safe to destroy on any runtime worker. A thread-affine resource cannot be placed in an `Arc` unless its host wrapper arranges safe deferred destruction.

```juan
let config = Rc.new(load_config())
let same_config = config
let value = config.max_players
```

Field access, UFCS and operators operate through `Rc<T>` and `Arc<T>` as through a `View<T>`. Their contents may include resources; `@drop` runs when the last strong owner is destroyed.

`Weak<T>` observes an `Rc<T>` without owning it; `ArcWeak<T>` observes an `Arc<T>`. `upgrade()` returns `Option<Rc<T>>` or `Option<Arc<T>>` and is `None` after the allocation is destroyed. `Weak<T>` is confined to its runtime thread; `ArcWeak<T>` follows its `Arc<T>` thread constraints.

#### Reference Cycles

A strong cycle through `Rc` or `Arc` is never freed and its destructors never run. It is a compile error for a type to reach itself through a path containing at least one strong counted edge and no `Weak<T>`, `ArcWeak<T>` or `Handle<T>`. The diagnostic prints the path and recommends `Pool<T>` with `Handle<T>`.

Recursion through unique ownership alone is legal:

```juan
type Node = {
    children: Buffer<Node>
}

type Shared = {
    children: Buffer<Rc<Shared>> // Error: possible strong cycle
}
```

Owned closure environments are checked as records of their capture types within a package. Capture types erased behind a package-boundary `fn` value can still conceal a runtime cycle. Development runtimes can detect and report such cycles on demand and at shutdown; this diagnostic traversal is not part of normal execution.

#### Mutation Through Shared Values

`Cell<T>` provides runtime-checked exclusive access to a value inside `Rc<T>`:

```juan
let shared = Rc.new(Cell.new(Counter { hits = 0 }))

let mut counter = shared.get_mut()
counter.hits += 1
```

`get()` returns `View<T>` and `get_mut()` returns `ViewMut<T>`. Overlapping a mutable view with any other live view of the same cell traps. `Cell<T>` is not shareable; cross-task mutation uses `Arc<Mutex<T>>`, `Arc<RwLock<T>>` or `Arc<Atomic<T>>`.

#### Pools & Handles

`Pool<T>` owns values addressed by generational `Handle<T>`:

```juan
let mut enemies: Pool<Enemy> = Pool.new()
let handle = enemies.insert(Enemy { health = 50 })

if let Some(mut enemy) = enemies.get_mut(handle) {
    enemy.health -= 10
}

enemies.remove(handle)
enemies.get(handle) // None
```

Pool operations are:

* `insert(take value): Handle<T>`
* `get(handle): Option<View<T>>`
* `get_mut(handle): Option<ViewMut<T>>` — Requires mutable access to the pool.
* `remove(handle): Option<T>` — Transfers the value out.
* `contains(handle): bool`

A handle contains a runtime-unique pool identity, slot index and 32-bit slot generation. Pool identities are never reused during the runtime's life. A handle presented to another pool yields `None`. A slot is permanently retired before its generation would wrap, so a stale handle can never identify a new value. Handles never keep their targets alive.

The pool cannot be structurally changed while an entry view is live. A mutable entry view reserves exclusive access until its last use. Handles are the intended representation of gameplay identities across frames, scripts, tasks and the host boundary.

#### Closures

An owned closure created with `move fn` is move-only, owns its captures and retains its code generation. Storing it in several places uses `Rc<fn(...)>`. A borrowed closure follows Section 8.1.

#### Allocation

Heap allocation has the inferred `alloc` effect. Ownership transfer, counted-handle copies and handle lookups do not allocate. A frame costs its executed work and destructors. Allocation failure traps.

### 14.2 Parallel Execution

A runtime has one designated host thread and a reusable work-stealing pool of OS threads. Juan tasks from the same runtime may execute simultaneously on different CPU cores. The host configures the maximum worker count.

The scheduler may execute work sequentially when parallel execution would cost more. Pure or disjoint-memory computations produce the same result for every worker count; atomics, channels, nondeterministic effects and order-sensitive host operations may observe the unspecified schedule.

### 14.3 Parallel Iteration

`par for` divides an iteration space into jobs, schedules them across the worker pool and joins them before continuing:

```juan
par for mut enemy in enemies {
    enemy.position += enemy.velocity * delta
}
```

The iterable is evaluated once. Ranges, arrays, buffers and slices are splittable. Host types may expose a checked splittable iteration binding.

* `par for item` gives each job read access to its element.
* `par for mut item` gives each job exclusive access to a distinct element place and requires a mutable iterable.
* Every element is visited exactly once, but execution order is unspecified.
* The body produces `Unit`. `continue` affects one iteration; `break` and control flow out of the enclosing function are forbidden.
* Captured reads may overlap. Captured writes must be disjoint from every other job or use a synchronized type.

The scheduler chooses job count and grain size. Nested parallel work reuses the same pool rather than creating OS threads recursively.

### 14.4 Parallel Blocks

Each top-level item in a `parallel` block is a child task. All items must produce `Unit`, and the block joins every child before returning `Unit`:

```juan
parallel {
    update_physics(world.physics)
    update_ai(world.ai)
    update_animation(world.animation)
}
```

The compiler derives each child's access from parameter modes and direct place usage. Multiple reads may overlap; a write may not overlap another read or write. Disjoint fields and disjoint indexed regions may execute simultaneously.

The children have no source-order execution guarantee. A trap cancels unfinished siblings, waits for them to stop at task safe points and then propagates.

### 14.5 Structured Tasks

`task` creates a structured task scope. `spawn { ... }` starts a child task and returns a scoped move-only `Task<T>`. Prefix `await` consumes the handle and produces its result:

```juan
let world_data = task {
    let terrain = spawn {
        generate_terrain(seed)
    }

    let navigation = spawn {
        build_navigation(snapshot)
    }

    WorldData {
        terrain = await terrain,
        navigation = await navigation
    }
}
```

Spawn captures are inferred as in Section 5.4. Scoped reads and writes are valid because every child finishes before the task scope exits. `move spawn { ... }` transfers every referenced move-only value into the child and copies referenced copyable values. Overlapping task access is rejected as in a `parallel` block.

The spawn body is an implicit function body: its final expression is the task result, and `return` or `?` exits that task rather than the enclosing function. `await` suspends the current Juan task without blocking its OS worker.

A `Task<T>` cannot leave its task scope, be stored in an `Rc` or `Pool`, or be awaited outside its creating scope. Exiting the scope joins unawaited children and discards their results. A trap cancels and joins sibling tasks before propagating. Juan has no detached task syntax.

### 14.6 Thread Safety & Shared Memory

The compiler derives two structural properties for cross-thread values:

* **Movable** — Unique ownership may transfer to another task. All owned fields must also be movable.
* **Shareable** — Multiple tasks may read the value simultaneously. All reachable mutable state must be synchronized or inaccessible.

Primitive immutable values, `Handle<T>`, owned `str`, named function values, union constructors and records/unions of shareable fields are shareable. `Arc<T>` and `ArcWeak<T>` are shareable when their element type is shareable. `Rc<T>`, `Weak<T>`, `Script` and arena-scoped values are neither movable nor shareable across tasks. Owned arenas may not transfer while derived values are live. Buffers and resources are movable when their elements or host declarations permit transfer. Scoped reads and writes may cross worker threads only inside a structured region that joins before their source scope exits.

The standard synchronized types are `Atomic<T>`, `Mutex<T>`, `RwLock<T>` and `Channel<T>`. Mutable state shared by tasks must be inside one of these types. `Mutex.lock()` and `RwLock.write()` return scoped mutable lock views; `RwLock.read()` returns a scoped read lock view:

```juan
let mut state = game_state.lock()
state.score += 1
```

The guard releases at its enclosing block's exit, including `return`, `?` and traps, or on explicit `drop(guard)`. Its last data access does not release the lock. A lock view cannot escape, remain live across `await`, spawn or blocking work, or overlap acquisition of another lock; Juan code may hold only one lock at a time. Atomic operations are sequentially consistent unless an explicitly named operation selects another ordering.

Channels copy copyable values and transfer move-only values. Closing a channel wakes blocked receivers with `Option.None`; sending to a closed channel returns an error.

Runtime-owned values never span separate runtimes. Cross-runtime communication copies immutable data, transfers supported owned host values, serializes messages or uses thread-safe opaque host handles.

Functions inferred or declared `@main_thread` cannot run in `par for`, `parallel` children or spawned tasks. A host operation is callable on workers only when its binding is not main-thread-restricted. Juan's guarantees do not cover unsound host implementations behind those declarations.

### 14.7 VM Hot-Reload Generations

Hosts may omit hot reload.

Hot reload replaces VM bytecode. Ahead-of-time native modules are produced only during the host build and are not Juan hot-reload targets.

Each loaded bytecode module belongs to a generation:

* Existing frames finish using the generation they entered.
* Existing closures and function values stay bound to the generation that created them.
* New exported calls use the newest compatible generation.
* Type-layout or incompatible signature changes require explicit state migration or a clean runtime restart.

A retired generation remains pinned by active frames, running scripts, registered callbacks, queued tasks, and closures or function values stored in any live owner. Every named function value and owned closure holds a generation reference; the containing module, active frames, running scripts and registration sets hold references as well. A generation may be unloaded when its reference count reaches zero and the runtime reaches an unload-safe point.

The runtime reports which owners prevent unloading and may warn or restart a development runtime when retired-generation memory exceeds a configured budget.

### 14.8 Initialization & Registration

General initialization and reloadable callback registration are separate operations.

* Initialization runs once when creating a runtime and may create persistent state.
* Registration declares generation-owned callbacks, systems, commands or UI extensions.
* Migration converts explicit persistent state between incompatible schemas.

Registration receives a capability-limited `Registrations` value and cannot spawn gameplay entities or perform unrelated startup work.

```juan
fn register(mut registrations: Registrations) {
    registrations.on_update(UpdateRegistration {
        phase = UpdatePhase.Gameplay,
        priority = 0,
        callback = update
    })
}
```

Reload builds a new registration set transactionally. If registration fails, the old set stays active. On success, at a quiescent host boundary the runtime cancels and cleans up scripts owned by the old registration set, then atomically replaces the registrations. No old or new registration callback runs during that transition.

Registration execution order is deterministic:

1. Explicit phase.
2. Lower numeric priority first.
3. Fully qualified registration identity as the tie-breaker.

### 14.9 Frame Arenas

An `Arena` is a fixed-capacity bump allocator. Its backing storage is allocated once when the arena is created. Allocation within that storage advances a pointer and never accesses the heap. Resetting or destroying the arena frees all its values together without running individual destructors.

Bump allocation uses shared arena access and may occur while earlier values from that arena remain live because their locations never change. `reset`, `reserve` and destruction require exclusive access and no live derived value.

* Bump allocation within capacity has no `alloc` effect and is allowed under `@no_heap_alloc`.
* Exhaustion traps and never falls back to the heap.
* `reset()` retains the backing storage.
* `reserve(additional: ByteSize)` explicitly grows the backing storage and has the `alloc` effect. No value derived from the arena may be live when it is called.

#### The Host Frame

Before a frame begins, the host configures the `ByteSize` capacities of the frame arena and every worker slice. `begin_frame(dt: Duration)` opens the frame and `end_frame()` closes it. During an open frame, `frame` is a built-in scoped borrow of the host's frame arena. Accessing it outside a frame traps.

```juan
fn draw_names(world: World) {
    let mut names = Buffer.new_in(frame)

    for entity in world.visible() {
        names.push(frame.format("{} ({})", entity.name, entity.id))
    }

    debug.draw_list(names.slice())
}
```

`end_frame()` resets the arena. The compiler prevents every value scoped to that frame from surviving it. Exhaustion reports the allocation site and configured capacity.

#### Arena Values

Standard containers and string operations provide arena forms:

* `Buffer.new_in(arena)`, `Map.new_in(arena)` and `StringBuilder.new_in(arena)`
* `arena.format(...)`, `arena.concat(a, b)` and `arena.copy_str(text)`

The resulting values are scoped to the arena. They may not be stored in globals, counted owners, pools, heap containers or owned closures. They may be returned only from a function that received their source arena as a parameter.

```juan
fn describe(arena: Arena, player: Player): str =>
    arena.format("{} hp={}", player.name, player.health)
```

An arena string converts to an owned `str` only through `to_owned()`, which allocates on the heap. Growing an arena buffer allocates a new region in the same arena and relocates its elements; live element views or slices forbid that growth. Superseded regions remain reserved until reset.

#### Element Rules

Every value physically owned by an arena container, including elements, map strategies and other metadata, must be trivially destructible or scoped to the same arena.

```juan
let entity_handles: Buffer<Handle<Entity>> = Buffer.new_in(frame) // Allowed
let positions: Buffer<Vec3> = Buffer.new_in(frame)                // Allowed
let labels: Buffer<str> = Buffer.new_in(frame)                    // Allows frame strings
let configs: Buffer<Rc<Config>> = Buffer.new_in(frame)            // Error: count decrement
let files: Buffer<File> = Buffer.new_in(frame)                     // Error: resource cleanup
```

#### User Arenas

`Arena.new(capacity: ByteSize)` creates an owned, move-only arena and performs one heap allocation:

```juan
let mut level_memory = Arena.new(1mb)
let tiles: Buffer<Tile> = Buffer.new_in(level_memory)

level_memory.reset() // Error: tiles is used below
consume_tiles(tiles)
```

`reset()`, `reserve()` and destruction require that no derived value is live.

#### Arenas & Scripts

Values belonging to `frame` or an arena merely borrowed by a script may not remain live across a script suspension. Values from an arena owned by the script's coroutine frame may do so.

The owned arena must be declared before its derived values so reverse destruction order remains valid. Once a scoped relationship exists inside a coroutine frame, that frame remains at a fixed address for its lifetime.

```juan
script.start(move fn() {
    let mut mission_memory = Arena.new(64kb)
    let route: Buffer<Vec3> = Buffer.new_in(mission_memory)

    wait(5s)
    follow(route.slice())
})
```

#### Threads

Each worker thread has its own frame-arena slice, and all slices reset together at `end_frame()`. Arena values are neither shareable nor movable across tasks.

#### Allocation Guarantee

`@no_heap_alloc` forbids heap allocation through the function, its calls and its destructors, but permits bump allocation inside an existing arena. `Rc.new`, `Buffer.new`, `Arena.new`, `Arena.reserve`, `script.start` and `to_owned()` violate it. `Buffer.new_in(frame)` and `frame.format` do not. Because arenas never grow implicitly, exhaustion traps rather than weakening the guarantee.

### 14.10 Scripts

A script is a frame coroutine resumed by the runtime once per tick until it finishes. Scripts express missions, cutscenes, AI behaviours, timed sequences and UI flows. Their sequential portions run on the designated host thread.

#### Starting a Script

```juan
import std.script

fn on_level_start() {
    let _ = script.start(move fn() {
        play_cutscene("intro")
        wait_until(move fn() => cutscene.is_done())

        spawn_enemies(5)
        wait(2s)

        objective.show("Clear the area")
        wait_until(move fn() => enemies_alive() == 0)

        objective.complete()
    })
}
```

The built-in `script.start` accepts either `take body: @suspend take fn()` or `take body: @suspend take fn(): Result<Unit, E>`. The selected return type is inferred from the body. Error values must be owned and support the standard diagnostic-formatting operation; generated adapters retain their type identity and diagnostic text for the host.

Starting a script consumes its body and allocates pinned coroutine state, with the `alloc` effect. A non-suspending body is accepted and finishes on its first tick. `script.start_with` and `script.start_child` accept the same two body forms.

`ScriptOptions` has fields `phase: UpdatePhase`, `priority: i32 = 0` and `cancel_with_parent: bool = false`. `script.start` inherits the caller's phase, or the host's default script phase outside a callback. `script.start_with(options, body)` selects a phase and priority explicitly:

```juan
script.start_with(ScriptOptions {
    phase = UpdatePhase.Gameplay,
    priority = 10
}, move fn() {
    update_objective()
})
```

The body's final expression must be `Unit` or `Result<Unit, E>`. `return` follows the body's selected return type; a bare return is valid only for `Unit`. `?` in a `Result` body propagates its error to the host's script-error callback together with script identity and trace.

#### Suspension Points

These operations stop the current script slice and resume it on a later tick:

* `yield_frame()` — Resume on the next tick.
* `wait(duration: Duration)` — Wait on the scheduler's scaled game clock.
* `wait_realtime(duration: Duration)` — Wait using unscaled wall time.
* `wait_frames(count: u32)` — Resume after the specified number of ticks.
* `wait_until(take condition: @no_suspend fn(): bool)` — Poll an owned non-suspending callable at the start of each tick.
* `wait_for(child: Script)` — Resume after the child finishes or is cancelled.
* `wait_any(scripts: Slice<Script>): Result<Script, WaitError>` — Copy the handles into owned coroutine state and resume after any finishes. Creating the snapshot has the `alloc` effect.

`wait(0s)` and `wait_frames(0)` are equivalent to `yield_frame()`. A borrowed closure cannot be passed to `wait_until` because the stored callable outlives the current slice.

#### The `suspend` Effect

Calling a suspension point gives a function the inferred, transitive `suspend` effect. Calling a potentially suspending function makes its caller suspending; only a script body is an entry point into a suspending call chain.

Function types publish whether their callable may suspend:

* `@suspend fn()` — May suspend and may be called only from a suspending context.
* `@no_suspend fn(): bool` — Cannot suspend. This is the default for a bare `fn(...)` type.

A non-suspending callable may satisfy an `@suspend` function type. A suspending callable cannot satisfy a bare or `@no_suspend` function type. Function declarations infer the effect; the attributes are written when a function-type contract requires them.

`@pure` is incompatible with `suspend`, while `@deterministic` and `@no_heap_alloc` may be compatible. `script.start` allocates the coroutine frame once. Afterward, `yield_frame`, `wait`, `wait_frames` and `wait_for` update existing state without allocating. Suspending helper calls must also have preallocated state to satisfy `@no_heap_alloc`; unbounded recursive suspension cannot claim that guarantee. `wait_until` and `wait_any` allocate owned state and violate `@no_heap_alloc`.

```juan
@no_heap_alloc
fn countdown(mut timer: Timer) {
    while timer.remaining > 0 {
        yield_frame()
        timer.remaining--
    }
}
```

Suspension points are forbidden inside `par for`, `parallel`, `spawn`, a `wait_until` condition, a destructor, contract, lock scope or constant initializer.

#### Values Across Suspension

Scoped views into pools, cells, locks or externally owned state, values belonging to `frame` or a borrowed arena, and ephemeral index places may not remain live across suspension. Owned values, counted values, handles, script-owned arenas and values derived from those arenas may remain live.

A suspending helper may retain read or exclusive parameter access across suspension only when the actual source is a local owned by the same pinned script call chain and no other script, task or host alias can access it. The compiler publishes which parameters survive suspension and checks the requirement at every call. This permits `countdown(mut timer)` on a script-local timer but rejects passing a pool entry or cell view. Every outstanding structured task must be joined first. The exclusive access lasts until the helper returns.

```juan
if let Some(mut enemy) = enemies.get_mut(handle) {
    enemy.alert()
    wait(1s) // Error: the pool entry remains borrowed
    enemy.attack()
}
```

The value must instead be reacquired:

```juan
if let Some(mut enemy) = enemies.get_mut(handle) {
    enemy.alert()
}
wait(1s)
if let Some(mut enemy) = enemies.get_mut(handle) {
    enemy.attack()
}
```

#### Scheduler

The host calls `tick(phase, dt)` with a `Duration` delta for each registered phase. Waiting conditions are checked once before resuming ready scripts, using the same phase/priority/start ordering. A satisfied wait becomes ready for that tick. `wait_any` snapshots and releases its input slice before suspension, rejects an empty list with an error, and breaks simultaneous-completion ties by input order. Waiting on oneself traps. Completion waits return no earlier than the next tick.

Ready scripts run in this deterministic order:

1. Lower numeric priority first.
2. Earlier start order first.

Each script runs until suspension, completion, trap, budget exhaustion or cancellation. Sequential script portions never run simultaneously with one another. A script may use `task`, `spawn`, `par for` and `parallel`; that parallel work joins before the script continues. Scripts started during a tick first run on the next tick of their phase.

#### Handles, Completion & Cancellation

`Script` is a copyable, nontrivially destructible handle to a counted completion record. The scheduler holds a strong reference while the script runs. Copying a handle retains the record.

On completion or cancellation, the coroutine frame and its locals are destroyed immediately. The completion record retains its state and error while any handle exists; the last handle frees it.

`Script` operations are host-thread-only. Dropping all external handles does not cancel a running script because the scheduler owns it. Scripts started by a registered callback inherit its registration owner; other starts belong to the runtime and are cancelled at shutdown.

Script handles provide:

* `is_running(): bool`
* `finished(): bool`
* `cancel()` — Request cancellation and destroy live locals in reverse order. A script not yet started never runs its body.

Cancellation is cooperative and observed at compiler-inserted checkpoints before loop backedges, function entries and suspension. The executing script unwinds only after returning from a host call; another thread never destroys its active frame. A suspended script can be cancelled without running its body again. Cancellation is observable inside the body only through destructors.

The host may configure a per-slice execution budget. Both VM and AOT code check the same logical work counter at checkpoints. Exhaustion traps the script and runs cleanup; it does not silently yield while forbidden borrows are live. Cleanup has a separate host-configured budget; exhaustion of that emergency budget aborts the runtime, so deterministic cleanup applies only to normal unwinding. Host calls must honor their own time limits because Juan cannot preempt arbitrary native code.

Scripts are unstructured concurrency bounded by ticks and registration lifetime rather than lexical scope.

#### Child Scripts

`script.start_child(body): Script` links a child to the current script. Cancelling the parent cancels the child, and the parent may call `wait_for` on it. A parent that completes normally leaves a child running unless that child's `cancel_with_parent` option is true. `script.start_child_with(options, body)` supplies child options; parent cancellation always propagates.

#### Errors & Traps

A trap ends only the affected script, destroys its live locals and reports to the host. A trap in a structured parallel region propagates into its containing script after sibling work is cancelled and joined.

#### Hot Reload

A script belongs to the generation that started it. Running scripts continue in that generation unless the host requests restart-on-reload for their phase. Scripts owned by a revoked registration set are cancelled before the new set is installed. A generation remains pinned while any of its scripts run.

#### Structured Tasks

`task`, `spawn` and `await` remain the structured parallel form for work that finishes within one script slice. `await` may suspend a worker task but never crosses ticks. Work that waits for another frame must be expressed as a script.

---

## 15. Rust Interoperability & Compilation

Juan is a scripting language for Rust hosts. Its compiler, runtime and embedding library are written in Rust, and generated bindings expose safe Rust APIs.

Juan never links a standalone executable and has no process entry point. A Rust host loads bytecode or links Juan-generated native modules and decides which exported functions to call.

Development builds compile every Juan module to bytecode. The embedded VM executes it, replaces compatible module generations during hot reload and preserves all Juan safety checks.

Release builds compile every included Juan module ahead of time into native objects. The Rust build links or packages those objects with generated metadata and adapters. Release builds do not execute Juan bytecode or require the VM.

The runtime never generates native machine code. Ahead-of-time output is not executable by itself and cannot define a process entry point.

Native modules and independently compiled host components use a stable internal calling convention and layout contract instead of Rust's native ABI and default layouts. No C source API is exposed.

Generated Rust adapters provide typed handles, `Result`, counted-value wrappers, panic containment, access validation and bulk data operations. Owned Juan values move across the boundary. Counted values such as `str`, `Rc`, `Arc`, `Weak`, `ArcWeak`, `Script` and named function values retain and release through generated wrappers. Scoped values are validated for the duration of the call, and Rust never receives a raw pointer into Juan-owned memory. The low-level boundary never creates overlapping Rust `&mut` references.

Host functions appear to Juan as typed free functions. VM and ahead-of-time code must pass the same conformance suite for results, traps, overflow, evaluation order, destruction, reference counting, arena provenance, suspension, contracts, host calls and reload behavior.

---

## 16. Basic Parsing Rules

Identifiers use ASCII letters, numbers and `_`. They must start with a letter or `_`. Strings, character literals, comments and documentation support Unicode.

Integer literals may be decimal, binary (`0b`), octal (`0o`) or hexadecimal (`0x`). Their optional suffix is one of `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128` or `usize`; an unconstrained unsuffixed integer defaults to `i32`. Floating-point literals use a decimal fraction, an exponent or an `f32`/`f64` suffix; an unconstrained unsuffixed float defaults to `f64`. Underscores may separate digits but may not begin or end a digit sequence. A leading `-` is the unary operator, not part of a literal.

The built-in unit suffixes `b`, `kb`, `mb`, `gb`, `ns`, `us`, `ms`, `s`, `min` and `h` may follow a decimal integer or floating-point body without whitespace. A literal has either a numeric type suffix or a unit suffix, never both. The lexer consumes the entire adjacent suffix identifier and requires an exact built-in suffix match; unknown suffixes are errors. `0b` followed immediately by `0` or `1` starts a binary literal. Standalone `0b` is zero bytes; invalid binary digits are diagnosed rather than split into adjacent literals. Use `0b0` for binary zero. An underscore may separate digits but may not separate the numeric body from its suffix, so `1_024kb` is valid and `1_kb` is not. Section 7.10 defines unit scaling and exactness.

A decimal point belongs to a numeric literal only when immediately followed by a decimal digit. This keeps literal UFCS calls and ranges unambiguous:

```juan
0.abs()       // i32 literal followed by UFCS
0i64.abs()    // explicitly typed i64 literal followed by UFCS
0.5.floor()   // f64 literal followed by UFCS
0..10         // integer literal followed by a range
```

Strings use double quotes and support escapes such as `\n`, `\r`, `\t`, `\\`, `\"` and `\u{1F600}`. A single-quoted literal contains exactly one Unicode scalar and has type `char`. A `b'X'` literal contains one ASCII byte and has type `u8`; a `b"..."` literal contains only bytes and escapes in the range `0..=255` and has type `Array<u8, N>`.

Commas separate arguments, record fields and values, generic arguments and array elements. Multiline argument, record and array lists may have a trailing comma. Match arms and block items do not use commas.

Keywords such as `as`, `from`, `where`, `par`, `parallel`, `task`, `spawn`, `await` and `move` are contextual where their grammar position is unambiguous. Built-in attribute and macro syntax names are not ordinary hard keywords.

The parser preserves comments, whitespace, attributes, macro invocations and invalid fragments in a lossless syntax tree. It creates error nodes and continues parsing when possible.

The compiler pipeline is:

```text
Lex
  -> Parse lossless CST
  -> Discover module and macro imports
  -> Expand macros
  -> Resolve ordinary names
  -> Check types, access, ownership, escape and effects
  -> Lower to Juan IR
  -> Emit VM bytecode or a host-linked native object
```


## 17. Standard Libraries & Host Metadata

Library APIs use ordinary types, free functions and the existing derive mechanism. Engine operations are provided by host bindings.

### 17.1 Binary Data & Bit Operations

`std.binary` provides checked reads and writes over `Slice<u8>` and `SliceMut<u8>`. Integer and floating-point operations name their byte order explicitly:

```juan
let speed = packet.read_f32_le(3)?
let identifier = packet.read_u32_be(7)?
```

Offsets are `usize` byte indices. These operations copy values and permit unaligned offsets; insufficient bytes return `Result.Err`. They never construct a typed reference into the input. Boolean, enum and text decoders additionally validate their encodings. Reading arbitrary record memory or padding is not serialization.

`std.bits` supplies `extract(value, offset, width)`, `replace(value, field, offset, width)`, rotations and bit counts for fixed-width unsigned integers. Bit zero is the least significant bit. Invalid ranges and replacement values that do not fit return errors. Full-width operations avoid shifting by the operand width. Juan has no native record bitfield syntax.

`BitReader` and `BitWriter` select `BitOrder.LsbFirst` or `BitOrder.MsbFirst` at construction. `read_bits(count)` and `write_bits(value, count)` advance an explicit cursor, support widths from 1 through 64, and report bounds and range errors. Stream bit order and multibyte endianness are separate settings.

### 17.2 Vector Swizzles

The math library provides read-only UFCS swizzles for two-, three- and four-component vectors:

```juan
let horizontal = position.xz()
let reversed = color.bgr()
```

Components use either `xyzw` or `rgba` consistently within one swizzle. Selecting a missing component is a compile error. Repetition is allowed, and the result is an independent vector value. Swizzles do not produce writable places. Ordinary field assignment remains available for individual components.

### 17.3 Serialization

`@derive(serialization)` emits typed fieldwise encode/decode operations for selected format libraries. Formats define byte order, string encoding, length representation and validation limits. The derive rejects unsupported fields and never serializes record padding, raw addresses or runtime code.

Decoding returns `Result` with field/path information and rejects invalid tags, lengths and primitive encodings. Readers impose configurable depth and allocation limits. Durable formats identify schemas and fields explicitly; defaults, unknown-field behavior and migrations are declared in serializer metadata rather than inferred from source field order.

Handles require an explicit stable external-identity mapping. Closures, scoped views, locks and script execution frames have no automatic serialization. Save files serialize persistent game state and explicit script checkpoints, not suspended native stacks.

### 17.4 Events & Subscriptions

Hosts expose typed `Event<T>` sources. `subscribe` takes an owned repeatedly callable callback and returns a move-only `Subscription`; dropping it unregisters the callback. Consuming callables are rejected. Mutable callbacks are invoked exclusively, and reentrant emissions are queued instead of recursively entering the same mutable environment.

Main-thread event delivery follows registration phase/priority/identity ordering. Payload ownership and queue capacity are part of each binding's contract. Queue overflow is reported through its declared error policy.

`wait_event(event)` has the `suspend` effect, installs a one-shot subscription and waits until an owned payload can be delivered on a later tick. Cancellation removes the subscription. Event waits may allocate and do not require polling a user closure.

### 17.5 Editor Metadata

Generated bindings expose opt-in record/field metadata for inspector labels, ranges, defaults, asset kinds and serializer identities. Imported attribute macros express that metadata using Section 13. Metadata does not bypass visibility, ownership or validation. Editors modify live values through validated host accessors, with incompatible type changes following the ordinary reload rules.
