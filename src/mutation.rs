use std::collections::HashSet;

use async_graphql::{Context, Error, Object, Result};
use bson::Bson;
use bson::Uuid;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};

use crate::product_variant::ProductVariant;
use crate::query::query_user;
use crate::user::User;
use crate::{
    mutation_input_structs::{AddReviewInput, UpdateReviewInput},
    query::query_review,
    review::Review,
};

/// Describes GraphQL review mutations.
pub struct Mutation;

#[Object]
impl Mutation {
    /// Adds a review.
    async fn add_review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "AddReviewInput")] input: AddReviewInput,
    ) -> Result<Review> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        validate_input(db_client, &input).await?;
        let current_timestamp = DateTime::now();
        let review = Review {
            _id: Uuid::new(),
            user: User { _id: input.user_id },
            product_variant: ProductVariant { _id: input.product_variant_id },
            body: input.body,
            rating: input.rating,
            created_at: current_timestamp,
            is_visible: input.is_visible.unwrap_or(true),
        };
        match collection.insert_one(review, None).await {
            Ok(result) => {
                let id = uuid_from_bson(result.inserted_id)?;
                query_review(&collection, id).await
            }
            Err(_) => Err(Error::new("Adding review failed in MongoDB.")),
        }
    }

    /// Updates a specific review referenced with an id.
    async fn update_review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UpdateReviewInput")] input: UpdateReviewInput,
    ) -> Result<Review> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        let product_variant_collection: Collection<ProductVariant> =
            db_client.collection::<ProductVariant>("product_variants");
        let current_timestamp = DateTime::now();
        update_body(
            &collection,
            &product_variant_collection,
            &input,
            &current_timestamp,
        )
        .await?;
        update_rating(&collection, &input, &current_timestamp).await?;
        let review = query_review(&collection, input.id).await?;
        Ok(review)
    }

    /// Deletes review of id.
    async fn delete_review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of review to delete.")] id: Uuid,
    ) -> Result<bool> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        if let Err(_) = collection.delete_one(doc! {"_id": id }, None).await {
            let message = format!("Deleting review of id: `{}` failed in MongoDB.", id);
            return Err(Error::new(message));
        }
        Ok(true)
    }
}

/// Extracts UUID from Bson.
///
/// Adding a review returns a UUID in a Bson document. This function helps to extract the UUID.
fn uuid_from_bson(bson: Bson) -> Result<Uuid> {
    match bson {
        Bson::Binary(id) => Ok(id.to_uuid()?),
        _ => {
            let message = format!(
                "Returned id: `{}` needs to be a Binary in order to be parsed as a Uuid",
                bson
            );
            Err(Error::new(message))
        }
    }
}

/// Updates product variant ids of a review.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - `UpdateReviewInput`.
async fn update_product_variant_ids(
    collection: &Collection<Review>,
    product_variant_collection: &Collection<ProductVariant>,
    input: &UpdateReviewInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_product_variant_ids) = &input.product_variant_ids {
        validate_product_variant_ids(&product_variant_collection, definitely_product_variant_ids)
            .await?;
        let normalized_product_variants: Vec<ProductVariant> = definitely_product_variant_ids
            .iter()
            .map(|id| ProductVariant { _id: id.clone() })
            .collect();
        if let Err(_) = collection.update_one(doc!{"_id": input.id }, doc!{"$set": {"internal_product_variants": normalized_product_variants, "last_updated_at": current_timestamp}}, None).await {
            let message = format!("Updating product_variant_ids of review of id: `{}` failed in MongoDB.", input.id);
            return Err(Error::new(message))
        }
    }
    Ok(())
}

/// Updates name of a review.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - `UpdateReviewInput`.
async fn update_name(
    collection: &Collection<Review>,
    input: &UpdateReviewInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_name) = &input.name {
        let result = collection
            .update_one(
                doc! {"_id": input.id },
                doc! {"$set": {"name": definitely_name, "last_updated_at": current_timestamp}},
                None,
            )
            .await;
        if let Err(_) = result {
            let message = format!(
                "Updating name of review of id: `{}` failed in MongoDB.",
                input.id
            );
            return Err(Error::new(message));
        }
    }
    Ok(())
}

/// Checks if product variants and user in AddReviewInput are in the system (MongoDB database populated with events).
async fn validate_input(db_client: &Database, input: &AddReviewInput) -> Result<()> {
    let product_variant_collection: Collection<ProductVariant> =
        db_client.collection::<ProductVariant>("product_variants");
    let user_collection: Collection<User> = db_client.collection::<User>("users");
    validate_product_variant_ids(&product_variant_collection, &input.product_variant_ids).await?;
    validate_user(&user_collection, input.user_id).await?;
    Ok(())
}

/// Checks if product variants are in the system (MongoDB database populated with events).
///
/// Used before adding or modifying product variants / reviews.
async fn validate_product_variant_ids(
    collection: &Collection<ProductVariant>,
    product_variant_ids: &HashSet<Uuid>,
) -> Result<()> {
    let product_variant_ids_vec: Vec<Uuid> = product_variant_ids.clone().into_iter().collect();
    match collection
        .find(doc! {"_id": { "$in": &product_variant_ids_vec } }, None)
        .await
    {
        Ok(cursor) => {
            let product_variants: Vec<ProductVariant> = cursor.try_collect().await?;
            product_variant_ids_vec.iter().fold(Ok(()), |_, p| {
                match product_variants.contains(&ProductVariant { _id: *p }) {
                    true => Ok(()),
                    false => {
                        let message = format!(
                            "Product variant with the UUID: `{}` is not present in the system.",
                            p
                        );
                        Err(Error::new(message))
                    }
                }
            })
        }
        Err(_) => Err(Error::new(
            "Product variants with the specified UUIDs are not present in the system.",
        )),
    }
}

/// Checks if user is in the system (MongoDB database populated with events).
///
/// Used before adding reviews.
async fn validate_user(collection: &Collection<User>, id: Uuid) -> Result<()> {
    query_user(&collection, id).await.map(|_| ())
}
