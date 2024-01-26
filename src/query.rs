use crate::{product_variant::ProductVariant, user::User, Review};
use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, Collection, Database};

/// Describes GraphQL review queries.
pub struct Query;

#[Object]
impl Query {
    /// Entity resolver for user of specific id.
    #[graphql(entity)]
    async fn user_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of user to retrieve.")] id: Uuid,
    ) -> Result<User> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_user(&collection, id).await
    }

    /// Entity resolver for product variant of specific id.
    #[graphql(entity)]
    async fn product_variant_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of product variant to retrieve.")] id: Uuid,
    ) -> Result<ProductVariant> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ProductVariant> = db_client.collection::<ProductVariant>("product_variants");
        query_product_variant(&collection, id).await
    }

    /// Retrieves review of specific id.
    async fn review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of review to retrieve.")] id: Uuid,
    ) -> Result<Review> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        query_review(&collection, id).await
    }

    /// Entity resolver for review of specific id.
    #[graphql(entity)]
    async fn review_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(key, desc = "UUID of review to retrieve.")] id: Uuid,
    ) -> Result<Review> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        query_review(&collection, id).await
    }
}

/// Shared function to query a review from a MongoDB collection of reviews
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of review.
pub async fn query_review(collection: &Collection<Review>, id: Uuid) -> Result<Review> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_review) => match maybe_review {
            Some(review) => Ok(review),
            None => {
                let message = format!("Review with UUID id: `{}` not found.", id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("Review with UUID id: `{}` not found.", id);
            Err(Error::new(message))
        }
    }
}

/// Shared function to query a user from a MongoDB collection of users.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of user.
pub async fn query_user(collection: &Collection<User>, id: Uuid) -> Result<User> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => Ok(user),
            None => {
                let message = format!("User with UUID id: `{}` not found.", id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("User with UUID id: `{}` not found.", id);
            Err(Error::new(message))
        }
    }
}

/// Shared function to query a product variant from a MongoDB collection of product variants.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of product variant.
pub async fn query_product_variant(collection: &Collection<ProductVariant>, id: Uuid) -> Result<ProductVariant> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_product_variant) => match maybe_product_variant {
            Some(product_variant) => Ok(product_variant),
            None => {
                let message = format!("ProductVariant with UUID id: `{}` not found.", id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("ProductVariant with UUID id: `{}` not found.", id);
            Err(Error::new(message))
        }
    }
}
