use rand::random;
use std::marker::PhantomData;

pub mod product;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Id<E> {
    id: String,
    _phantom: PhantomData<E>,
}
impl<E> Id<E> {
    pub fn new<I: Into<String>>(id: I) -> Self {
        Self {
            id: id.into(),
            _phantom: PhantomData,
        }
    }

    pub fn generate() -> Self {
        Self::new(generate_id_str())
    }

    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }

    pub fn to_string(&self) -> String {
        self.id.clone()
    }
}
impl<E> From<String> for Id<E> {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

pub fn generate_id_str() -> String {
    base_62::encode(&random::<[u8; 16]>())
}
