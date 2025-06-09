
#[derive(Debug)]
pub enum SignatureError {
    InvalidParity(u64),
    FromHex(hex::FromHexError),
}

impl From<hex::FromHexError> for SignatureError {
    fn from(err: hex::FromHexError) -> Self {
        Self::FromHex(err)
    }
}

