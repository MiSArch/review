use async_graphql::{Context, Error, Object, Result};
use bson::Bson;
use bson::Uuid;
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};

use crate::product_variant::ProductVariant;
use crate::query::query_user;
use crate::user::User;
use crate::user_mutation::UserMutation;
use crate::{
    mutation_input_structs::{AddReviewInput, UpdateReviewInput},
    query::query_review,
    review::Review,
};

/// Describes GraphQL review mutations.
pub struct Mutation;

#[Object]
impl Mutation {
    /// Entity resolver for user of specific id.
    //#[graphql(entity)]
    async fn user<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of user to retrieve.")] id: Uuid,
    ) -> Result<UserMutation> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<User> = db_client.collection::<User>("users");
        let user = query_user(&collection, id).await?;
        Ok(UserMutation { _id: user._id })
    }
}