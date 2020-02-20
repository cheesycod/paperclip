//! Paperclip is a OpenAPI code generator for efficient type-safe
//! compile-time checked HTTP APIs in Rust.
//!
//! See the [website](https://paperclip.waffles.space) for detailed
//! documentation and examples.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

mod error;
#[cfg(feature = "v2")]
pub mod v2;

pub use error::{PaperClipError, PaperClipResult};
#[cfg(feature = "v2")]
pub use paperclip_macros::api_v2_schema_struct as api_v2_schema;

#[cfg(feature = "actix")]
pub mod actix {
    //! Plugin types, traits and macros for actix-web framework.

    pub use paperclip_actix::{api_v2_operation, api_v2_schema};
    pub use paperclip_actix::{web, App, Mountable, OpenApiExt};
    pub use paperclip_core::v2::ResponderWrapper;
}
