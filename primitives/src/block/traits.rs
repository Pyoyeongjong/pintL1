pub trait Block {
    type Header: BlockHeader;
    type Body: BlockBody;
}
pub trait BlockHeader {}
pub trait BlockBody {}
