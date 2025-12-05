use pqcrypto_mldsa::mldsa44;
use pqcrypto_traits::sign::{PublicKey as PkTrait, SecretKey as SkTrait, DetachedSignature as DsTrait};
use crate::types::HybridTx;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Post-quantum signature verification failed")]
    PqVerifyFailed,
    #[error("Malformed key or signature")]
    Malformed,
}

/// Post-Quantum Keypair
pub struct PqKeypair {
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
}

impl PqKeypair {
    pub fn generate() -> Self {
        let (pk, sk) = mldsa44::keypair();
        Self {
            public: pk.as_bytes().to_vec(),
            secret: sk.as_bytes().to_vec(),
        }
    }

    pub fn public_key(&self) -> Result<mldsa44::PublicKey, CryptoError> {
        mldsa44::PublicKey::from_bytes(&self.public).map_err(|_| CryptoError::Malformed)
    }

    pub fn secret_key(&self) -> Result<mldsa44::SecretKey, CryptoError> {
        mldsa44::SecretKey::from_bytes(&self.secret).map_err(|_| CryptoError::Malformed)
    }
}

/// Sign a message with the PQ secret key
pub fn sign_pq_message(secret: &mldsa44::SecretKey, msg: &[u8]) -> Vec<u8> {
    let sig = mldsa44::sign_detached(msg, secret);
    sig.as_bytes().to_vec()
}

/// Verify a PQ signature on a HybridTx
pub fn verify_pq_tx(tx: &HybridTx) -> Result<(), CryptoError> {
    let pq_sig = tx.pq_signature.as_ref().ok_or(CryptoError::Malformed)?;
    let pq_pk = tx.pq_pubkey.as_ref().ok_or(CryptoError::Malformed)?;

    let pk = mldsa44::PublicKey::from_bytes(pq_pk).map_err(|_| CryptoError::Malformed)?;
    let sig = mldsa44::DetachedSignature::from_bytes(pq_sig).map_err(|_| CryptoError::Malformed)?;

    let msg = bincode::serialize(&tx.body).map_err(|_| CryptoError::Malformed)?;

    mldsa44::verify_detached_signature(&sig, &msg, &pk)
        .map_err(|_| CryptoError::PqVerifyFailed)?;

    Ok(())
}
