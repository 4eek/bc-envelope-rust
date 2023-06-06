use std::rc::Rc;
use bc_components::{Digest, Compressed, EncryptedMessage, DigestProvider};
use dcbor::{CBOR, CBOREncodable};
use crate::{assertion::Assertion, KnownValue, Error};

/// A flexible container for structured data.
///
/// Envelopes are immutable. You create "mutations" by creating new envelopes from old envelopes.
#[derive(Clone, Debug)]
pub enum Envelope {
    /// Represents an envelope with one or more assertions.
    Node { subject: Rc<Self>, assertions: Vec<Rc<Self>>, digest: Digest },

    /// Represents an envelope with encoded CBOR data.
    Leaf { cbor: CBOR, digest: Digest },

    /// Represents an envelope that wraps another envelope.
    Wrapped { envelope: Rc<Self>, digest: Digest },

    /// Represents a value from a namespace of unsigned integers.
    KnownValue { value: KnownValue, digest: Digest },

    /// Represents an assertion.
    ///
    /// An assertion is a predicate-object pair, each of which is itself an ``Envelope``.
    Assertion(Assertion),

    /// Represents an encrypted envelope.
    Encrypted(EncryptedMessage),

    /// Represents a compressed envelope.
    Compressed(Compressed),

    /// Represents an elided envelope.
    Elided(Digest),
}

impl Envelope {
    /// Create an Envelope from a &dyn CBOREncodable
    pub fn from_cbor_encodable(cbor_encodable: &dyn CBOREncodable) -> Rc<Self> {
        let cbor = cbor_encodable.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Rc::new(Envelope::Leaf {
            cbor,
            digest,
        })
    }
}

pub trait Enclosable {
    fn enclose(self) -> Rc<Envelope>;
}

impl Enclosable for Rc<Envelope> {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_wrapped(self))
    }
}

impl Enclosable for Envelope {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_wrapped(Rc::new(self)))
    }
}

impl Enclosable for KnownValue {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_known_value(self))
    }
}

impl Enclosable for Assertion {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_assertion(self))
    }
}

impl Enclosable for EncryptedMessage {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_encrypted(self).unwrap())
    }
}

impl Enclosable for Compressed {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_compressed(self).unwrap())
    }
}

impl Enclosable for CBOR {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self))
    }
}

impl Enclosable for &str {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(CBOR::Text(self.to_string())))
    }
}

impl Enclosable for u8 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for u16 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for u32 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for u64 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for usize {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i8 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i16 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i32 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i64 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

pub fn new_envelope_with_unchecked_assertions(subject: Rc<Envelope>, unchecked_assertions: Vec<Rc<Envelope>>) -> Envelope {
    assert!(!unchecked_assertions.is_empty());
    let mut sorted_assertions = unchecked_assertions;
    sorted_assertions.sort_by(|a, b| a.digest().cmp(&b.digest()));
    let mut digests = vec![subject.digest().into_owned()];
    digests.extend(sorted_assertions.iter().map(|a| a.digest().into_owned()));
    let digest = Digest::from_digests(&digests);
    Envelope::Node { subject, assertions: sorted_assertions, digest }
}

/// Internal constructors
impl Envelope {
    pub(crate) fn new_with_assertions(subject: Rc<Self>, assertions: Vec<Rc<Self>>) -> Result<Self, Error> {
        if !assertions.iter().all(|a| a.is_subject_assertion() || a.is_subject_obscured()) {
            return Err(Error::InvalidFormat);
        }
        Ok(new_envelope_with_unchecked_assertions(subject, assertions))
    }

    pub(crate) fn new_with_assertion(assertion: Assertion) -> Self {
        Self::Assertion(assertion)
    }

    pub(crate) fn new_with_known_value(value: KnownValue) -> Self {
        let digest = value.digest().into_owned();
        Self::KnownValue { value, digest }
    }

    pub(crate) fn new_with_encrypted(encrypted_message: EncryptedMessage) -> Result<Self, Error> {
        if !encrypted_message.has_digest() {
            return Err(Error::MissingDigest);
        }
        Ok(Self::Encrypted(encrypted_message))
    }

    pub(crate) fn new_with_compressed(compressed: Compressed) -> Result<Self, Error> {
        if !compressed.has_digest() {
            return Err(Error::MissingDigest);
        }
        Ok(Self::Compressed(compressed))
    }

    pub(crate) fn new_elided(digest: Digest) -> Self {
        Self::Elided(digest)
    }

    pub(crate) fn new_leaf<T: CBOREncodable>(cbor: T) -> Self {
        let cbor = cbor.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Self::Leaf { cbor, digest }
    }

    pub(crate) fn new_wrapped(envelope: Rc<Self>) -> Self {
        let digest = Digest::from_digests(&[envelope.digest().into_owned()]);
        Self::Wrapped { envelope, digest }
    }
}

impl Envelope {
    pub fn new_assertion_with_predobj<P: Enclosable + 'static, O: Enclosable + 'static>(predicate: P, object: O) -> Rc<Self> {
        Rc::new(Self::new_with_assertion(Assertion::new(predicate, object)))
    }
}

#[cfg(test)]
mod tests {
    use bc_components::{DigestProvider, Compressed};
    use crate::{Envelope, KnownValue, Assertion, envelope::Enclosable};

    #[test]
    fn test_any_envelope() {
        let e1 = Envelope::new_leaf("Hello");
        let e2 = "Hello".enclose();
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_known_value() {
        let known_value = KnownValue::new(100);
        let e1 = Envelope::new_with_known_value(known_value.clone());
        let e2 = known_value.enclose();
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_assertion() {
        let assertion = Assertion::new("knows", "Bob");
        let e1 = Envelope::new_with_assertion(assertion.clone());
        let e2 = assertion.enclose();
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_encrypted() {
        //todo!()
    }

    #[test]
    fn test_any_compressed() {
        let data = "Hello".as_bytes();
        let digest = data.digest().into_owned();
        let compressed = Compressed::from_uncompressed_data(data, Some(digest));
        let e1 = Envelope::new_with_compressed(compressed.clone()).unwrap();
        let e2 = compressed.enclose();
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_cbor_encodable() {
        let e1 = Envelope::new_leaf(1);
        let e2 = 1.enclose();
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }
}
