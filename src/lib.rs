//! PHP serialization format support

#![warn(
    unused_qualifications,
    variant_size_differences,
    clippy::checked_conversions,
    clippy::needless_borrow,
    clippy::shadow_unrelated,
    clippy::wrong_pub_self_convention
)]
#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::as_conversions,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::indexing_slicing,
    clippy::option_unwrap_used,
    clippy::result_unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        unreachable_code,
        unused_qualifications
    )
)]
#![cfg_attr(not(debug_assertions), deny(warnings, missing_docs, clippy::dbg_macro))]

mod error;
pub use error::*;

mod str_trait;
pub use str_trait::*;

mod source;
pub use source::*;

mod types;
pub use types::*;

mod parse;
pub use parse::*;

mod emit;
pub use emit::*;
