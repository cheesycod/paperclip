# Host OpenAPI v2 spec through actix-web

With `actix` feature enabled, paperclip exports an **experimental** plugin for [actix-web](https://github.com/actix/actix-web) framework to host OpenAPI v2 spec for your APIs *automatically*. While it's not feature complete, you can rely on it to not break your actix-web flow.

Let's start with a simple actix-web application. It has `actix-web` and `serde` for JSON'ifying your APIs. Let's also add `paperclip` with `actix` feature.

```toml
# [package] ignored for brevity

[dependencies]
actix-web = "2.0"
paperclip = { git = "https://github.com/wafflespeanut/paperclip", features = ["actix"] }
serde = "1.0"
```

Our `main.rs` looks like this:

```rust
use actix_web::{App, web::{self, Json}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Pet {
    name: String,
    id: Option<i64>,
}

fn add_pet(_body: Json<Pet>) -> Json<Pet> {
    unimplemented!();
}

fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| App::new()
        .service(
            web::resource("/pets")
                .route(web::post().to(add_pet))
        )
    ).bind("127.0.0.1:8080")?
    .run()
}
```

Now, let's modify it to use the plugin!

```rust
use actix_web::{App, HttpServer};
use paperclip::actix::{
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt, api_v2_schema, api_v2_operation,
    // use this instead of actix_web::web
    web::{self, Json},
};
use serde::{Serialize, Deserialize};

// Mark containers (body, query, parameter, etc.) like so...
#[api_v2_schema]
#[derive(Serialize, Deserialize)]
struct Pet {
    name: String,
    id: Option<i64>,
}

// Mark operations like so...
#[api_v2_operation]
fn add_pet(_body: Json<Pet>) -> Json<Pet> {
    unimplemented!();
}

fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new()
        // Record services and routes from this line.
        .wrap_api()
        // Add routes like you normally do...
        .service(
            web::resource("/pets")
                .route(web::post().to(add_pet))
        )
        // Mount the JSON spec at this path.
        .with_json_spec_at("/api/spec")

        // ... or if you wish to build the spec by yourself...

        // .with_raw_json_spec(|app, spec| {
        //     app.route("/api/spec", web::get().to(move || {
        //         actix_web::HttpResponse::Ok().json(&spec)
        //     }))
        // })

        // IMPORTANT: Build the app!
        .build()
    ).bind("127.0.0.1:8080")?
    .run()
}
```

We have:

 - Imported `OpenApiExt` extension trait for `actix_web::App` along with `api_v2_schema` and `api_v2_operation` proc macro attributes.
 - Switched from `actix_web::web` to `paperclip::actix::web`.
 - Marked our `Pet` struct and `add_pet` function as OpenAPI-compatible schema and operation using proc macro attributes.
 - Transformed our `actix_web::App` to a wrapper using `.wrap_api()`.
 - Mounted the JSON spec at a relative path using `.with_json_spec_at("/api/spec")`.
 - Built (using `.build()`) and passed the original `App` back to actix-web.

Note that we never touched the service, resources, routes or anything else! This means that our original actix-web flow is unchanged.

Now, testing with **cURL**:

```
curl http://localhost:8080/api/spec
```

... we get the swagger spec as a JSON!

```js
// NOTE: Formatted for clarity
{
  "swagger": "2.0",
  "definitions": {
    "Pet": {
      "properties": {
        "id": {
          "type": "integer",
          "format": "int64"
        },
        "name": {
          "type": "string"
        }
      },
      "required": ["name"]
    }
  },
  "paths": {
    "/pets": {
      "post": {
        "responses": {
          "200": {
            "schema": {
              "$ref": "#/definitions/Pet"
            }
          }
        }
      },
      "parameters": [{
        "in": "body",
        "name": "body",
        "required": true,
        "schema": {
          "$ref": "#/definitions/Pet"
        }
      }]
    }
  }
}
```

Similarly, if we were to use other extractors like `web::Query<T>`, `web::Form<T>` or `web::Path`, the plugin will emit the corresponding specification as expected.

#### Manually defining additional response codes

There is a macro `api_v2_errors` which helps to manually add responses other than 200.

```rust
use paperclip::actix::api_v2_errors;

#[api_v2_errors(
    code=400,
    code=401, description="Unauthorized: Can't read session from header",
    code=500,
)]
pub enum MyError {
    /* ... */
}
```

#### Known limitations

- **Enums:** OpenAPI (v2) itself supports using simple enums (i.e., with unit variants), but Rust and serde has support for variants with fields and tuples. I still haven't looked deep enough either to argue that this cannot be done in OpenAPI or find an elegant way to represent this in OpenAPI.
- **Functions returning abstractions:** The plugin has no way to obtain any useful information from functions returning abstractions such as `HttpResponse`, `impl Responder` or containers such as `Result<T, E>` containing those abstractions. So, the plugin silently ignores these types, which results in an empty value in your hosted specification.

#### Missing features

At the time of this writing, this plugin didn't support a few OpenAPI v2 features:

Affected entity | Missing feature(s)
--------------- | ---------------
[Parameter](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/2.0.md#parameter-object) | Non-body parameters allowing validations like `allowEmptyValue`, `collectionFormat`, `items`, etc.
[Parameter](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/2.0.md#parameter-object) | Headers as parameters.
[Responses](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/2.0.md#responsesObject) | Response codes other than `200 OK`.
Security ([definitions](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/2.0.md#securityDefinitionsObject) and [requirements](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/2.0.md#securityRequirementObject)) | Authentication and Authorization.

#### Performance implications?

Even though we use some wrappers and generate schema structs for building the spec, we do this only once i.e., until the `.build()` function call. At runtime, it's basically an [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) deref and [`RwLock`](https://docs.rs/parking_lot/*/parking_lot/type.RwLock.html) read, which is quite fast!

We also add a wrapper to blocks in functions tagged with `#[api_v2_operation]`, but those wrappers are unit structs and the code eventually gets optimized away anyway.
