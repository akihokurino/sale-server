use rand::random;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub mod product;
pub mod time;
pub mod user;

#[derive(Debug, Ord, PartialOrd, Clone)]
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
}
impl<E> From<String> for Id<E> {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}
impl<E> Into<String> for Id<E> {
    fn into(self) -> String {
        self.id
    }
}
impl<E> Hash for Id<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
impl<E> Eq for Id<E> {}
impl<E> PartialEq<Self> for Id<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

pub fn generate_id_str() -> String {
    base_62::encode(&random::<[u8; 16]>())
}
