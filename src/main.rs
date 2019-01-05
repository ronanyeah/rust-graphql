#![deny(warnings)]

extern crate juniper;
extern crate juniper_warp;
extern crate warp;

use juniper::tests::model::Database;
use juniper::{EmptyMutation, RootNode};
use warp::{http::Response, log, Filter};

type Schema = RootNode<'static, Database, EmptyMutation<Database>>;

fn schema() -> Schema {
    Schema::new(Database::new(), EmptyMutation::<Database>::new())
}

fn main() {
    ::std::env::set_var("RUST_LOG", "warp_server");

    let log = log("warp_server");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });

    let state = warp::any().map(move || Database::new());
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    let routes = warp::get2()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql"))
        .or(homepage)
        .or(warp::path("graphql").and(graphql_filter))
        .with(log);

    warp::serve(routes).run(([127, 0, 0, 1], 3030));
}
