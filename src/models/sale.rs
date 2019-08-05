use diesel::PgConnection;
use diesel::BelongingToDsl;
use chrono::NaiveDate;
use juniper::{FieldResult};
use crate::schema;
use crate::schema::sales;
use crate::schema::sale_products;
use crate::db_connection::PgPooledConnection;

#[derive(Identifiable, Queryable, Debug, Clone, PartialEq)]
#[table_name="sales"]
#[derive(juniper::GraphQLObject)]
#[graphql(description="Sale Bill")]
pub struct Sale {
    pub id: i32,
    pub user_id: i32,
    pub sale_date: NaiveDate,
    pub total: f64
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="sales"]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description="Sale Bill")]
pub struct NewSale {
    pub id: Option<i32>,
    pub sale_date: Option<NaiveDate>,
    pub user_id: Option<i32>,
    pub total: Option<f64>
}

use crate::models::sale_product::{ SaleProduct, NewSaleProduct, NewSaleProducts };

#[derive(Debug, Clone)]
#[derive(juniper::GraphQLObject)]
pub struct FullSale {
    pub sale: Sale,
    pub sale_products: Vec<SaleProduct>
}

use std::sync::Arc;

pub struct Context {
    pub user_id: i32,
    pub conn: Arc<PgPooledConnection>,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::object(
    Context = Context,
)]
impl Query {

    fn sale(context: &Context, sale_id: i32) -> FieldResult<FullSale> {
        use diesel::{ ExpressionMethods, QueryDsl, RunQueryDsl };

        let conn: &PgConnection = &context.conn;
        let sale: Sale =
            schema::sales::table
                .filter(sales::dsl::user_id.eq(context.user_id))
                .find(sale_id)
                .first::<Sale>(conn)?;
        
        let sale_products = 
            SaleProduct::belonging_to(&sale)
                .load::<SaleProduct>(conn)?;
        Ok(FullSale{ sale, sale_products })
    }
}

pub struct Mutation;

#[juniper::object(
    Context = Context,
)]
impl Mutation {

    fn createSale(context: &Context, param_new_sale: NewSale, param_new_sale_products: NewSaleProducts) 
        -> FieldResult<FullSale> {
            use diesel::RunQueryDsl;
            use diesel::Connection;

            let conn: &PgConnection = &context.conn;

            let new_sale = NewSale {
                user_id: Some(context.user_id),
                ..param_new_sale
            };

            conn.transaction(|| {
                let sale = 
                    diesel::insert_into(schema::sales::table)
                        .values(new_sale)
                        .returning(
                            (
                                sales::dsl::id,
                                sales::dsl::user_id,
                                sales::dsl::sale_date,
                                sales::dsl::total
                            )
                        )
                        .get_result::<Sale>(conn)?;

                let sale_products: Result<Vec<SaleProduct>, _> =
                    param_new_sale_products.data.into_iter().map(|param_new_sale_product| {
                        let new_sale_product = NewSaleProduct {
                            sale_id: Some(sale.id), ..param_new_sale_product
                        };
                        diesel::insert_into(schema::sale_products::table)
                            .values(new_sale_product)
                            .returning(
                                (
                                    sale_products::dsl::id,
                                    sale_products::dsl::product_id,
                                    sale_products::dsl::sale_id,
                                    sale_products::dsl::amount,
                                    sale_products::dsl::discount,
                                    sale_products::dsl::tax,
                                    sale_products::dsl::price,
                                    sale_products::dsl::total
                                )
                            )
                            .get_result::<SaleProduct>(conn)
                        }).collect();

                Ok(FullSale{ sale, sale_products: sale_products? })
            })
        }


    fn updateSale(context: &Context, param_sale: NewSale, param_sale_products: NewSaleProducts) 
        -> FieldResult<FullSale> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use diesel::Connection;
            use crate::schema::sales::dsl;

            let conn: &PgConnection = &context.conn;
            let sale_id = param_sale.id.ok_or(
                diesel::result::Error::QueryBuilderError("missing id".into())
            )?;

            conn.transaction(|| {
                let sale = 
                    diesel::update(dsl::sales
                                       .filter(dsl::user_id.eq(context.user_id))
                                       .find(sale_id))
                        .set(&param_sale)
                        .get_result::<Sale>(conn)?;
                
                let sale_products: Result<Vec<SaleProduct>, _> =
                    param_sale_products.data.into_iter().map (|sale_product| {
                        diesel::update(schema::sale_products::table)
                            .set(&sale_product)
                            .get_result::<SaleProduct>(conn)
                    }).collect();

                Ok(FullSale{ sale, sale_products: sale_products? })
            })
        }

    fn destroySale(context: &Context, sale_id: i32) 
        -> FieldResult<i32> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use crate::schema::sales::dsl;

            let conn: &PgConnection = &context.conn;
            diesel::delete(dsl::sales.filter(dsl::user_id.eq(context.user_id)).find(sale_id))
                .execute(conn)?;
            Ok(sale_id)
        }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}

pub fn create_context(logged_user_id: i32, pg_pool: PgPooledConnection) -> Context {
    Context { user_id: logged_user_id, conn: Arc::new(pg_pool)}
}