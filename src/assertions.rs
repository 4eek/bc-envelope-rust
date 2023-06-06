use std::rc::Rc;

use bc_components::DigestProvider;

use crate::{Envelope, Error, envelope::{new_envelope_with_unchecked_assertions, Enclosable}};

// Support for manipulating assertions.

impl Envelope {
    pub fn add_assertion(self: Rc<Self>, assertion: Rc<Envelope>) -> Rc<Envelope> {
        self.add_assertion_opt(Some(assertion), false).unwrap()
    }

    pub fn add_assertion_opt(self: Rc<Self>, assertion: Option<Rc<Envelope>>, salted: bool) -> Result<Rc<Envelope>, Error> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    return Err(Error::InvalidFormat)
                }
                let envelope2 = if salted {
                    assertion.add_salt()
                } else {
                    assertion
                };

                match &*self {
                    Envelope::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest() == envelope2.digest()) {
                            let mut assertions = assertions.clone();
                            assertions.push(envelope2);
                            Ok(Rc::new(new_envelope_with_unchecked_assertions(subject.clone(), assertions)))
                        } else {
                            Ok(self)
                        }
                    },
                    _ => Ok(Rc::new(new_envelope_with_unchecked_assertions(self.subject(), vec![envelope2]))),
                }
            },
            None => Ok(self),
        }
    }

    pub fn add_assertion_with_predobj_salted<P: Enclosable + 'static, O: Enclosable + 'static>(self: Rc<Self>, predicate: P, object: O, salted: bool) -> Rc<Envelope>
    {
        let assertion = Envelope::new_assertion_with_predobj(predicate, object);
        self.add_assertion_opt(Some(assertion), salted).unwrap()
    }

    pub fn add_assertion_with_predobj<P: Enclosable + 'static, O: Enclosable + 'static>(self: Rc<Self>, predicate: P, object: O) -> Rc<Envelope> {
        self.add_assertion_with_predobj_salted(predicate, object, false)
    }
}
