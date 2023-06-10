#![feature(iter_intersperse)]

mod assertions;
mod cbor;
mod compress;
mod digest;
mod encrypt;
mod expression;
mod functions;
mod known_values;
mod parameters;
mod queries;
mod recipient;
mod salt;
mod signature;
mod string_utils;
mod tree_format;
mod wrap;

mod format;
pub use format::{EnvelopeFormat, EnvelopeFormatItem};

mod sskr;
pub use sskr::{SSKRShare, SSKRSpec, SSKRGroupSpec, SSKRSecret};

mod into_envelope;
pub use into_envelope::IntoEnvelope;

mod elide;
pub use elide::ObscureAction;

mod envelope;
pub use crate::envelope::Envelope;

mod format_context;
pub use format_context::{FormatContext, FORMAT_CONTEXT};

mod error;
pub use error::Error;

mod walk;
pub use walk::{EdgeType, Visitor};

mod assertion;
pub use assertion::Assertion;

pub mod known_value;
pub use known_value::KnownValue;

pub mod function;
pub use function::Function;

pub mod parameter;
pub use parameter::Parameter;

mod known_values_store;
pub use known_values_store::KnownValuesStore;

mod functions_store;
pub use functions_store::FunctionsStore;

mod parameters_store;
pub use parameters_store::ParametersStore;

#[cfg(test)]
mod tests {
    pub mod test_data;
    mod seed;
    pub use seed::Seed;

    mod check_encoding;
    mod compression_tests;
    mod core_encoding_tests;
    mod core_nesting_tests;
    mod core_tests;
    mod crypto_tests;
    mod elision_tests;
    mod encrypted_tests;
    mod format_tests;
    mod function_tests;
    mod non_correlation_tests;
    mod obscuring_tests;
    mod type_tests;

    use bc_crypto::hash::sha256;

    #[test]
    fn it_works() {
        let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
        let result = hex::encode(sha256(input.as_bytes()));
        assert_eq!(result, expected);
    }
}
