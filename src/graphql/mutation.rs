use async_graphql::{Context, Error, Object, Result};
use bson::Bson;
use bson::Uuid;
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};

use crate::authorization::authorize_user;

use super::model::product_variant::ProductVariant;
use super::model::review::Review;
use super::model::user::User;
use super::mutation_input_structs::CreateReviewInput;
use super::mutation_input_structs::UpdateReviewInput;
use super::query::query_object;

/// Describes GraphQL review mutations.
pub struct Mutation;

#[Object]
impl Mutation {
    /// Adds a review for a user and a product variant with a content, rating and visibility.
    async fn create_review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "CreateReviewInput")] input: CreateReviewInput,
    ) -> Result<Review> {
        authorize_user(&ctx, Some(input.user_id))?;
        let db_client = ctx.data::<Database>()?;
        let product_variant_collection: Collection<ProductVariant> =
            db_client.collection::<ProductVariant>("product_variants");
        let review_collection: Collection<Review> = db_client.collection::<Review>("reviews");
        validate_input(db_client, &input).await?;
        let current_timestamp = DateTime::now();
        let product_variant =
            query_object(&product_variant_collection, input.product_variant_id).await?;
        let review = Review {
            _id: Uuid::new(),
            user: User { _id: input.user_id },
            product_variant,
            body: input.body.clone(),
            rating: input.rating,
            created_at: current_timestamp,
            last_updated_at: current_timestamp,
            is_visible: input.is_visible.unwrap_or(true),
        };
        review_is_already_written_by_user(&review_collection, &input).await?;
        match review_collection.insert_one(review, None).await {
            Ok(result) => {
                let id = uuid_from_bson(result.inserted_id)?;
                query_object(&review_collection, id).await
            }
            Err(_) => Err(Error::new("Adding review failed in MongoDB.")),
        }
    }

    /// Updates a specific review referenced with an UUID.
    async fn update_review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UpdateReviewInput")] input: UpdateReviewInput,
    ) -> Result<Review> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        let current_timestamp = DateTime::now();
        let review = query_object(&collection, input.id).await?;
        authorize_user(&ctx, Some(review.user._id))?;
        update_body(&collection, &input, &current_timestamp).await?;
        update_rating(&collection, &input, &current_timestamp).await?;
        update_visibility(&collection, &input, &current_timestamp).await?;
        let review = query_object(&collection, input.id).await?;
        Ok(review)
    }

    /// Deletes review of UUID.
    async fn delete_review<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of review to delete.")] id: Uuid,
    ) -> Result<bool> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        let review = query_object(&collection, id).await?;
        authorize_user(&ctx, Some(review.user._id))?;
        if let Err(_) = collection.delete_one(doc! {"_id": id }, None).await {
            let message = format!("Deleting review of id: `{}` failed in MongoDB.", id);
            return Err(Error::new(message));
        }
        Ok(true)
    }
}

/// Extracts UUID from BSON.
///
/// Adding a review returns a UUID in a BSON document. This function helps to extract the UUID.
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

/// Updates body of a review.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - Update review input containing modified body.
/// * `current_timestamp` - Timestamp of review body update.
async fn update_body(
    collection: &Collection<Review>,
    input: &UpdateReviewInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_body) = &input.body {
        if let Err(_) = collection
            .update_one(
                doc! {"_id": input.id },
                doc! {"$set": {"body": definitely_body, "last_updated_at": current_timestamp}},
                None,
            )
            .await
        {
            let message = format!(
                "Updating body of review of id: `{}` failed in MongoDB.",
                input.id
            );
            return Err(Error::new(message));
        }
    }
    Ok(())
}

/// Updates rating of a review.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - Update review input containing new rating.
/// * `current_timestamp` - Timestamp of review rating update.
async fn update_rating(
    collection: &Collection<Review>,
    input: &UpdateReviewInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_rating) = &input.rating {
        if let Err(_) = collection
            .update_one(
                doc! {"_id": input.id },
                doc! {"$set": {"rating": definitely_rating, "last_updated_at": current_timestamp}},
                None,
            )
            .await
        {
            let message = format!(
                "Updating rating of review of id: `{}` failed in MongoDB.",
                input.id
            );
            return Err(Error::new(message));
        }
    }
    Ok(())
}

/// Updates visibility of a review.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - Update review input containing new visibility.
/// * `current_timestamp` - Timestamp of review visibility update.
async fn update_visibility(
    collection: &Collection<Review>,
    input: &UpdateReviewInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_is_visible) = &input.is_visible {
        if let Err(_) = collection.update_one(doc!{"_id": input.id }, doc!{"$set": {"is_visible": definitely_is_visible, "last_updated_at": current_timestamp}}, None).await {
            let message = format!("Updating visibility of review of id: `{}` failed in MongoDB.", input.id);
            return Err(Error::new(message))
        }
    }
    Ok(())
}

/// Checks if product variants and user in CreateReviewInput are in the system (MongoDB database populated with events).
///
/// * `db_client` - MongoDB database client.
/// * `input` - Create review input containing information to create review.
async fn validate_input(db_client: &Database, input: &CreateReviewInput) -> Result<()> {
    let product_variant_collection: Collection<ProductVariant> =
        db_client.collection::<ProductVariant>("product_variants");
    let user_collection: Collection<User> = db_client.collection::<User>("users");
    validate_product_variant_id(&product_variant_collection, input.product_variant_id).await?;
    validate_user(&user_collection, input.user_id).await?;
    Ok(())
}

/// Checks if product variant in is in the system (MongoDB database populated with events).
///
/// Used before adding reviews.
///
/// * `collection` - MongoDB collection to validate against.
/// * `product_variant_id` - Product variant UUID to validate.
async fn validate_product_variant_id(
    collection: &Collection<ProductVariant>,
    product_variant_id: Uuid,
) -> Result<()> {
    let message = format!(
        "Product variant with the UUID: `{}` is not present in the system.",
        product_variant_id
    );
    match collection
        .find_one(doc! {"_id": product_variant_id }, None)
        .await
    {
        Ok(maybe_product_variant) => match maybe_product_variant {
            Some(_) => Ok(()),
            None => Err(Error::new(message)),
        },
        Err(_) => Err(Error::new(message)),
    }
}

/// Checks if user is in the system (MongoDB database populated with events).
///
/// Used before adding reviews.
///
/// * `collection` - MongoDB collection to validate against.
/// * `id` - User UUID to validate.
async fn validate_user(collection: &Collection<User>, id: Uuid) -> Result<()> {
    query_object(&collection, id).await.map(|_| ())
}

/// Throws an error if user has already written a review for the product variant.
///
/// * `collection` - MongoDB collection to check against.
/// * `input` - Create review input containing user UUID and product variant UUID to check.
async fn review_is_already_written_by_user(
    collection: &Collection<Review>,
    input: &CreateReviewInput,
) -> Result<()> {
    let message = format!(
        "User of UUID: `{}` has already written a review for product variant of UUID: `{}`.",
        input.user_id, input.product_variant_id
    );
    match collection
        .find_one(
            doc! {"product_variant._id": input.product_variant_id, "user._id": input.user_id },
            None,
        )
        .await
    {
        Ok(maybe_product_variant) => match maybe_product_variant {
            Some(_) => Err(Error::new(message)),
            None => Ok(()),
        },
        Err(_) => Err(Error::new(message)),
    }
}
