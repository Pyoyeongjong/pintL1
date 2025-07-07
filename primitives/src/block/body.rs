use crate::block::header::SealedHeader;

#[derive(Debug)]
pub struct BlockBody<T> {
    pub transaction: Vec<T>,
}

impl<T> crate::block::traits::BlockBody for BlockBody<T> {}
pub struct SealedBlock<B: crate::block::traits::Block> {
    header: SealedHeader<B::Header>,
    body: B::Body,
}
