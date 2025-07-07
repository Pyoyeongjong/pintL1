use crate::block::{body::BlockBody, header::Header};

// For Disk Storage
pub mod body;
pub mod header;
pub mod traits;
#[derive(Debug)]
pub struct Block<T, H = Header> {
    pub header: H,
    pub body: BlockBody<T>,
}

impl<T, H> Block<T, H> {
    pub const fn new(header: H, body: BlockBody<T>) -> Self {
        Self { header, body }
    }

    pub fn into_header(self) -> H {
        self.header
    }

    pub fn into_body(self) -> BlockBody<T> {
        self.body
    }
}

impl<T> crate::block::traits::Block for Block<T> {
    type Header = Header;

    type Body = BlockBody<T>;
}
