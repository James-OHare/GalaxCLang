# GalaxC

**The programming language engineered for the void.**

GalaxC (Galaxy Code) is a statically typed, compiled programming language designed from first principles for ultra-reliable software in spacecraft, deep-space probes, orbital stations, robotics, navigation, life support, and other long-duration critical systems. It combines the safety rigor of Ada, the ergonomics of modern scripting languages, and a novel syntax identity built around determinism, explicit failure handling, and mission-grade fault tolerance.

GalaxC compiles to C for maximum portability across every platform that matters, from radiation-hardened flight processors to ground station servers.

**Tagline:** *Code that survives the void.*

---

## Table of Contents

1. [Language Vision](#1-language-vision)
2. [Core Philosophy](#2-core-philosophy)
3. [Syntax Design](#3-syntax-design)
4. [Type System](#4-type-system)
5. [Safety Model](#5-safety-model)
6. [Error Handling Model](#6-error-handling-model)
7. [Tasking and Concurrency](#7-tasking-and-concurrency)
8. [Mission-Critical Features](#8-mission-critical-features)
9. [Standard Library Design](#9-standard-library-design)
10. [Examples](#10-examples)
11. [Formal Grammar Sketch](#11-formal-grammar-sketch)
12. [Runtime Model](#12-runtime-model)
13. [Tooling](#13-tooling)
14. [Compatibility Strategy](#14-compatibility-strategy)
15. [Version 0.1 Design Decisions](#15-version-01-design-decisions)
16. [Final Identity](#16-final-identity)

---

## 1. Language Vision

GalaxC exists because no current language properly serves the intersection of safety, readability, mission-critical determinism, and developer ergonomics required by space-grade software. C is unsafe. C++ is enormous and treacherous. Ada is safe but syntactically heavy. Rust is powerful but has a steep learning curve and no built-in tasking model. Python is pleasant but unfit for real-time critical systems.

GalaxC solves this by being:

- **Safe by default.** No null pointers, no undefined behavior, no silent failures, no hidden mutation, no data races, no unchecked overflow, no use-after-free, no buffer overflows.
- **Explicit about everything that matters.** Errors, side effects, resource use, mutation, timing constraints, and failure modes are all visible in the source code.
- **Readable.** The syntax is clean, consistent, and scannable. Basic programs feel approachable. Complex programs remain auditable.
- **Deterministic.** Execution order, memory use, and timing behavior are predictable and bounded where required.
- **Mission-aware.** First-class support for unit-safe numerics, watchdogs, checkpointing, safe-mode transitions, telemetry, fault containment, and redundancy.
- **Concurrent via tasking.** An Ada-inspired tasking model provides safe, deterministic concurrency with rendezvous-based communication, protected shared state, and priority-driven scheduling.

### What makes GalaxC different

| Concern | C | Ada | Rust | GalaxC |
|---|---|---|---|---|
| Null safety | No | Partial | Yes | Yes |
| Data race freedom | No | Partial | Yes | Yes |
| Built-in tasking | No | Yes | No | Yes |
| Unit-safe numerics | No | No | No | Yes |
| Effect annotations | No | No | No | Yes |
| Watchdog/deadline support | No | Partial | No | Yes |
| Readable syntax | Yes | Verbose | Moderate | Yes |
| Compiles to C | N/A | Some | No | Yes |

---

## 2. Core Philosophy

### Guiding Principles

1. **No silent failures.** Every operation that can fail must declare it. The compiler rejects code that ignores failure paths. A silent failure in space means loss of mission.

2. **Immutable by default.** All bindings are immutable unless explicitly declared mutable with `var`. This eliminates accidental mutation, the source of countless bugs in long-running systems.

3. **Explicit side effects.** Functions that perform I/O, access hardware, allocate memory, use timing, or communicate over a network must annotate those effects. Pure functions are the default assumption.

4. **Deterministic execution.** In safe code, the order of operations, memory layout, and timing behavior are deterministic. No garbage collector. No hidden allocations. No runtime surprises.

5. **Make invalid states unrepresentable.** The type system should prevent illegal program states at compile time. If a thruster can only fire between 0 and 100% power, the type should enforce that.

6. **Failure is normal.** Space systems experience failures constantly: sensor glitches, communication dropouts, radiation-induced bit flips. GalaxC treats failure as a first-class concept, not an exception.

7. **Tasks are the concurrency model.** Concurrency is expressed through tasks with clearly defined communication boundaries, not raw threads. Tasks communicate via rendezvous and protected objects, eliminating data races by construction.

8. **Readable code is safe code.** If code is hard to read, it is hard to audit, and unaudited code has no place in mission software. GalaxC syntax is designed to be scannable, greppable, and reviewable.

9. **The compiler is your co-pilot.** The compiler should catch as many problems as possible before the code ever runs. Type errors, exhaustiveness failures, unit mismatches, unhandled errors, effect violations, and ownership mistakes are all compile-time errors.

10. **Small core, rich libraries.** The language core is minimal and well-defined. Domain-specific functionality lives in the standard library and mission libraries. This keeps the language learnable while enabling deep capability.

---

## 3. Syntax Design

### 3.1 Overview

GalaxC syntax uses `=>` to open blocks and `end` to close them. There are no braces. Indentation is conventional but not syntactically significant; the compiler enforces formatting via the built-in formatter.

Comments use `--` for single-line and `---` for documentation. Module-level documentation uses `--!`.

Identifiers use `snake_case` for values and functions, `PascalCase` for types, and `UPPER_CASE` for constants.

### 3.2 Keywords

```
-- Declarations
op          -- function/operation definition
let         -- immutable binding
var         -- mutable binding
const       -- compile-time constant
struct      -- record/struct type
enum        -- sum type / algebraic data type
ability     -- trait/interface
task        -- concurrent task declaration
protected   -- protected shared object
orbit       -- module declaration
dock        -- import

-- Control flow
if          -- conditional
else        -- alternative branch
match       -- pattern match
for         -- iteration
while       -- conditional loop
loop        -- infinite loop (with explicit break)
break       -- exit loop
continue    -- skip to next iteration
return      -- early return
select      -- task communication select
accept      -- accept a rendezvous entry

-- Types and values
self        -- current instance
Self        -- current type
true        -- boolean true
false       -- boolean false
none        -- absence value (Option.None)
ok          -- success wrapper
err         -- error wrapper

-- Safety and effects
safe        -- safe code block
@effect     -- effect annotation
@deadline   -- timing constraint
@priority   -- task priority
@watchdog   -- watchdog annotation
@fallback   -- fallback handler
@checkpoint -- checkpoint annotation
@bounded    -- bounded resource annotation
@entry      -- task entry point

-- Ownership
own         -- owned value (default)
ref         -- immutable borrow
mut ref     -- mutable borrow

-- Modifiers
pub         -- public visibility
mut         -- mutable modifier
async       -- asynchronous modifier (for I/O tasks)
```

### 3.3 Operators

```
-- Arithmetic
+   -   *   /   %

-- Comparison
==  !=  <   >   <=  >=

-- Logical
and   or   not

-- Bitwise
&   |   ^   ~   <<  >>

-- Assignment
=       -- binding / assignment
+=  -=  *=  /=  %=

-- Special
?       -- error propagation (unwrap-or-propagate)
!!      -- error conversion (unwrap-or-convert)
>>      -- pipeline operator
::      -- path separator
..      -- range operator
->      -- return type annotation
=>      -- block opener
|x|     -- closure parameter list
@       -- annotation prefix
```

### 3.4 Block Structure

All blocks open with `=>` and close with `end`. This applies to functions, control flow, type definitions, modules, and task declarations.

```
op greet(name: Text) -> Text =>
    return "Hello, " ++ name
end
```

Single-expression blocks can be written inline:

```
op double(x: Int) -> Int => x * 2
```

### 3.5 Functions

```
-- Basic function
op add(a: Int, b: Int) -> Int =>
    return a + b
end

-- Function with no return value
op log_event(msg: Text) =>
    console.write(msg)
end

-- Function with effect annotation
@effect(io)
op read_file(path: Text) -> Result<Text, IoError> =>
    let handle = file.open(path)?
    let content = handle.read_all()?
    return ok(content)
end

-- Pure function (no annotation needed, purity is default)
op celsius_to_kelvin(c: Float64) -> Float64 =>
    return c + 273.15
end
```

### 3.6 Variables and Constants

```
-- Immutable binding (default)
let x = 42
let name: Text = "Voyager"

-- Mutable binding
var counter: Int = 0
counter = counter + 1

-- Compile-time constant
const MAX_THRUST: Float64<newtons> = 450.0
const LIGHT_SPEED: Float64<meters_per_second> = 299_792_458.0
```

### 3.7 Structs

```
struct Waypoint =>
    x: Float64<meters>
    y: Float64<meters>
    z: Float64<meters>
    label: Text
    priority: Int
end

-- Construction
let wp = Waypoint {
    x: 100.0,
    y: 200.0,
    z: 50.0,
    label: "Alpha",
    priority: 1,
}
```

### 3.8 Enums (Sum Types)

```
enum Command =>
    Thrust(force: Float64<newtons>)
    Rotate(axis: Vec3, angle: Float64<radians>)
    Hold
    Shutdown
end

-- Usage
let cmd = Command.Thrust(force: 200.0)
```

### 3.9 Pattern Matching

```
match cmd =>
    Command.Thrust(force) =>
        apply_thrust(force)
    end
    Command.Rotate(axis, angle) =>
        rotate_vehicle(axis, angle)
    end
    Command.Hold =>
        hold_position()
    end
    Command.Shutdown =>
        initiate_shutdown()
    end
end
```

Pattern matching is exhaustive. The compiler rejects a `match` that does not cover all variants. A wildcard `_` can be used as a catch-all.

### 3.10 Control Flow

```
-- If/else
if temperature > 400.0 =>
    trigger_alarm("overheating")
else if temperature < -200.0 =>
    trigger_alarm("too cold")
else =>
    log("nominal")
end

-- While loop
while fuel_remaining() > 0.0 =>
    burn(1.0)
end

-- For loop (iteration)
for sensor in sensors =>
    let reading = sensor.read()?
    log_reading(reading)
end

-- For with range
for i in 0..10 =>
    process(i)
end

-- Infinite loop with break
loop =>
    let status = check_status()
    if status == Status.Done =>
        break
    end
    wait(1.seconds)
end
```

### 3.11 Modules and Imports

```
-- Module declaration (top of file)
orbit navigation.guidance

-- Import a module
dock core.math
dock core.collections.Vec
dock core.collections.{Map, Set}
dock mission.telemetry as telem

-- Qualified usage
let distance = math.sqrt(dx * dx + dy * dy)

-- Or unqualified after specific import
dock core.math.{sqrt, cos, sin}
let distance = sqrt(dx * dx + dy * dy)
```

### 3.12 Abilities (Traits)

```
ability Serializable =>
    op serialize(self) -> Bytes
    op size_hint(self) -> Int
end

ability Displayable =>
    op display(self) -> Text
end

-- Implementation
struct Telemetry =>
    timestamp: Int
    sensor_id: Int
    value: Float64
end

impl Serializable for Telemetry =>
    op serialize(self) -> Bytes =>
        var buf = Bytes.with_capacity(self.size_hint())
        buf.write_int(self.timestamp)
        buf.write_int(self.sensor_id)
        buf.write_float(self.value)
        return buf
    end

    op size_hint(self) -> Int => 24
end
```

### 3.13 Generics

```
op first<T>(items: Slice<T>) -> Option<T> =>
    if items.len() == 0 =>
        return none
    end
    return some(items[0])
end

struct Pair<A, B> =>
    first: A
    second: B
end

ability Container<T> =>
    op push(mut self, item: T)
    op pop(mut self) -> Option<T>
    op len(self) -> Int
end
```

### 3.14 Comments and Documentation

```
-- This is a single-line comment

--- Computes the delta-v required for a Hohmann transfer orbit.
---
--- Parameters:
---   r1 -- radius of the initial circular orbit (meters)
---   r2 -- radius of the target circular orbit (meters)
---   mu -- gravitational parameter of the central body (m^3/s^2)
---
--- Returns the total delta-v in meters per second.
op hohmann_delta_v(
    r1: Float64<meters>,
    r2: Float64<meters>,
    mu: Float64<meters3_per_second2>,
) -> Float64<meters_per_second> =>
    let a_transfer = (r1 + r2) / 2.0
    let dv1 = math.sqrt(mu / r1) * (math.sqrt(2.0 * r2 / (r1 + r2)) - 1.0)
    let dv2 = math.sqrt(mu / r2) * (1.0 - math.sqrt(2.0 * r1 / (r1 + r2)))
    return math.abs(dv1) + math.abs(dv2)
end
```

### 3.15 Annotations

```
@effect(io, timing)
@deadline(ms: 50)
@priority(critical)
op send_telemetry(packet: TelemetryPacket) -> Result<(), CommError> =>
    let encoded = packet.serialize()
    comm.transmit(encoded)?
    return ok(())
end

@bounded(memory: kb(64))
op process_buffer(data: Slice<Byte>) -> Result<ProcessedData, BufferError> =>
    -- compiler verifies this function uses at most 64KB of stack + local heap
    ...
end

@checkpoint(interval: 10.seconds)
op long_computation(input: DataSet) -> Result<Output, ComputeError> =>
    -- compiler inserts checkpoint save points at annotated intervals
    ...
end
```

---

## 4. Type System

### 4.1 Primitive Types

| Type | Description | Size |
|---|---|---|
| `Bool` | Boolean true/false | 1 byte |
| `Int` | Platform-native signed integer | Platform |
| `Int8` | 8-bit signed integer | 1 byte |
| `Int16` | 16-bit signed integer | 2 bytes |
| `Int32` | 32-bit signed integer | 4 bytes |
| `Int64` | 64-bit signed integer | 8 bytes |
| `Uint8` | 8-bit unsigned integer | 1 byte |
| `Uint16` | 16-bit unsigned integer | 2 bytes |
| `Uint32` | 32-bit unsigned integer | 4 bytes |
| `Uint64` | 64-bit unsigned integer | 8 bytes |
| `Float32` | 32-bit IEEE 754 float | 4 bytes |
| `Float64` | 64-bit IEEE 754 float | 8 bytes |
| `Byte` | Alias for Uint8 | 1 byte |
| `Text` | UTF-8 string | Pointer + length |
| `Char` | Unicode scalar value | 4 bytes |
| `Never` | Uninhabited type (function never returns) | 0 bytes |

All arithmetic on integer types is **checked by default**. Overflow causes a recoverable error (Result), not undefined behavior. Wrapping, saturating, and unchecked arithmetic are available as explicit method calls:

```
let a: Int32 = 2_000_000_000
let b: Int32 = 2_000_000_000
let c = a.checked_add(b)        -- Result<Int32, OverflowError>
let d = a.wrapping_add(b)       -- Int32, wraps silently (must opt in)
let e = a.saturating_add(b)     -- Int32, clamps to max
```

### 4.2 Compound Types

```
-- Fixed-size array
let readings: [Float64; 8] = [0.0; 8]

-- Slice (borrowed view of contiguous memory)
let window: Slice<Float64> = readings[2..5]

-- Tuple
let pair: (Int, Text) = (42, "answer")

-- Option (presence or absence)
let maybe: Option<Int> = some(42)
let nothing: Option<Int> = none

-- Result (success or typed failure)
let outcome: Result<Data, SensorError> = ok(data)
let failure: Result<Data, SensorError> = err(SensorError.Timeout)
```

### 4.3 Option and Result

These are the two core algebraic types for handling absence and failure.

```
enum Option<T> =>
    Some(T)
    None
end

enum Result<T, E> =>
    Ok(T)
    Err(E)
end
```

The `?` operator propagates errors:

```
op load_config() -> Result<Config, ConfigError> =>
    let text = read_file("config.gxc")?       -- propagates IoError -> ConfigError
    let parsed = parse_config(text)?            -- propagates ParseError -> ConfigError
    return ok(parsed)
end
```

The `!!` operator converts errors:

```
let value = risky_operation() !! DefaultError.from(context)
```

### 4.4 Unit-Safe Numeric Types

GalaxC prevents unit mismatch errors (like the Mars Climate Orbiter loss) at the type level:

```
-- Declaring unit-annotated values
let distance: Float64<meters> = 1000.0
let time: Float64<seconds> = 10.0
let speed = distance / time     -- inferred: Float64<meters_per_second>

-- This is a compile error:
let bad = distance + time       -- ERROR: cannot add meters to seconds

-- Unit conversions are explicit
let km = distance.to_unit<kilometers>()

-- Custom units
unit newtons = kilograms * meters / seconds^2
unit pascals = newtons / meters^2

let force: Float64<newtons> = 100.0
let area: Float64<meters2> = 2.0
let pressure = force / area     -- Float64<pascals>
```

The unit system is dimensional analysis at compile time. The compiler tracks dimensions (length, mass, time, temperature, current, etc.) and verifies that operations produce dimensionally consistent results.

### 4.5 Enums and Pattern Matching

Enums in GalaxC are full algebraic data types (tagged unions):

```
enum ThrusterState =>
    Off
    Warming(progress: Float64)
    Firing(power: Float64<newtons>)
    Cooldown(remaining: Float64<seconds>)
    Fault(code: Int, message: Text)
end
```

Pattern matching must be exhaustive:

```
op describe_thruster(state: ThrusterState) -> Text =>
    match state =>
        ThrusterState.Off => "Thruster offline"
        ThrusterState.Warming(p) => "Warming: " ++ p.display() ++ "%"
        ThrusterState.Firing(power) => "Firing at " ++ power.display()
        ThrusterState.Cooldown(t) => "Cooling: " ++ t.display() ++ " remaining"
        ThrusterState.Fault(code, msg) => "FAULT " ++ code.display() ++ ": " ++ msg
    end
end
```

### 4.6 Abilities (Traits)

```
ability Measurable =>
    op measure(self) -> Result<Float64, MeasureError>
    op unit_label(self) -> Text
end

ability Bounded =>
    const MIN: Self
    const MAX: Self
    op clamp(self, lo: Self, hi: Self) -> Self
end

-- Ability bounds on generics
op max_reading<T: Measurable + Bounded>(sensors: Slice<T>) -> Result<Float64, MeasureError> =>
    var best = T.MIN
    for s in sensors =>
        let val = s.measure()?
        if val > best =>
            best = val
        end
    end
    return ok(best)
end
```

### 4.7 Preventing Invalid States

GalaxC uses the type system to make illegal states unrepresentable:

```
-- Instead of a boolean flag that could be wrong:
-- BAD: struct Valve { is_open: Bool, flow_rate: Float64 }

-- Use an enum that structurally prevents invalid combinations:
enum Valve =>
    Closed
    Open(flow_rate: Float64<liters_per_second>)
    Fault(reason: Text)
end

-- You literally cannot have a "closed valve with a flow rate"
-- or an "open valve with no flow rate"
```

---

## 5. Safety Model

### 5.1 Memory Safety

GalaxC uses ownership and borrowing to guarantee memory safety without a garbage collector.

**Rules:**

1. Every value has exactly one owner.
2. When the owner goes out of scope, the value is dropped (and its resources freed).
3. You can have either one mutable borrow OR any number of immutable borrows, but not both simultaneously.
4. References cannot outlive the value they point to.

```
op process() =>
    let data = Vec.new<Int>()    -- data owns the vector

    let view = ref data          -- immutable borrow
    log(view.len())

    var editor = mut ref data    -- mutable borrow (no other borrows active)
    editor.push(42)

    -- data is dropped here, memory freed
end
```

These rules are enforced entirely at compile time. There is no runtime overhead for memory safety in safe code.

### 5.2 Concurrency Safety

Data races are prevented by the ownership and tasking models:

1. Tasks do not share mutable state directly.
2. Communication between tasks uses rendezvous (synchronous message passing) or protected objects (compiler-verified mutex-like access).
3. The compiler rejects any code that could create a data race.

See [Section 7: Tasking and Concurrency](#7-tasking-and-concurrency) for the full model.

### 5.3 Determinism

GalaxC guarantees deterministic execution in safe code:

- No garbage collection pauses.
- No hidden memory allocation (all allocation is explicit via effects).
- No thread scheduling non-determinism (tasks have priorities and deterministic scheduling).
- Floating-point operations follow IEEE 754 strictly.
- Integer overflow is checked, not undefined.
- Evaluation order is strictly left-to-right, top-to-bottom.

### 5.4 Side-Effect Control

Functions are pure by default. Side effects must be declared:

```
@effect(io)
op write_log(msg: Text) =>
    file.append("mission.log", msg)
end

@effect(hardware)
op fire_thruster(id: Int, power: Float64<newtons>) =>
    hardware.write_register(THRUSTER_BASE + id, power)
end

@effect(timing)
op wait_ms(ms: Int) =>
    system.sleep(ms)
end

-- A pure function cannot call an effectful function.
-- This is a compile error:
op compute(x: Int) -> Int =>
    write_log("computing")     -- ERROR: pure function cannot perform 'io' effect
    return x * 2
end
```

### 5.5 What Is Forbidden in Safe Code

- Null pointers (the type system has no null; use Option)
- Dangling references (ownership and borrowing prevent this)
- Buffer overflows (array access is bounds-checked; slices carry their length)
- Use-after-free (ownership prevents this)
- Data races (the tasking model prevents this)
- Uninitialized memory access (all variables must be initialized)
- Integer overflow (checked by default)
- Implicit narrowing conversions (all narrowing is explicit and checked)
- Unchecked type casts (safe casts only; reinterpretation requires `unsafe` blocks)
- Calling foreign C functions outside `unsafe` blocks

An `unsafe` block exists for interfacing with hardware and C code, but it is lexically marked and auditable:

```
unsafe =>
    let raw = ptr.read_volatile(REGISTER_ADDR)
end
```

---

## 6. Error Handling Model

### 6.1 Error Representation

Errors in GalaxC are values, not exceptions. Every fallible operation returns a `Result<T, E>` where `E` is a concrete error type.

```
enum SensorError =>
    Timeout
    OutOfRange(value: Float64, min: Float64, max: Float64)
    HardwareFault(code: Int)
    CalibrationExpired
end

op read_temperature(id: SensorId) -> Result<Float64<kelvin>, SensorError> =>
    let raw = hardware.adc_read(id) !! SensorError.Timeout
    if raw < VALID_MIN or raw > VALID_MAX =>
        return err(SensorError.OutOfRange(
            value: raw,
            min: VALID_MIN,
            max: VALID_MAX,
        ))
    end
    let calibrated = apply_calibration(raw)?
    return ok(calibrated)
end
```

### 6.2 Error Propagation

The `?` operator propagates errors upward. It unwraps `Ok` values and returns `Err` values to the caller:

```
op initialize_subsystem() -> Result<(), SubsystemError> =>
    let config = load_config()?          -- propagates ConfigError
    let sensors = init_sensors(config)?  -- propagates SensorError
    let comms = init_comms(config)?      -- propagates CommError
    return ok(())
end
```

Error types must be convertible. The compiler checks that the error type of the inner call can convert to the error type of the outer function (via an `Into` ability implementation).

### 6.3 Error Handling Patterns

```
-- Pattern match on result
match read_temperature(sensor_a) =>
    Result.Ok(temp) =>
        log("Temperature: " ++ temp.display())
    end
    Result.Err(SensorError.Timeout) =>
        log("Sensor timeout, using backup")
        use_backup_sensor()
    end
    Result.Err(e) =>
        log("Sensor error: " ++ e.display())
        enter_safe_mode()
    end
end

-- Provide a default on failure
let temp = read_temperature(sensor_a).unwrap_or(DEFAULT_TEMP)

-- Chain fallbacks
let temp = read_temperature(sensor_a)
    .or_else(|| read_temperature(sensor_b))
    .or_else(|| ok(DEFAULT_TEMP))
    .unwrap()
```

### 6.4 Compiler Enforcement

The compiler enforces the following rules:

1. A `Result` value cannot be silently discarded. You must handle it, propagate it, or explicitly acknowledge it.
2. All `match` arms on a `Result` or `Option` must be exhaustive.
3. Functions that can fail must declare their error type in the return signature.
4. The `?` operator can only be used in functions that return `Result`.
5. Error types must implement the `Displayable` ability so diagnostics are always available.

---

## 7. Tasking and Concurrency

GalaxC provides an Ada-inspired tasking model as its primary concurrency mechanism. Tasks are lightweight concurrent units of execution with well-defined communication interfaces.

### 7.1 Task Declarations

A task has a specification (its interface) and a body (its implementation):

```
--- A task that periodically polls a sensor array and provides
--- the latest readings on demand.
task SensorPoller =>
    @entry op poll() -> Result<SensorReading, SensorError>
    @entry op get_status() -> PollerStatus
    @entry op shutdown()
end
```

### 7.2 Task Bodies

```
task body SensorPoller(sensor_id: SensorId, interval: Duration) =>
    var status = PollerStatus.Running
    var latest: Option<SensorReading> = none

    loop =>
        select =>
            accept poll() =>
                let reading = read_sensor(sensor_id)?
                latest = some(reading)
                return ok(reading)
            end

            or accept get_status() =>
                return status
            end

            or accept shutdown() =>
                status = PollerStatus.Stopped
                break
            end

            or delay interval =>
                -- Timeout: perform automatic background poll
                match read_sensor(sensor_id) =>
                    Result.Ok(r) => latest = some(r)
                    Result.Err(_) => status = PollerStatus.Degraded
                end
            end
        end
    end
end
```

### 7.3 Rendezvous Communication

Tasks communicate through rendezvous: the caller waits at an entry call, the task waits at an accept statement, and they synchronize:

```
-- Caller side
let poller = SensorPoller.spawn(sensor_id: 1, interval: 500.ms)
let reading = poller.poll()?       -- blocks until the task accepts
poller.shutdown()                  -- blocks until the task accepts
```

This is inherently race-free because data is exchanged during a synchronized handshake.

### 7.4 Protected Objects

For shared state that needs concurrent read/write access, GalaxC provides protected objects with compiler-verified mutual exclusion:

```
protected SharedState =>
    var altitude: Float64<meters> = 0.0
    var velocity: Float64<meters_per_second> = 0.0

    op read_altitude(self) -> Float64<meters> =>
        return self.altitude
    end

    op read_velocity(self) -> Float64<meters_per_second> =>
        return self.velocity
    end

    op update(mut self, alt: Float64<meters>, vel: Float64<meters_per_second>) =>
        self.altitude = alt
        self.velocity = vel
    end
end
```

Protected objects guarantee:
- Multiple readers or a single writer (never both)
- Deadlock-free access (no nested locking)
- Bounded waiting (priority ceiling protocol)

### 7.5 Task Priorities

```
@priority(critical)
task FlightController =>
    @entry op update_trajectory(cmd: TrajectoryCommand)
    @entry op emergency_abort()
end

@priority(normal)
task TelemetryReporter =>
    @entry op report()
end

@priority(low)
task DiagnosticsLogger =>
    @entry op log_diagnostics()
end
```

The scheduler guarantees that higher-priority tasks preempt lower-priority tasks. Priority inversion is prevented via the priority ceiling protocol on protected objects.

### 7.6 Select Statements

The `select` statement allows a task to wait on multiple entry calls or timeouts:

```
select =>
    accept command(cmd) =>
        execute(cmd)
    end

    or accept query() =>
        return current_state()
    end

    or delay 5.seconds =>
        -- No communication for 5 seconds, take action
        log("No commands received, continuing autonomous")
    end

    or when fuel_low() =>
        -- Guarded alternative, only available when condition is true
        accept emergency_refuel(amount) =>
            refuel(amount)
        end
    end
end
```

---

## 8. Mission-Critical Features

### 8.1 Checkpointing

```
@checkpoint(interval: 30.seconds)
op long_computation(data: DataSet) -> Result<Output, ComputeError> =>
    var accumulator = Accumulator.new()
    for item in data =>
        accumulator.ingest(item)?
    end
    return ok(accumulator.finalize())
end
```

The compiler inserts checkpoint saves at annotated intervals. If the process restarts, it resumes from the last checkpoint rather than starting over. This is critical for long-duration deep-space computations that cannot afford to lose hours of progress to a transient fault.

### 8.2 Safe Mode Transitions

```
enum SystemMode =>
    Nominal
    Degraded(reason: Text)
    SafeMode
    Emergency
end

@fallback(enter_safe_mode)
op navigate() -> Result<(), NavError> =>
    let position = gps.read()?
    let target = mission_plan.next_waypoint()?
    let correction = compute_correction(position, target)?
    apply_correction(correction)?
    return ok(())
end

op enter_safe_mode() =>
    thruster.all_stop()
    solar_panels.sun_point()
    comm.send_distress()
    log("Entered safe mode")
end
```

### 8.3 Watchdog Support

```
@watchdog(timeout: 5.seconds)
op main_control_loop() =>
    loop =>
        system.pet_watchdog()
        let status = check_all_subsystems()
        match status =>
            Status.Nominal => continue_operations()
            Status.Degraded(reason) => reduce_operations(reason)
            Status.Critical => enter_safe_mode()
        end
    end
end
```

If the function does not call `pet_watchdog()` within the timeout, the runtime triggers the watchdog handler, which by default resets the subsystem.

### 8.4 Fault Containment

```
-- Subsystems are isolated fault domains
@fault_domain("propulsion")
orbit propulsion

@fault_domain("navigation")
orbit navigation

-- A fault in propulsion cannot corrupt navigation state
-- The runtime enforces memory isolation between fault domains
```

### 8.5 Telemetry

```
@effect(telemetry)
op report_status(state: SystemState) =>
    telem.send(TelemetryPacket {
        timestamp: time.mission_clock(),
        subsystem: "guidance",
        data: state.serialize(),
        priority: TelemetryPriority.Normal,
    })
end
```

### 8.6 Command Handling

```
op handle_ground_command(raw: Bytes) -> Result<(), CommandError> =>
    let cmd = Command.deserialize(raw)?
    let verified = cmd.verify_checksum()?

    match verified =>
        Command.UpdateTrajectory(params) =>
            flight_controller.update_trajectory(params)
        end
        Command.RequestTelemetry(subsys) =>
            let data = collect_telemetry(subsys)?
            comm.transmit(data)?
        end
        Command.EmergencyAbort =>
            flight_controller.emergency_abort()
        end
        _ =>
            return err(CommandError.UnknownCommand(cmd.id))
        end
    end
    return ok(())
end
```

### 8.7 Redundancy

```
--- Triple-modular redundancy: run the same computation on three
--- independent inputs and vote on the result.
op tmr_vote<T: Eq>(a: T, b: T, c: T) -> Result<T, RedundancyError> =>
    if a == b => return ok(a)
    if a == c => return ok(a)
    if b == c => return ok(b)
    return err(RedundancyError.NoConsensus)
end

op safe_attitude_read() -> Result<AttitudeData, SensorError> =>
    let a = imu_a.read()?
    let b = imu_b.read()?
    let c = imu_c.read()?
    return tmr_vote(a, b, c) !! SensorError.RedundancyFailure
end
```

### 8.8 Verification Hooks

```
-- Compile-time assertions
static_assert(size_of<TelemetryPacket>() == 128)
static_assert(MAX_THRUST > 0.0)

-- Runtime pre/post conditions
@requires(power >= 0.0 and power <= 1.0)
@ensures(result.is_ok())
op set_thruster_power(id: Int, power: Float64) -> Result<(), ThrusterError> =>
    ...
end
```

---

## 9. Standard Library Design

### 9.1 Module Overview

| Module | Purpose |
|---|---|
| `core` | Fundamental types: Option, Result, Bool, Int, Float, Text, Bytes, Never |
| `core.ops` | Operator abilities: Add, Sub, Mul, Div, Eq, Ord, Display |
| `core.units` | Unit definitions and dimensional analysis |
| `math` | Mathematical functions: sqrt, sin, cos, atan2, abs, min, max, clamp |
| `time` | Mission time, Duration, Instant, Deadline, mission_clock() |
| `io` | File, stream, and console I/O |
| `collections` | Vec, Map, Set, RingBuffer, BoundedVec, Deque |
| `text` | String manipulation, formatting, parsing |
| `fmt` | Display formatting, structured output |
| `telemetry` | Telemetry channels, structured logging, packet encoding |
| `sync` | Channels, atomics, barriers (for low-level use behind protected objects) |
| `check` | CRC, checksums, data integrity verification |
| `crypto` | Hashing, HMAC (no full TLS -- this is for data integrity, not web) |
| `system` | Platform interface: watchdog, sleep, environment, process |
| `hardware` | Register access, DMA, interrupt handling (requires `unsafe`) |
| `testing` | Test framework: assertions, test runner, property-based testing |

### 9.2 Core Types

```
-- These are built into every GalaxC program

enum Option<T> =>
    Some(T)
    None
end

enum Result<T, E> =>
    Ok(T)
    Err(E)
end

enum Ordering =>
    Less
    Equal
    Greater
end
```

### 9.3 Collections

```
dock core.collections.Vec
dock core.collections.BoundedVec
dock core.collections.Map
dock core.collections.RingBuffer

-- BoundedVec has a compile-time capacity limit
let readings: BoundedVec<Float64, 256> = BoundedVec.new()

-- RingBuffer for telemetry history
let history: RingBuffer<TelemetryPacket, 1024> = RingBuffer.new()
```

### 9.4 Time

```
dock core.time

let mission_start = time.mission_clock()
let elapsed = time.since(mission_start)

-- Deadlines
let deadline = time.deadline(100.ms)
if time.past_deadline(deadline) =>
    log("Missed deadline!")
end
```

---

## 10. Examples

### Example 1: Hello World

```
orbit main

@effect(io)
op launch() =>
    console.write("GalaxC online. All systems nominal.")
end
```

### Example 2: Variables and Basic Math

```
orbit main

dock core.math

op launch() =>
    let radius: Float64 = 6371.0
    let circumference = 2.0 * math.PI * radius
    console.write("Earth circumference: " ++ circumference.display() ++ " km")
end
```

### Example 3: Structs and Methods

```
orbit main

struct CrewMember =>
    name: Text
    role: Text
    hours_in_space: Int
end

impl CrewMember =>
    op new(name: Text, role: Text) -> CrewMember =>
        return CrewMember {
            name: name,
            role: role,
            hours_in_space: 0,
        }
    end

    op log_hours(mut self, hours: Int) =>
        self.hours_in_space = self.hours_in_space + hours
    end
end

impl Displayable for CrewMember =>
    op display(self) -> Text =>
        return self.name ++ " (" ++ self.role ++ ") - " ++
               self.hours_in_space.display() ++ "h in space"
    end
end
```

### Example 4: Error Handling

```
orbit main

dock core.io

enum ConfigError =>
    FileNotFound(path: Text)
    ParseError(line: Int, message: Text)
    MissingField(name: Text)
end

@effect(io)
op load_mission_config(path: Text) -> Result<MissionConfig, ConfigError> =>
    let text = io.read_file(path)
        !! ConfigError.FileNotFound(path: path)
    let parsed = parse_toml(text)?
    let name = parsed.get("mission_name")
        .ok_or(ConfigError.MissingField(name: "mission_name"))?
    let duration = parsed.get_int("duration_days")
        .ok_or(ConfigError.MissingField(name: "duration_days"))?
    return ok(MissionConfig { name: name, duration_days: duration })
end
```

### Example 5: Pattern Matching

```
orbit main

enum CelestialBody =>
    Star(name: Text, mass_solar: Float64)
    Planet(name: Text, orbital_period_days: Float64)
    Moon(name: Text, parent: Text)
    Asteroid(designation: Text)
end

op describe(body: CelestialBody) -> Text =>
    match body =>
        CelestialBody.Star(name, mass) =>
            name ++ ": star, " ++ mass.display() ++ " solar masses"
        end
        CelestialBody.Planet(name, period) =>
            name ++ ": planet, " ++ period.display() ++ " day orbit"
        end
        CelestialBody.Moon(name, parent) =>
            name ++ ": moon of " ++ parent
        end
        CelestialBody.Asteroid(id) =>
            "Asteroid " ++ id
        end
    end
end
```

### Example 6: Unit-Safe Numerics

```
orbit main

dock core.math
dock core.units

op compute_orbital_velocity(
    mu: Float64<meters3_per_second2>,
    radius: Float64<meters>,
) -> Float64<meters_per_second> =>
    return math.sqrt(mu / radius)
end

op launch() =>
    const EARTH_MU: Float64<meters3_per_second2> = 3.986e14
    let leo_radius: Float64<meters> = 6_771_000.0
    let velocity = compute_orbital_velocity(EARTH_MU, leo_radius)
    console.write("LEO velocity: " ++ velocity.display() ++ " m/s")
end
```

### Example 7: Tasking (Ada-Style Concurrency)

```
orbit main

dock core.time
dock core.sync

task HeartbeatMonitor =>
    @entry op check_heartbeat() -> Bool
    @entry op shutdown()
end

task body HeartbeatMonitor(interval: Duration) =>
    var alive = true
    var last_beat = time.mission_clock()

    loop =>
        select =>
            accept check_heartbeat() =>
                let elapsed = time.since(last_beat)
                return elapsed < interval * 3
            end

            or accept shutdown() =>
                alive = false
                break
            end

            or delay interval =>
                last_beat = time.mission_clock()
                telem.send_heartbeat()
            end
        end
    end
end

@effect(io, timing)
op launch() =>
    let monitor = HeartbeatMonitor.spawn(interval: 1.seconds)

    for i in 0..100 =>
        if not monitor.check_heartbeat() =>
            console.write("Heartbeat lost!")
            enter_safe_mode()
        end
        time.sleep(500.ms)
    end

    monitor.shutdown()
end
```

### Example 8: Hardware and Telemetry

```
orbit main

dock core.hardware
dock mission.telemetry as telem

struct SensorReading =>
    sensor_id: Int
    raw_value: Uint16
    calibrated: Float64<kelvin>
    timestamp: Int
end

@effect(hardware, telemetry)
op read_thermal_sensor(id: Int) -> Result<SensorReading, SensorError> =>
    let raw = unsafe =>
        hardware.read_register_u16(THERMAL_BASE_ADDR + id * 2)
    end

    if raw == 0xFFFF =>
        return err(SensorError.HardwareFault(code: id))
    end

    let calibrated = calibrate_thermal(raw)

    let reading = SensorReading {
        sensor_id: id,
        raw_value: raw,
        calibrated: calibrated,
        timestamp: time.mission_clock(),
    }

    telem.log(reading)
    return ok(reading)
end
```

### Example 9: Fallback and Safe Mode

```
orbit main

dock core.time
dock mission.telemetry as telem

enum SystemMode =>
    Nominal
    Degraded(subsystem: Text)
    SafeMode
end

@effect(io, hardware, timing)
@watchdog(timeout: 10.seconds)
op main_loop() =>
    var mode = SystemMode.Nominal

    loop =>
        system.pet_watchdog()

        match mode =>
            SystemMode.Nominal =>
                match run_nominal_operations() =>
                    Result.Ok(()) => ()
                    Result.Err(e) =>
                        telem.send_alert("Degraded: " ++ e.display())
                        mode = SystemMode.Degraded(subsystem: e.subsystem())
                    end
                end
            end
            SystemMode.Degraded(sub) =>
                match try_recover(sub) =>
                    Result.Ok(()) =>
                        mode = SystemMode.Nominal
                    end
                    Result.Err(_) =>
                        telem.send_alert("Entering safe mode")
                        mode = SystemMode.SafeMode
                    end
                end
            end
            SystemMode.SafeMode =>
                maintain_safe_state()
                if check_ground_command_available() =>
                    match handle_recovery_command() =>
                        Result.Ok(()) => mode = SystemMode.Nominal
                        Result.Err(_) => ()
                    end
                end
            end
        end

        time.sleep(100.ms)
    end
end
```

### Example 10: Complete Small Program

```
--! A simple orbit calculator that computes Hohmann transfer parameters
--! between two circular orbits around Earth.

orbit main

dock core.math
dock core.units
dock core.fmt

const EARTH_MU: Float64<meters3_per_second2> = 3.986004418e14

struct TransferResult =>
    delta_v1: Float64<meters_per_second>
    delta_v2: Float64<meters_per_second>
    total_delta_v: Float64<meters_per_second>
    transfer_time: Float64<seconds>
end

impl Displayable for TransferResult =>
    op display(self) -> Text =>
        var out = "Hohmann Transfer:\n"
        out = out ++ "  Burn 1:    " ++ self.delta_v1.display() ++ " m/s\n"
        out = out ++ "  Burn 2:    " ++ self.delta_v2.display() ++ " m/s\n"
        out = out ++ "  Total dV:  " ++ self.total_delta_v.display() ++ " m/s\n"
        out = out ++ "  Duration:  " ++ (self.transfer_time / 3600.0).display() ++ " hours"
        return out
    end
end

op hohmann_transfer(
    r1: Float64<meters>,
    r2: Float64<meters>,
) -> TransferResult =>
    let a_transfer = (r1 + r2) / 2.0

    let v1 = math.sqrt(EARTH_MU / r1)
    let v2 = math.sqrt(EARTH_MU / r2)

    let v_transfer_1 = math.sqrt(EARTH_MU * (2.0 / r1 - 1.0 / a_transfer))
    let v_transfer_2 = math.sqrt(EARTH_MU * (2.0 / r2 - 1.0 / a_transfer))

    let dv1 = math.abs(v_transfer_1 - v1)
    let dv2 = math.abs(v2 - v_transfer_2)

    let t_transfer = math.PI * math.sqrt(
        a_transfer * a_transfer * a_transfer / EARTH_MU
    )

    return TransferResult {
        delta_v1: dv1,
        delta_v2: dv2,
        total_delta_v: dv1 + dv2,
        transfer_time: t_transfer,
    }
end

@effect(io)
op launch() =>
    let leo = 6_571_000.0    -- 200km altitude
    let geo = 42_164_000.0   -- GEO altitude

    let result = hohmann_transfer(leo, geo)
    console.write(result.display())
end
```

---

## 11. Formal Grammar Sketch

```
program         = module_decl? import* declaration*

module_decl     = "orbit" module_path
module_path     = IDENT ("." IDENT)*

import          = "dock" module_path ("as" IDENT)?
                | "dock" module_path ".{" IDENT ("," IDENT)* "}"

declaration     = function_decl
                | struct_decl
                | enum_decl
                | ability_decl
                | impl_decl
                | task_decl
                | task_body_decl
                | protected_decl
                | const_decl

function_decl   = annotation* "op" IDENT generics? "(" params? ")" ("->" type)? block

annotation      = "@" IDENT ( "(" annotation_args ")" )?

block           = "=>" statement* "end"
                | "=>" expression

params          = param ("," param)*
param           = "mut"? IDENT ":" type

generics        = "<" type_param ("," type_param)* ">"
type_param      = IDENT (":" type_bound)?
type_bound      = IDENT ("+" IDENT)*

type            = IDENT generics?
                | IDENT "<" unit_expr ">"
                | "[" type ";" INT "]"
                | "(" type ("," type)* ")"
                | "Slice<" type ">"
                | "Option<" type ">"
                | "Result<" type "," type ">"

struct_decl     = "struct" IDENT generics? "=>" field* "end"
field           = IDENT ":" type

enum_decl       = "enum" IDENT generics? "=>" variant* "end"
variant         = IDENT ("(" field ("," field)* ")")?

ability_decl    = "ability" IDENT generics? "=>" ability_item* "end"
ability_item    = "op" IDENT "(" params? ")" ("->" type)?
                | "const" IDENT ":" type

impl_decl       = "impl" IDENT "for" IDENT "=>" function_decl* "end"
                | "impl" IDENT "=>" function_decl* "end"

task_decl       = "task" IDENT "=>" task_entry* "end"
task_entry      = annotation* "op" IDENT "(" params? ")" ("->" type)?

task_body_decl  = "task" "body" IDENT "(" params? ")" "=>" statement* "end"

protected_decl  = "protected" IDENT "=>" protected_item* "end"
protected_item  = "var" IDENT ":" type "=" expression
                | function_decl

const_decl      = "const" IDENT ":" type "=" expression

statement       = let_stmt
                | var_stmt
                | assign_stmt
                | expr_stmt
                | if_stmt
                | match_stmt
                | for_stmt
                | while_stmt
                | loop_stmt
                | return_stmt
                | break_stmt
                | continue_stmt
                | select_stmt

let_stmt        = "let" IDENT (":" type)? "=" expression
var_stmt        = "var" IDENT (":" type)? "=" expression
assign_stmt     = place "=" expression
                | place "+=" expression
                | place "-=" expression

if_stmt         = "if" expression "=>" statement*
                  ("else" "if" expression "=>" statement*)*
                  ("else" "=>" statement*)?
                  "end"

match_stmt      = "match" expression "=>" match_arm* "end"
match_arm       = pattern "=>" statement* "end"
                | pattern "=>" expression

pattern         = IDENT "." IDENT ("(" pattern_fields ")")?
                | literal
                | IDENT
                | "_"

for_stmt        = "for" IDENT "in" expression "=>" statement* "end"
while_stmt      = "while" expression "=>" statement* "end"
loop_stmt       = "loop" "=>" statement* "end"

select_stmt     = "select" "=>"
                  ("accept" IDENT "(" params? ")" "=>" statement* "end"
                   | "or" ...)*
                  "end"

expression      = literal
                | IDENT
                | expression binop expression
                | unop expression
                | expression "(" args ")"
                | expression "." IDENT
                | expression "?" 
                | expression "!!" expression
                | expression ">>" expression
                | "(" expression ")"
                | block_expr
                | closure
                | struct_literal
                | unsafe_block

closure         = "|" params? "|" expression
                | "|" params? "|" block

struct_literal  = IDENT "{" (IDENT ":" expression ",")* "}"

binop           = "+" | "-" | "*" | "/" | "%"
                | "==" | "!=" | "<" | ">" | "<=" | ">="
                | "and" | "or"
                | "&" | "|" | "^" | "<<" | ">>"
                | "++"
                | ".."

unop            = "-" | "not" | "~"

literal         = INT | FLOAT | STRING | "true" | "false" | "none"
```

---

## 12. Runtime Model

### Compilation Pipeline

GalaxC is a compiled language. Source code is compiled ahead-of-time to native machine code via C as an intermediate representation:

```
.gxc source -> Lexer -> Parser -> AST -> Type Checker -> IR -> C Codegen -> CC -> Native Binary
```

### Compile-Time Checks

The compiler performs the following checks before any code runs:
- Type correctness
- Ownership and borrowing validity
- Exhaustive pattern matching
- Error handling completeness
- Effect annotation consistency
- Unit dimensional analysis
- Integer overflow potential (where provable)
- Dead code detection
- Unused variable/import warnings
- Mutability violations
- Task entry signature compatibility

### Runtime Checks

Minimal runtime overhead for maximum safety:
- Array bounds checking (eliminable by proof in future versions)
- Integer overflow checking (for checked arithmetic)
- Watchdog timer management
- Task scheduling
- Checkpoint serialization

### Performance

GalaxC generates clean, idiomatic C code. Performance characteristics:
- No garbage collector overhead
- No virtual dispatch by default (static dispatch via monomorphization)
- Predictable memory layout (no hidden fields, no vtables unless explicitly using dynamic dispatch)
- Stack allocation by default; heap allocation is explicit
- Zero-cost abstractions: generics, abilities, and pattern matching have no runtime overhead after compilation

### Determinism

In safe code:
- No allocation without explicit effect annotation
- No I/O without explicit effect annotation
- No concurrency without explicit task declarations
- Floating-point follows IEEE 754 strictly (no fast-math by default)
- Evaluation order is defined (left-to-right, top-to-bottom)

---

## 13. Tooling

### Compiler (`galaxc`)

```
galaxc build <file.gxc>       -- compile to native binary
galaxc check <file.gxc>       -- type check only (no codegen)
galaxc emit-c <file.gxc>      -- emit generated C code
galaxc emit-ir <file.gxc>     -- emit intermediate representation
galaxc run <file.gxc>         -- compile and run in one step
galaxc fmt <file.gxc>         -- format source code
galaxc test <path>             -- run test suite
galaxc init <name>             -- create new project
galaxc version                 -- print version info
```

### Debugger (`galaxc-dbg`)

Interactive TUI debugger with:
- Source-level breakpoints (line, function, conditional)
- Step in, step out, step over, continue
- Variable inspection with full GalaxC type display
- Stack trace with source mapping
- Watch expressions
- Task state viewer (see all active tasks, their states, pending entries)
- Protected object viewer (see current readers/writers)
- Memory inspector
- Telemetry stream viewer
- Timeline execution view
- REPL for evaluating GalaxC expressions during stopped execution
- Checkpoint browser (view and restore checkpoints)

### Formatter (`galaxc fmt`)

Opinionated, deterministic formatter. One canonical way to format GalaxC code. No configuration beyond line width.

### Linter

Built into the compiler. Warns about:
- Unused variables, imports, and functions
- Unreachable code
- Redundant pattern arms
- Inefficient patterns
- Missing documentation on public items
- Overly complex functions (cyclomatic complexity)

### Documentation Generator

`galaxc doc` generates HTML documentation from `---` doc comments, including:
- Module hierarchy
- Type signatures
- Cross-references
- Code examples (tested)

### Test Runner

`galaxc test` discovers and runs:
- Unit tests (annotated with `@test`)
- Integration tests (in `tests/` directory)
- Doc tests (code in documentation comments)
- Property-based tests (with `@property_test`)

```
@test
op test_addition() =>
    assert_eq(add(2, 3), 5)
end

@test
op test_sensor_timeout() =>
    let result = read_sensor_with_timeout(0.ms)
    assert(result.is_err())
end
```

---

## 14. Compatibility Strategy

### C Interoperability

GalaxC compiles to C and can call C functions directly via `extern` declarations:

```
extern "C" =>
    op memcpy(dest: RawPtr, src: RawPtr, n: Int) -> RawPtr
    op printf(fmt: RawPtr, ...) -> Int
end
```

All `extern` calls require an `unsafe` block at the call site. The GalaxC compiler generates standard C headers for GalaxC functions marked `pub extern`, allowing C code to call GalaxC code.

### Embedded Systems

GalaxC is designed for embedded targets:
- No standard library dependency required (bare-metal mode with `@no_std`)
- Configurable allocator (or no allocator -- `@no_alloc`)
- Direct register access via `hardware` module
- Linker script support
- Cross-compilation via C compiler toolchains

### What Is Not Supported

- No C++ interop (C++ ABI is too complex and unstable)
- No Python interop (different execution model)
- No JVM/CLR interop (irrelevant for mission-critical systems)

---

## 15. Version 0.1 Design Decisions

### Included in v0.1

- Complete lexer and parser for all described syntax
- Type checking for primitive types, structs, enums, and generics
- Pattern matching with exhaustiveness checking
- Option and Result types
- Ownership and borrowing (basic model)
- Effect annotations (parsed and checked)
- C code generation for core subset
- CLI with build, check, emit-c, run, fmt commands
- Interactive TUI debugger with breakpoints, stepping, and variable inspection
- Standard library: core types, basic math, basic I/O
- 10+ working examples
- Comprehensive test suite

### Deferred to Later Versions

- Full unit-safe numeric type checking (parser supports syntax; checker warns but permits)
- Full optimizer pipeline
- Task runtime implementation (syntax and checks are in; runtime scheduler is stub)
- Protected object runtime (syntax and checks are in; locking is stub)
- Checkpoint runtime
- Watchdog runtime integration
- Incremental compilation
- IDE language server (LSP)
- Package manager
- Property-based test generation
- WASM target
- Full documentation generator

### Design Constraints

This is a real compiler, not a prototype. The architecture is designed for extension. Adding features to a well-structured compiler is straightforward; fixing a badly-structured one wastes years.

---

## 16. Final Identity

**Name:** GalaxC (Galaxy Code)

**Tagline:** *Code that survives the void.*

**File Extension:** `.gxc`

**Entry Point:** The `launch()` function in the `main` orbit.

### Hello World

```
orbit main

@effect(io)
op launch() =>
    console.write("GalaxC online. All systems nominal.")
end
```

### Safe Mode Demo

```
orbit main

dock core.time
dock mission.telemetry as telem

@effect(io, timing)
@watchdog(timeout: 10.seconds)
op launch() =>
    var mode = SystemMode.Nominal

    loop =>
        system.pet_watchdog()

        match mode =>
            SystemMode.Nominal =>
                match run_all_checks() =>
                    Result.Ok(()) => telem.send_status("nominal")
                    Result.Err(e) =>
                        telem.send_alert(e.display())
                        mode = SystemMode.SafeMode
                    end
                end
            end
            SystemMode.SafeMode =>
                maintain_minimal_operations()
                if ground_link_available() =>
                    mode = await_ground_recovery()?
                end
            end
        end

        time.sleep(100.ms)
    end
end

enum SystemMode =>
    Nominal
    SafeMode
end
```

---

## Building from Source

### Prerequisites

- Rust toolchain (1.70 or later)
- A C compiler (GCC, Clang, or MSVC)

### Build

```
cargo build --release
```

### Install

```
cargo install --path crates/galaxc-cli
cargo install --path crates/galaxc-dbg
```

### Test

```
cargo test
```

### Usage

```
galaxc build examples/hello.gxc
./hello

galaxc run examples/orbit_calculator.gxc

galaxc check examples/mission_loop.gxc

galaxc-dbg examples/hello.gxc
```

---

## Project Structure

```
GalaxC/
  Cargo.toml               Workspace root
  README.md                 This document
  LICENSE                   MIT License
  .gitignore                Build artifact exclusions
  crates/
    galaxc/                 Core compiler library
      src/
        lib.rs              Library root
        lexer/              Tokenizer
        ast/                Abstract syntax tree definitions
        parser/             Recursive descent parser
        types/              Type system
        checker/            Type checker and validation
        ir/                 Intermediate representation
        codegen/            C code generator
        diagnostics/        Error reporting
    galaxc-cli/             Command-line interface
      src/
        main.rs             CLI entry point
    galaxc-dbg/             Interactive debugger
      src/
        main.rs             Debugger entry point
  stdlib/                   Standard library (GalaxC source)
  examples/                 Example programs
  tests/                    Integration tests
  docs/                     Additional documentation
```

---

*GalaxC is engineered for the missions where failure is not an option and silence is not an answer.*
