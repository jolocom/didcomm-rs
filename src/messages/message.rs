use serde::{
    Serialize,
    Deserialize
};
use super::{
    headers::{DidcommHeader, JwmHeader},
    prior_claims::PriorClaims,
    };
use crate::Error;

/// DIDComm message structure.
/// [Specification](https://identity.foundation/didcomm-messaging/spec/#message-structure)
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(flatten)]
    jwm_header: JwmHeader,
    #[serde(flatten)]
    didcomm_header: DidcommHeader,
    body: Vec<u8>,    
}

impl Message {
    /// Generates EMPTY default message.
    /// Use extension messages to build final one before `send`ing.
    ///
    pub fn new() -> Self {
        Message {
            jwm_header: JwmHeader::default(),
            didcomm_header: DidcommHeader::new(),
            body: vec!(),
        }
    }
    /// Checks if message is rotation one.
    /// Exposed for explicit checks on sdk level.
    ///
    pub fn is_rotation(&self) -> bool {
        self.didcomm_header.from_prior().is_some()
    }
    /// If message `is_rotation()` true - returns from_prion claims.
    /// Errors otherwise with `Error::NoRotationData`
    /// 
    pub fn get_prior(&self) -> Result<PriorClaims, Error> {
        if self.is_rotation() {
            Ok(self.didcomm_header.from_prior().clone().unwrap())
        } else {
           Err(Error::NoRotationData)
        }
    }
    /// `&DidcommHeader` getter.
    ///
    pub fn get_didcomm_header(&self) -> &DidcommHeader {
        &self.didcomm_header
    }
    /// `&JwmHeader` getter.
    ///
    pub fn get_jwm_header(&self) -> &JwmHeader {
        &self.jwm_header
    }
    /// `&Vec<u8>` of `Message`'s body.
    ///
    pub fn get_body(&self) -> &Vec<u8> {
        &self.body
    }
    /// Creates set of Jwm related headers for the JWE
    /// Modifies JWM related header portion to match
    ///     encryption implementation and leaves other
    ///     parts unchanged.  TODO + FIXME: complete implementation
    pub fn as_jws(self) -> Self {
        Self { 
            jwm_header: self.jwm_header.as_a256_gcm(None, None), 
            ..self
        }
    }
    /// Creates set of Jwm related headers for the JWS
    /// Modifies JWM related header portion to match
    ///     signature implementation and leaves Other
    ///     parts unchanged.
    /// TODO + FIXME: complete implementation
    pub fn as_jwe(self) -> Self {
        Self {
            jwm_header: self.jwm_header.as_es256(None, None),
            ..self
        }
    }
    /// Seals self and returns ready to send JWE
    ///
    /// # Parameters
    ///
    /// `ek` - encryption key for inner message payload JWE encryption
    /// TODO: Add example[s]
    pub fn seal(self, ek: Vec<u8>) -> Result<String, Error> {
        todo!()
    }
    /// Signs raw message and then packs it to encrypted envelope
    /// [Spec](https://identity.foundation/didcomm-messaging/spec/#message-signing)
    ///
    /// # Parameters
    ///
    /// `ek` - encryption key for inner message payload JWE encryption
    ///
    /// `sk` - signing key for enveloped message JWS encryption
    /// TODO: Adde example[s]
    pub fn seal_signed(self, ek: &[u8], sk: &[u8]) -> Result<String, Error> {
        todo!()
        //let mut crypto_envelope = Message {
        //    headers: Headers::encrypt_jws(self.headers.clone())?,
        //    body: self.sign_compact_jws(&sk)?.as_bytes().to_vec()
        //};
        //crypto_envelope.pack_compact_jwe(&ek)
    }
    /// Wrap self to be mediated by some mediator.
    /// Takes one mediator at a time to make sure that mediated chain preserves unchanged.
    /// This method can be chained any number of times to match all the mediators in the chain.
    ///
    /// # Parameters
    ///
    /// `ek` - encryption key for inner message payload JWE encryption
    ///
    /// `to` - list of destination recepients. can be empty (Optional) `String::default()`
    ///
    /// `form` - sender identifyer `String`
    ///
    /// `expires_time` - `Option<usize>` seconds from the UTC Epoch seconds,
    ///     signals when the message is no longer valid, and is to be used by
    ///     the recipient to discard expired messages on receipt
    /// TODO: Add example[s]
    pub fn routed_by(self,
        ek: Vec<u8>,
        to: Vec<String>,
        from: String,
        expires_time: Option<usize>)
        -> Result<Self, Error> {
            todo!()
       // let payload = self.pack_compact_jwe(&ek)?;
       // let forward_headers = DidcommHeader::forward(to, from, expires_time)?;
       // let mut packed = Message::new();
       // packed.headers = forward_headers;
       // packed.body = payload.as_bytes().to_vec();
       // Ok(packed)
    }
}

/// Associated functions implementations.
impl Message {
    pub fn get_iv(received: &[u8]) -> Result<Vec<u8>, Error> {
        // parse from compact
        let as_str = String::from_utf8(received.to_vec())?;
        let json: serde_json::Value =
            if let Some(header_end) = as_str.find('.') {
                    serde_json::from_str(&String::from_utf8(base64_url::decode(&as_str[..header_end])?)?)?
            } else {
                serde_json::from_str(&as_str)?
            };
        if let Some(iv) = json.get("iv") {
            if let Some(t) = iv.as_str() {
                let bytes = base64_url::decode(t)?; 
                if bytes.len() != 24usize {
                    Err(Error::Generic(format!("wrong nonce (iv) size: {}", bytes.len())))
                } else {
                    Ok(bytes)
                }
            } else { Err(Error::Other("wrong nonce format".into())) }
        } else {
            Err(Error::Generic("failed to parse iv from JOSE header".into()))
        }
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use crate::Error;

    #[test]
    fn iv_from_json_test() -> Result<(), Error> {
        // Arrange
        let h = JwmHeader::default();
        // Example JWM from RFC: https://tools.ietf.org/html/draft-looker-jwm-01#section-2.3
        let raw_json = r#" { "protected": "eyJ0eXAiOiJKV00iLCJlbmMiOiJBMjU2R0NNIiwia2lkIjoiUEdvWHpzME5XYVJfbWVLZ1RaTGJFdURvU1ZUYUZ1eXJiV0k3VjlkcGpDZyIsImFsZyI6IkVDREgtRVMrQTI1NktXIiwiZXBrIjp7Imt0eSI6IkVDIiwiY3J2IjoiUC0yNTYiLCJ4IjoiLU5oN1NoUkJfeGFDQlpSZElpVkN1bDNTb1IwWXc0VEdFUXFxR2lqMXZKcyIsInkiOiI5dEx4ODFQTWZRa3JPdzh5dUkyWXdJMG83TXROemFDR2ZDQmJaQlc1WXJNIn19",
                "recipients": [
                  {
                    "encrypted_key": "J1Fs9JaDjOT_5481ORQWfEZmHy7OjE3pTNKccnK7hlqjxbPalQWWLg"
                  }
                ],
                "iv": "u5kIzo0m_d2PjI4mu5kIzo0m_d2PjI4m",
                "ciphertext": "qGuFFoHy7HBmkf2BaY6eREwzEjn6O_FnRoXj2H-DAXo1PgQdfON-_1QbxtnT8e8z_M6Gown7s8fLtYNmIHAuixqFQnSA4fdMcMSi02z1MYEn2JC-1EkVbWr4TqQgFP1EyymB6XjCWDiwTYd2xpKoUshu8WW601HLSgFIRUG3-cK_ZSdFaoWosIgAH5EQ2ayJkRB_7dXuo9Bi1MK6TYGZKezc6rpCK_VRSnLXhFwa1C3T0QBes",
                "tag": "doeAoagwJe9BwKayfcduiw"
            }"#;
        // Act
        let iv = Message::get_iv(raw_json.as_bytes())?;
        println!("{:?}", iv);
        // Assert
        Ok(())
    }
}

#[cfg(test)]
mod crypto_tests {
    extern crate chacha20poly1305;
    extern crate sodiumoxide;
    extern crate x25519_dalek;

   // use crate::Error;
   // use super::*;

}
