use juniper::FieldResult;
use crate::models::Context;
use crate::models::sale::{Sale, NewSale, FullSale};
use crate::models::sale_product::NewSaleProducts;
use crate::models::price::NewPriceProductsToUpdate;
use crate::models::product::{FullProduct, Product, NewProduct};
use crate::models::sale_state::Event;

pub struct Mutation;

#[juniper::object(
    Context = Context,
)]
impl Mutation {
    fn createSale(
        context: &Context,
        param_new_sale: NewSale,
        param_new_sale_products: NewSaleProducts,
    ) -> FieldResult<FullSale> {
        Sale::create_sale(context, param_new_sale, param_new_sale_products)
    }

    fn approveSale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        Sale::set_state(context, sale_id, Event::Approve)
    }

    fn cancelSale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform credit note or debit note
        Sale::set_state(context, sale_id, Event::Cancel)
    }

    fn paySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform collection
        Sale::set_state(context, sale_id, Event::Pay)
    }

    fn partiallyPaySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform collection
        Sale::set_state(context, sale_id, Event::PartiallyPay)
    }

    fn updateSale(
        context: &Context,
        param_sale: NewSale,
        param_sale_products: NewSaleProducts,
    ) -> FieldResult<FullSale> {
        Sale::update_sale(context, param_sale, param_sale_products)
    }

    fn destroySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        Sale::destroy_sale(context, sale_id)
    }

    fn createProduct(
        context: &Context,
        param_new_product: NewProduct,
        param_new_price_products: NewPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        Product::create_product(context, param_new_product, param_new_price_products)
    }

    fn updateProduct(
        context: &Context,
        param_product: NewProduct,
        param_price_products: NewPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        Product::update_product(context, param_product, param_price_products)
    }

    fn destroyProduct(context: &Context, product_id: i32) -> FieldResult<bool> {
        Product::destroy_product(context, product_id)
    }
}
