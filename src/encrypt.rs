use std::{rc::Rc, borrow::Cow};

use bc_components::{SymmetricKey, Nonce, Digest, DigestProvider, tags_registry::LEAF};
use dcbor::{CBOREncodable, CBORTaggedDecodable, CBORTaggedEncodable, CBOR, tagged};

use crate::{Envelope, Error, envelope::new_envelope_with_unchecked_assertions};

impl Envelope {
    /// Returns a new envelope with its subject encrypted.
    ///
    /// Assertions are not encrypted. To encrypt an entire envelope including its
    /// assertions it must first be wrapped using the ``wrap()`` method.
    ///
    /// - Parameters:
    ///   - key: The `SymmetricKey` to be used to encrypt the subject.
    ///
    /// - Returns: The encrypted envelope.
    ///
    /// - Throws: If the envelope is already encrypted.
    pub fn encrypt_subject(self: Rc<Self>, key: &SymmetricKey) -> Result<Rc<Self>, Error> {
        self.encrypt_subject_opt(key, None)
    }

    pub fn encrypt_subject_opt(self: Rc<Self>, key: &SymmetricKey, test_nonce: Option<Nonce>) -> Result<Rc<Self>, Error> {
        let result: Rc<Envelope>;
        let original_digest: Cow<Digest>;

        match &*self {
            Envelope::Node { subject, assertions, digest: envelope_digest } => {
                if subject.is_encrypted() {
                    return Err(Error::AlreadyEncrypted);
                }
                let encoded_cbor = subject.cbor_data();
                let digest = subject.digest();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                let encrypted_subject = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                result = Rc::new(new_envelope_with_unchecked_assertions(encrypted_subject, assertions.clone()));
                original_digest = Cow::Borrowed(envelope_digest);
            }
            Envelope::Leaf { cbor, digest } => {
                let encoded_cbor = tagged(LEAF, cbor.clone()).cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = Cow::Borrowed(digest);
            }
            Envelope::Wrapped { digest, .. } => {
                let encoded_cbor = self.untagged_cbor().cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = Cow::Borrowed(digest);
            }
            Envelope::KnownValue { value, digest } => {
                let encoded_cbor = value.cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = Cow::Borrowed(digest);
            }
            Envelope::Assertion(assertion) => {
                let digest = assertion.digest();
                let encoded_cbor = assertion.cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = digest;
            }
            Envelope::Encrypted { .. } => {
                return Err(Error::AlreadyEncrypted);
            }
            Envelope::Compressed(compressed) => {
                let digest = compressed.digest();
                let encoded_cbor = compressed.tagged_cbor().cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = digest;
            }
            Envelope::Elided { .. } => {
                return Err(Error::AlreadyElided);
            }
        }
        assert_eq!(result.digest(), original_digest);
        Ok(result)
    }

    pub fn decrypt_subject(self: Rc<Self>, key: &SymmetricKey) -> Result<Rc<Self>, Error> {
        match &*self.clone().subject() {
            Envelope::Encrypted(message) => {
                let encoded_cbor = key.decrypt(message).map_err(Error::CryptoError)?;
                let subject_digest = message.opt_digest().ok_or(Error::MissingDigest)?;
                let cbor = CBOR::from_data(&encoded_cbor).map_err(Error::CBORError)?;
                let result_subject = Rc::new(Self::from_untagged_cbor(&cbor).map_err(Error::CBORError)?).subject();
                if *result_subject.digest() != subject_digest {
                    return Err(Error::InvalidDigest);
                }
                match &*self {
                    Envelope::Node { assertions, digest, .. } => {
                        let result = Rc::new(new_envelope_with_unchecked_assertions(result_subject, assertions.clone()));
                        if *result.digest() != *digest {
                            return Err(Error::InvalidDigest);
                        }
                        Ok(result)
                    }
                    _ => Ok(result_subject)
                }
            },
            _ => Err(Error::NotEncrypted)
        }
    }
}