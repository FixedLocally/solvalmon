use solana_sdk::{pubkey::Pubkey, signature::Signature};

pub fn check_sig(pubkey: Pubkey, sig: &str, str_to_sign: &str) -> bool {
    sig.parse::<Signature>().map_or_else(|_| false, |sig| {
        sig.verify(&pubkey.to_bytes(), str_to_sign.as_bytes())
    })
}