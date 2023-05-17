use std::rc::Rc;

use bc_components::tags_registry;
use dcbor::{CBORTagged, Tag, CBOREncodable, CBORTaggedEncodable, CBOR, CBORDecodable, CBORTaggedDecodable, CBORError};

use crate::KnownFunctions;
pub use crate::function_registry::*;

#[derive(Clone, Debug, Eq)]
pub enum FunctionName {
    Static(&'static str),
    Dynamic(String),
}

impl FunctionName {
    fn value(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Dynamic(name) => name,
        }
    }
}

impl PartialEq for FunctionName {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl std::hash::Hash for FunctionName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state)
    }
}

 #[derive(Debug, Clone, Eq)]
pub enum Function {
    Known(u64, Option<FunctionName>),
    Named(FunctionName),
}

impl Function {
    pub fn new_known(value: u64, name: Option<String>) -> Self {
        Self::Known(value, name.map(|name| FunctionName::Dynamic(name)))
    }

    pub fn new_named(name: String) -> Self {
        Self::Named(FunctionName::Dynamic(name))
    }

    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self::Known(value, Some(FunctionName::Static(name)))
    }

    pub fn name(&self) -> String {
        match self {
            Self::Known(value, name) => {
                if let Some(name) = name {
                    name.value().to_string()
                } else {
                    value.to_string()
                }
            },
            Self::Named(name) => name.value().to_string(),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Known(l, _), Function::Known(r, _)) => l == r,
            (Function::Named(l), Function::Named(r)) => l == r,
            _ => false,
        }
    }
}

impl std::hash::Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Function::Known(value, _) => value.hash(state),
            Function::Named(name) => name.hash(state),
        }
    }
}

impl From<u64> for Function {
    fn from(value: u64) -> Self {
        Self::new_known(value, None)
    }
}

impl From<&str> for Function {
    fn from(name: &str) -> Self {
        Self::new_named(name.to_string())
    }
}

impl CBORTagged for Function {
    const CBOR_TAG: Tag = tags_registry::FUNCTION;
}

impl CBOREncodable for Function {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORTaggedEncodable for Function {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Function::Known(value, _) => value.cbor(),
            Function::Named(name) => name.value().cbor(),
        }
    }
}

impl CBORDecodable for Function {
    fn from_cbor(cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for Function {
    fn from_untagged_cbor(untagged_cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        match untagged_cbor {
            CBOR::Unsigned(value) => Ok(Rc::new(Self::new_known(*value, None))),
            CBOR::Text(name) => Ok(Rc::new(Self::new_named(name.clone()))),
            _ => Err(CBORError::InvalidFormat),
        }
    }
}

impl Function {
    fn description(&self, known_functions: Option<&KnownFunctions>) -> String {
        match self {
            Function::Known(_, _) => {
                KnownFunctions::name_for_function(self, known_functions)
            },
            Function::Named(name) => {
                format!("\"{}\"", name.value())
            },
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}