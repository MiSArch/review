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
            product_variant: ProductVariant {
                _id: input.product_variant_id,
            },
            body: input.body.clone(),
            rating: input.rating,
            created_at: current_timestamp,
            last_updated_at: current_timestamp,
            is_visible: input.is_visible.unwrap_or(true),
        };
        review_is_already_written_by_user(&collection, &input).await?;
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
        let current_timestamp = DateTime::now();
        update_body(&collection, &input, &current_timestamp).await?;
        update_rating(&collection, &input, &current_timestamp).await?;
        update_visibility(&collection, &input, &current_timestamp).await?;
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

/// Updates body of a review.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - `UpdateReviewInput`.
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
/// * `input` - `UpdateReviewInput`.
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
/// * `input` - `UpdateReviewInput`.
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

/// Checks if product variants and user in AddReviewInput are in the system (MongoDB database populated with events).
async fn validate_input(db_client: &Database, input: &AddReviewInput) -> Result<()> {
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
async fn validate_user(collection: &Collection<User>, id: Uuid) -> Result<()> {
    query_user(&collection, id).await.map(|_| ())
}

/// Throws an error if user has already written a review for the product variant.
async fn review_is_already_written_by_user(
    collection: &Collection<Review>,
    input: &AddReviewInput,
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
