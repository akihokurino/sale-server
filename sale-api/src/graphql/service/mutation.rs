use crate::graphql::errors;
use async_graphql::{Context, MergedObject, Object};

#[derive(MergedObject, Default)]
pub struct MutationRoot(DefaultMutation);

#[derive(Default)]
pub struct DefaultMutation;
#[Object]
impl DefaultMutation {
    async fn todo(&self, _ctx: &Context<'_>) -> Result<bool, errors::Error> {
        todo!()
    }
}
