use std::cmp::Ordering;

use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use bson::{doc, Bson, Document, Uuid};
use mongodb::{options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

use crate::{
    base_connection::{BaseConnection, FindResultWrapper},
    order_datatypes::ReviewOrderInput,
    review::Review,
    review_connection::ReviewConnection,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, SimpleObject)]
#[graphql(complex)]
pub struct ProductVariant {
    /// UUID of the product variant.
    pub _id: Uuid,
}

#[ComplexObject]
impl ProductVariant {
    /// Retrieves reviews of product variant.
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
    /// 
    /// Filters reviews with `is_visible == false` to exclude them from the average rating.
    async fn average_rating<'a>(&self, ctx: &Context<'a>) -> Result<f32> {
        let review_connection = self.reviews(&ctx, None, None, None).await?;
        let reviews = review_connection.nodes;
        let (accumulated_reviews, total_count) = reviews.iter().filter(|r| r.is_visible).fold(
            (0, 0),
            |(prev_accumulated_reviews, prev_total_count), r| {
                (
                    prev_accumulated_reviews + r.rating as i32,
                    prev_total_count + 1,
                )
            },
        );
        if total_count == 0 {
            let message = format!("Average rating can not be calculated, no review exists for product variant of UUID: `{}`", self._id);
            Err(Error::new(message))
        } else {
            let average_rating = accumulated_reviews as f32 / total_count as f32;
            Ok(average_rating)
        }
    }
}

impl PartialOrd for ProductVariant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self._id.partial_cmp(&other._id)
    }
}

impl From<ProductVariant> for Bson {
    fn from(value: ProductVariant) -> Self {
        Bson::Document(doc!("_id": value._id))
    }
}
