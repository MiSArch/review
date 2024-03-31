use std::any::type_name;

use crate::{
    base_connection::{BaseConnection, FindResultWrapper}, order_datatypes::ReviewOrderInput, product::Product, product_variant::ProductVariant, review_connection::ReviewConnection, user::User, Review
};
use async_graphql::{Context, Error, Object, Result};

use bson::{Document, Uuid};
use mongodb::{bson::doc, options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::Deserialize;

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
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_object(&collection, id).await
    }

    /// Entity resolver for product of specific id.
    #[graphql(entity)]
    async fn product_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of product to retrieve.")] id: Uuid,
    ) -> Result<Product> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Product> =
            db_client.collection::<Product>("products");
        query_object(&collection, id).await
    }

    /// Entity resolver for product variant of specific id.
    #[graphql(entity)]
    async fn product_variant_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of product variant to retrieve.")] id: Uuid,
    ) -> Result<ProductVariant> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<ProductVariant> =
            db_client.collection::<ProductVariant>("product_variants");
        query_object(&collection, id).await
    }

    /// Retrieves all reviews.
    async fn reviews<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "Describes that the `first` N reviews should be retrieved.")]
        first: Option<u32>,
        #[graphql(desc = "Describes how many reviews should be skipped at the beginning.")]
        skip: Option<u64>,
        #[graphql(desc = "Specifies the order in which reviews are retrieved.")] order_by: Option<
            ReviewOrderInput,
        >,
    ) -> Result<ReviewConnection> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        let review_order = order_by.unwrap_or_default();
        let sorting_doc = doc! {review_order.field.unwrap_or_default().as_str(): i32::from(review_order.direction.unwrap_or_default())};
        let find_options = FindOptions::builder()
            .skip(skip)
            .limit(first.map(|v| i64::from(v)))
            .sort(sorting_doc)
            .build();
        let document_collection = collection.clone_with_type::<Document>();
        let maybe_find_results: Result<FindResult<Review>, CursorError> =
            PaginatedCursor::new(Some(find_options.clone()), None, None)
                .find(&document_collection, None)
                .await;
        match maybe_find_results {
            Ok(find_results) => {
                let find_result_wrapper = FindResultWrapper(find_results);
                let connection = Into::<BaseConnection<Review>>::into(find_result_wrapper);
                Ok(Into::<ReviewConnection>::into(connection))
            }
            Err(_) => return Err(Error::new("Retrieving reviews failed in MongoDB.")),
        }
    }

    /// Retrieves review of specific id.
    async fn review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of review to retrieve.")] id: Uuid,
    ) -> Result<Review> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        query_object(&collection, id).await
    }
}

/// Shared function to query an object: T from a MongoDB collection of object: T.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of object.
pub async fn query_object<T: for<'a> Deserialize<'a> + Unpin + Send + Sync>(
    collection: &Collection<T>,
    id: Uuid,
) -> Result<T> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_object) => match maybe_object {
            Some(object) => Ok(object),
            None => {
                let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
            Err(Error::new(message))
        }
    }
}
