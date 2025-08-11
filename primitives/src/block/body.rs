use crate::block::{header::SealedHeader, traits::Block};

#[derive(Debug)]
pub struct BlockBody<T> {
    pub transaction: Vec<T>,
}

impl<T> crate::block::traits::BlockBody for BlockBody<T> {}

#[derive(Clone)]
pub struct SealedBlock<B: Block> {
    header: SealedHeader<B::Header>,
    body: B::Body,
}
