use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use bson::{doc, Bson, Document, Uuid};
use mongodb::{options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

use super::{
    connection::{
        base_connection::{BaseConnection, FindResultWrapper},
        review_connection::ReviewConnection,
    },
    order_datatypes::ReviewOrderInput,
    product_variant::calculate_average_rating,
    review::Review,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Product {
    /// UUID of the product.
    pub _id: Uuid,
}

#[ComplexObject]
impl Product {
    /// Retrieves reviews of product.
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
        let filter = doc! {"product_variant.product_id": self._id};
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

    /// Retrieves average rating of product.
    async fn average_rating<'a>(&self, ctx: &Context<'a>) -> Result<f32> {
        let review_connection = self.reviews(&ctx, None, None, None).await?;
        calculate_average_rating(review_connection).await
    }
}

impl From<Product> for Bson {
    fn from(value: Product) -> Self {
        Bson::Document(doc!("_id": value._id))
    }
}

impl From<Uuid> for Product {
    fn from(value: Uuid) -> Self {
        Product { _id: value }
    }
}
