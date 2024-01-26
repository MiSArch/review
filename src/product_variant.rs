use async_graphql::{ComplexObject, Context, SimpleObject, Result};
use bson::{Uuid, doc, Bson};
use serde::{Deserialize, Serialize};

use crate::{order_datatypes::ReviewOrderInput, review_connection::ReviewConnection};
use std::{cmp::Ordering, hash::Hash};

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone, SimpleObject)]
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
        todo!();
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