//! Single source of truth for the condition registry.
//!
//! Every condition scheme is listed once here, and the macro generates:
//!
//! - the form-type list and form key lookup,
//! - form schema/values plumbing,
//! - conversion to the runtime `Condition` trait object.
//!
//! The `ConditionScheme` enum itself stays in `mod.rs` because it carries the
//! documentation and serde attributes. Adding a variant there without listing
//! it here fails to compile, because the generated matches must be exhaustive.
//!
//! [`ConditionConfig`] implementations live in `autopilot-core-form` (via its
//! `impl_condition_config!` macro), next to the trait they implement.

/// Generates the form registry for [`ConditionScheme`].
///
/// - `form`: conditions that can be created and edited through a form.
/// - `no_form`: conditions that only exist in JSON and cannot be built from a
///   form (currently only `Fail`).
macro_rules! condition_registry {
    (
        form {
            $(
                $variant:ident => {
                    key: $key:literal,
                    label: $label:literal,
                    scheme: $scheme:ty,
                    condition: $condition:ty,
                };
            )+
        }
        no_form {
            $(
                $no_form_variant:ident => {
                    key: $no_form_key:literal,
                };
            )+
        }
    ) => {
        impl ConditionScheme {
            /// Condition types exposed by form UIs, in enum order.
            pub fn form_types() -> Vec<ConditionFormType> {
                vec![
                    $(
                        ConditionFormType {
                            key: $key,
                            label: $label,
                        },
                    )+
                ]
            }

            /// Form registry key for this condition.
            pub fn form_type_key(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant(_) => $key,
                    )+
                    $(
                        Self::$no_form_variant(_) => $no_form_key,
                    )+
                }
            }

            /// Serializes the condition into form values.
            ///
            /// Conditions without a form are rejected.
            pub fn to_form_values(&self) -> Result<FormValues, FormError> {
                match self {
                    $(
                        Self::$variant(scheme) => scheme.to_form_values(),
                    )+
                    $(
                        Self::$no_form_variant(_) => Err(FormError::Unsupported(format!(
                            "{} conditions cannot be created from a form",
                            $no_form_key
                        ))),
                    )+
                }
            }

            /// Builds the form schema for a form registry key.
            pub fn form_schema(key: &str) -> Result<FormSchema, FormError> {
                match key {
                    $(
                        $key => <$scheme>::try_form_schema(),
                    )+
                    other => Err(FormError::Unsupported(format!(
                        "unknown condition form type '{other}'"
                    ))),
                }
            }

            /// Builds a condition from a form registry key and its values.
            pub fn from_form_values(key: &str, values: FormValues) -> Result<Self, FormError> {
                match key {
                    $(
                        $key => Ok(Self::$variant(<$scheme>::from_form_values(values)?)),
                    )+
                    other => Err(FormError::Unsupported(format!(
                        "unknown condition form type '{other}'"
                    ))),
                }
            }

            /// Converts the scheme into the runtime condition trait object.
            pub fn to_condition(&self) -> Box<dyn Condition> {
                match self {
                    $(
                        Self::$variant(scheme) => {
                            Box::new(<$condition>::from_scheme(scheme.clone()))
                        },
                    )+
                    $(
                        Self::$no_form_variant(scheme) => Box::new(scheme.clone()),
                    )+
                }
            }
        }
    };
}

pub(crate) use condition_registry;
