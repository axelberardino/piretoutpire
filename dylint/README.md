# Backend-rs dynamic lint

Linting rules for Rust code.
This directory shouldn't be included into the workspace directory and should be moved into a monorepo where it can live and be consumed by every other rust repository.

You'll be able to run it with `cargo backendctl clippy`.

## Requirement

You're going to need `cargo-dylint` and `dylint-link` available to be able to run the Rust linting tool, you can install them with `cargo install cargo-dylint dylint-link`.

## How to disable rules

To disable rules, you'll have to write something like this:

```
#[allow(unknown_lints)]
#[allow(one_letter_variable)]
```

Why? Because `#![register_tools]` are still unstable [Tracking issue](https://github.com/rust-lang/rust/issues/66079)

## How to test it

the incremental compilation is broken, so to test your dynamic linter
use this command for now

```sh
dylint> find target/ -name "*backend_rs_lint*" -delete; cargo test
```

## Rules

### build_rerun_proto

**What it does:**
Lint to prevent the build of tonic without setting the proto_builder::rerun_if_changed

**Why is this bad?**
When changing the proto files if you don't force a rerun you could have some issues on the CI

**Known problems:** 
None.

**Example:**
```rust
use std::{
     env,
     path::{Path, PathBuf},
 };
 fn main() {
     let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR env variable"));
     let proto_dir = Path::new("../../../platform/libraries/proto/proto");
     let protodefs = proto_dir.join("github.com/znly/protodefs");
     proto_builder::rerun_if_changed(protodefs).expect("could'nt walk through directories");
     tonic_build::configure()
         .build_server(true)
         .build_client(true)
         .file_descriptor_set_path(out_dir.join("service_descriptor.bin"))
         .compile(...)
         .expect("unable to compile service");
 }
```

### cql_statement

**What it does:**
Lint CQLStatement implementation ensures that code making calls to Scylla are using the CQL Observer

**Why is this bad?**
If we don't use the CQL Observer we can't track metrics of the calls made to Scylla and therefore we are
on the fog regarding what's happening.

**Known problems:** None.

**Example:**

```rust
use scylla::statement::prepared_statement::PreparedStatement;

pub struct MyStruct {
    query: PreparedStatement,
}
```

Use instead:

```rust
use drivers::reexports::scylladb::QueryStatement;

pub struct MyStruct {
    query: QueryStatement,
}
```

### impl_for_collection

**What it does:**
Lint to ensure that implementation from a type T into a Collection is what you really want to do.

**Why is this bad?**
Implementation from a type T into a Collection should be avoided, instead you should
probably implement FromIterator

**Known problems:** None.

**Example:**

```rust
struct Foo(T);

impl From<T> for Vec<T> {
    fn from(value: T) -> Vec<T> {
        ...etc
    }
}
```

### from_variable_value

**What it does:**
Searches for implementations of the `From<..>` and `TryFrom<..> trait and check if the variable's name is `value`or suggests to name it`value` instead.

**Why is this bad?**
It's not bad to do it otherwise but it's a convention.

**Known problems:** None.

**Example:**

```rust
struct Duration(String);

impl TryFrom<Duration> for String {
    type Error = AnyError;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
      unreachable!()
    }
}
```

Use instead:

```rust
struct Duration(String);

impl TryFrom<Duration> for String {
    type Error = AnyError;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
      unreachable!()
    }
}
```

### impl_type

**What it does:**
Lint From/TryFrom implementation over collections to ensure it's really what you want to do.

**Why is this bad?**
A From/TryFrom implementation over collections should be avoided

**Known problems:** None.

**Example:**

```rust
struct Duration(Vec<String>);

impl From<Vec<T>> for T {
    fn from(value: Vec<T>>) -> T {
        ...etc
    }
}
```

### impl_deref

**What it does:**
Lint Deref implementation to ensure it's really what you want to do.

**Why is this bad?**
A Deref implementation is not bad, but should be managed properly and implemented only for
pointer-like struct.

**Known problems:** None.

**Example:**

```rust
struct Duration(String);

impl Deref for Duration {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}
```

### one_letter_variable

**What it does:**
Lint one-character variables and suggest to change it to a more understandable name.

**Why is this bad?**
It's better to avoid having a one-character variable and have instead a more understandable
variable. It's a convention.

**Known problems:** None.

**Example:**
```rust
fn print(v: i32) {
    println!("{}", v);
}
```

Use instead:

```rust
fn print(value: i32) {
    println!("{}", value);
}
```

### try_from_tonic_error

**What it does:**
Lint TryFrom implementation to prevent Error type to be a tonic::Status.

**Why is this bad?**
We should never use a tonic::Status Error type for TryFrom implementation.
It should be an AnyError or a typed error.
When you TryFrom your struct `T` into `AnyResult<U>` and you are in a "grpc service" context
you should map_err your AnyError/TypedError into a tonic::Status rather than returning a tonic::Status.
This will provide the possibility to reuse this impl for other context than gprc's one.
When doing a conversion type, you should not introduce an hard dependency on something very specific.
Implementing a tonic::status error type lead your crate to depend closely to tonic. But your crate should remains generic, and not force the user to use any protocol.
Instead, it should be a generic AnyError or a typed error. It's the role of the caller to handle grpc error on top of your generic error.
This will provide the possibility to reuse this impl for other context than gprc's one and remove a dependency that might not be necessary.

**Known problems:** None.

**Example:**

```rust
impl TryFrom<T> for U {
    type Error = tonic::Status;

    fn try_from(value: T) -> Result<Self, Self::Error> {
            value
                .parse::<U>()
                .map_err(|_err| tonic::Status::invalid_argument("invalid T"))
                .map(Into::into)
    }
}
```

Use instead:

```rust
impl TryFrom<T> for U {
    type Error = AnyError;

    fn try_from(value: T) -> Result<Self, Self::Error> {
            value
                .parse::<U>()
                .map(Into::into)
    }
}
```
