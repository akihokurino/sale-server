use crate::domain;

pub type Id = domain::Id<User>;
#[derive(Debug, Clone)]
pub struct User {
    pub id: Id,
}
