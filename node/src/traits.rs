use crate::{builder::ComponentsBuilder, components::FullNodeTypes};

pub trait Node<N: FullNodeTypes> {
    type ComponentsBuilder;
}
