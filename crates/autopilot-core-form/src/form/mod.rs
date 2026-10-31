//! UI-agnostic form model built from JSON schemas.
//!
//! [`FormSchema`] describes fields and widget hints, [`FormValues`] holds raw
//! values keyed by field, and [`FormSchema::validate`] checks required fields.
//! [`ConditionConfig`] bridges typed configs to forms.

mod config;
mod error;
mod parse;
pub mod providers;
mod schema;
mod validate;
mod values;

pub use config::ConditionConfig;
pub use error::FormError;
pub use schema::{AUTOPILOT_HINT, FormField, FormFieldKind, FormOption, FormSchema};
pub use values::FormValues;

#[cfg(test)]
#[path = "form.spec.rs"]
mod spec;
