---
source: Context7 API (actix.rs official site)
library: Actix Web
package: actix-web
topic: app_data, middleware, route configuration, extractors
fetched: 2026-04-14T00:00:00Z
official_docs: https://actix.rs/docs/
---

# Actix Web — App Data, Middleware & Route Configuration

## Version Info

| Crate | Latest Stable |
|---|---|
| `actix-web` | **4.13.0** |

```toml
[dependencies]
actix-web = "4.13.0"
serde = { version = "1", features = ["derive"] }
env_logger = "0.11"
```

## App Data (Application State)

Use `web::Data<T>` to share application state across routes. **Important**: Use `App::app_data()` (NOT the deprecated `App::data()`).

### Basic State Setup

```rust
use actix_web::{get, web, App, HttpServer};

struct AppState {
    app_name: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Extractor Configuration via app_data

```rust
// Configure JSON extractor limits and error handling
let json_config = web::JsonConfig::default()
    .limit(4096)
    .error_handler(|err, _req| {
        error::InternalError::from_response(err, HttpResponse::Conflict().finish())
            .into()
    });

App::new().service(
    web::resource("/")
        .app_data(json_config)
        .route(web::post().to(index)),
)
```

### Migration Note (v3 → v4)

```diff
  use actix_web::web::Data;

- App::new()
-     .data(MyState::default())           // DEPRECATED
-     .service(handler)

+ let my_state: Data<MyState> = Data::new(MyState::default());
+ App::new()
+     .app_data(my_state)                 // CORRECT in v4
+     .service(handler)
```

## Route Configuration

### Attribute Macros

```rust
use actix_web::{get, post, web, HttpResponse, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
```

### Manual Route Configuration

```rust
App::new()
    .route("/", web::get().to(hello))
    .route("/echo", web::post().to(echo))
```

### Resource with Guards

```rust
use actix_web::{guard, web, App, HttpResponse};

App::new()
    .service(web::resource("/prefix").to(index))
    .service(
        web::resource("/user/{name}")
            .name("user_detail")
            .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(HttpResponse::Ok))
            .route(web::put().to(HttpResponse::Ok)),
    );
```

### Route with Multiple Guards

```rust
App::new().service(
    web::resource("/path").route(
        web::route()
            .guard(guard::Get())
            .guard(guard::Header("content-type", "text/plain"))
            .to(HttpResponse::Ok),
    ),
)
```

### Modular Configuration with `configure()`

```rust
use actix_web::{web, App, HttpResponse, HttpServer};

fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(config)
            .service(web::scope("/api").configure(scoped_config))
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("/") }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## Extractors

### JSON Body

```rust
use actix_web::{web, App, HttpServer, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    username: String,
}

async fn index(info: web::Json<Info>) -> Result<String> {
    Ok(format!("Welcome {}!", info.username))
}
```

### URL-Encoded Form

```rust
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

#[post("/login")]
async fn login(form: web::Form<LoginForm>) -> impl Responder {
    HttpResponse::Ok().body(format!("Welcome, {}!", form.username))
}

// Configure form extraction limit
let form_config = web::FormConfig::default().limit(16_384); // 16KB
App::new()
    .app_data(form_config)
    .service(login)
```

## Middleware

### Logger Middleware

```rust
use actix_web::middleware::Logger;
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Error Handlers Middleware

```rust
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{dev, http::{header, StatusCode}, web, App, HttpResponse, HttpServer, Result};

fn add_error_header<B>(mut res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

HttpServer::new(|| {
    App::new()
        .wrap(
            ErrorHandlers::new()
                .handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header),
        )
        .service(web::resource("/").route(web::get().to(HttpResponse::InternalServerError)))
})
```

### Custom Error Types with ResponseError

```rust
use actix_web::{error, http::{header::ContentType, StatusCode}, HttpResponse};
use derive_more::derive::{Display, Error};

#[derive(Debug, Display, Error)]
enum MyError {
    #[display("internal error")]
    InternalError,
    #[display("bad request")]
    BadClientData,
    #[display("timeout")]
    Timeout,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}
```
