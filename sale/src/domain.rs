use std::marker::PhantomData;

pub mod product;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Id<E> {
    id: String,
    _phantom: PhantomData<E>,
}
