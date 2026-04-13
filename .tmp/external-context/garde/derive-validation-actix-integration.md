---
source: docs.rs + crates.io
library: Garde
package: garde
topic: Derive macro usage, custom validation, actix-web integration
fetched: 2026-04-14T00:00:00Z
official_docs: https://docs.rs/garde/latest/garde/
---

# Garde — Derive Validation & Actix-Web Integration

## Version Info

| Crate | Latest Stable |
|---|---|
| `garde` | **0.22.1** |
| `garde-actix-web` | **0.12.0** |

```toml
[dependencies]
garde = { version = "0.22.1", features = ["derive"] }
# For actix-web integration:
garde-actix-web = "0.12.0"
```

### Feature Flags

| Feature | Description |
|---|---|
| `derive` | Enables `derive(Validate)` macro |
| `url` | URL validation via `url` crate |
| `email` | Email validation (HTML5 spec) |
| `email-idna` | IDNA support in emails |
| `regex` | Regular expression patterns |
| `credit-card` | Credit card number validation |
| `phone-number` | Phone number validation |
| `unicode` | Grapheme count validation |
| `full` | All of the above |

## Basic Derive Usage

```rust
use garde::{Validate, Valid};

#[derive(Validate)]
struct User<'a> {
    #[garde(ascii, length(min=3, max=25))]
    username: &'a str,
    #[garde(length(min=15))]
    password: &'a str,
}

let user = User {
    username: "test",
    password: "not_a_very_good_password",
};

if let Err(e) = user.validate() {
    println!("invalid user: {e}");
}
```

### Enum Validation

```rust
#[derive(Validate)]
enum Data {
    Struct {
        #[garde(range(min=-10, max=10))]
        field: i32,
    },
    Tuple(
        #[garde(ascii)]
        String
    ),
}
```

## Available Validation Rules

| Rule | Attribute | Notes |
|---|---|---|
| required | `#[garde(required)]` | Only for `Option` fields |
| ascii | `#[garde(ascii)]` | ASCII-only content |
| alphanumeric | `#[garde(alphanumeric)]` | Letters and digits only |
| email | `#[garde(email)]` | Requires `email` feature |
| url | `#[garde(url)]` | Requires `url` feature |
| ip / ipv4 / ipv6 | `#[garde(ip)]` | IP address validation |
| credit_card | `#[garde(credit_card)]` | Requires `credit-card` feature |
| phone_number | `#[garde(phone_number)]` | Requires `phone-number` feature |
| length | `#[garde(length(min=N, max=N))]` | Container/string length |
| matches | `#[garde(matches(field))]` | Field equality check |
| range | `#[garde(range(min=N, max=N))]` | Numeric range |
| contains | `#[garde(contains("str"))]` | Substring check |
| prefix | `#[garde(prefix("str"))]` | String prefix |
| suffix | `#[garde(suffix("str"))]` | String suffix |
| pattern | `#[garde(pattern("regex"))]` | Regex match (requires `regex`) |
| dive | `#[garde(dive)]` | Nested validation |
| skip | `#[garde(skip)]` | Skip validation |
| custom | `#[garde(custom(fn))]` | Custom validator function |

## Custom Validation Rules

### Basic Custom Validator

```rust
#[derive(garde::Validate)]
#[garde(context(PasswordContext))]
struct User {
    #[garde(custom(is_strong_password))]
    password: String,
}

struct PasswordContext {
    min_entropy: f32,
    entropy: cracken::password_entropy::EntropyEstimator,
}

fn is_strong_password(value: &str, context: &PasswordContext) -> garde::Result {
    let bits = context.entropy.estimate_password_entropy(value.as_bytes())
        .map(|e| e.mask_entropy)
        .unwrap_or(0.0);
    if bits < context.min_entropy {
        return Err(garde::Error::new("password is not strong enough"));
    }
    Ok(())
}

let ctx = PasswordContext { /* ... */ };
let user = User { /* ... */ };
user.validate(&ctx)?;
```

### Higher-Order Custom Validators

```rust
fn my_equals(other: &str) -> impl FnOnce(&str, &()) -> garde::Result + '_ {
    move |value, _| {
        if value != other {
            return Err(garde::Error::new(format!("not equal to {other}")));
        }
        Ok(())
    }
}

#[derive(garde::Validate)]
struct User {
    #[garde(length(min = 1, max = 255))]
    password: String,
    #[garde(custom(my_equals(&self.password)))]
    password2: String,
}
```

### Context with Self Access

```rust
struct Limits {
    min: usize,
    max: usize,
}

struct Config {
    username: Limits,
}

#[derive(garde::Validate)]
#[garde(context(Config as ctx))]
struct User {
    #[garde(length(min = ctx.username.min, max = ctx.username.max))]
    username: String,
}
```

## Handling Option Fields

```rust
#[derive(garde::Validate)]
struct Test {
    #[garde(required, ascii, length(min = 1))]
    value: Option<String>,
}
// Fails if: value is None, inner is empty, or inner contains non-ASCII
```

## Inner Type Validation (Collections)

```rust
#[derive(garde::Validate)]
struct Test {
    #[garde(
        length(min = 1),                    // validates Vec length
        inner(ascii, length(min = 1)),      // validates each String
    )]
    items: Vec<String>,
}
```

## Newtype Pattern

```rust
#[derive(garde::Validate)]
#[garde(transparent)]
struct Username(#[garde(length(min = 3, max = 20))] String);

#[derive(garde::Validate)]
struct User {
    #[garde(dive)]
    username: Username,
}
```

## Skip All with `allow_unvalidated`

```rust
#[derive(garde::Validate)]
#[garde(allow_unvalidated)]
struct Bar<'a> {
    #[garde(length(min = 1))]
    a: &'a str,
    b: &'a str,  // NOT validated, no #[garde(skip)] needed
}
```

---

## Integration with Actix-Web (`garde-actix-web`)

### Installation

```toml
[dependencies]
garde = { version = "0.22", features = ["derive"] }
garde-actix-web = "0.12"
```

### Compatibility Matrix

| garde | garde-actix-web |
|---|---|
| 0.22 | 0.12.x |
| 0.20 | 0.9.x, 0.10.x |
| 0.19 | 0.8.x |
| 0.18 | 0.5.x – 0.7.x |

### Usage — Drop-in Replacement for Actix Extractors

Use `garde_actix_web::web::*` types as drop-in replacements for `actix_web::web::*`. Validation happens automatically during `FromRequest`.

- Invalid payload → **400 Bad Request** (or **404** for `Path`)
- Custom error handling via extractor configs (e.g., `garde_actix_web::web::QueryConfig`)

```rust
use actix_web::HttpResponse;
use garde_actix_web::web::Path;  // instead of actix_web::web::Path
use garde::Validate;

#[derive(Validate)]
struct MyStruct<'a> {
    #[garde(ascii, length(min=3, max=25))]
    username: &'a str,
}

fn test(id: Path<MyStruct>) -> HttpResponse {
    todo!()
}
```

### Available Drop-in Extractors

| Actix Extractor | Garde Equivalent |
|---|---|
| `actix_web::web::Path<T>` | `garde_actix_web::web::Path<T>` |
| `actix_web::web::Query<T>` | `garde_actix_web::web::Query<T>` |
| `actix_web::web::Json<T>` | `garde_actix_web::web::Json<T>` |
| `actix_web::web::Form<T>` | `garde_actix_web::web::Form<T>` |

### ⚠️ Important: Context Must Implement Default

When using garde [custom validation](https://github.com/jprochazk/garde#custom-validation) with `garde-actix-web`, the `Context` type **must implement `Default`**, which is not required by garde alone.

### Feature Flags

| Feature | Description |
|---|---|
| `serde_qs` | Enables garde validation for `serde_qs::actix::QsQuery<T>` |
