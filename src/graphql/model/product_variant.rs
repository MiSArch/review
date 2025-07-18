use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use bson::{doc, Bson, Document, Uuid};
use mongodb::{options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

use crate::event::http_event_service::ProductVariantEventData;

use super::{
    connection::{
        base_connection::{BaseConnection, FindResultWrapper},
        review_connection::ReviewConnection,
    },
    order_datatypes::ReviewOrderInput,
    review::Review,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, SimpleObject)]
#[graphql(complex)]
pub struct ProductVariant {
    /// Product variant UUID.
    pub _id: Uuid,
    /// Associated product UUID.
    #[graphql(skip)]
    pub product_id: Uuid,
}

#[ComplexObject]
impl ProductVariant {
    /// Retrieves reviews of product variant.
    // TODO reviews should be optional
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
            .limit(first.map(|definitely_first| i64::from(definitely_first)))
            .sort(sorting_doc)
            .build();
        let document_collection = collection.clone_with_type::<Document>();
        let filter = doc! {"product_variant._id": self._id};
        let maybe_find_results: Result<FindResult<Review>, CursorError> =
            PaginatedCursor::new(Some(find_options.clone()), None, None)
                .find(&document_collection, Some(&filter))
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

    /// Retrieves average rating of product variant.
    async fn average_rating<'a>(&self, ctx: &Context<'a>) -> Option<f32> {
        match self.reviews(ctx, None, None, None).await {
            Ok(review_connection) => calculate_average_rating(review_connection).await,
            Err(_) => None,
        }
    }
}

impl From<ProductVariant> for Bson {
    fn from(value: ProductVariant) -> Self {
        Bson::Document(doc!("_id": value._id, "product_id": value.product_id))
    }
}

impl From<ProductVariantEventData> for ProductVariant {
    fn from(value: ProductVariantEventData) -> Self {
        Self {
            _id: value.id,
            product_id: value.product_id,
        }
    }
}

/// Shared function to calculate average rating of a review connection.
///
/// Filters reviews with `is_visible == false` to exclude them from the average rating.
///
/// `review_connection` - Connection of reviews to calculate average rating for.
pub async fn calculate_average_rating<'a>(review_connection: ReviewConnection) -> Option<f32> {
    let reviews = review_connection.nodes.clone();
    let (accumulated_reviews, total_count) =
        reviews.iter().filter(|review| review.is_visible).fold(
            (0, 0),
            |(prev_accumulated_reviews, prev_total_count), review| {
                (
                    prev_accumulated_reviews + review.rating as i32,
                    prev_total_count + 1,
                )
            },
        );
    if total_count == 0 {
        None
    } else {
        let average_rating = accumulated_reviews as f32 / total_count as f32;
        Some(average_rating)
    }
}
