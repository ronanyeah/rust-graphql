#![deny(warnings)]

use std::env;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate reqwest;
extern crate base64;
extern crate juniper;
extern crate juniper_warp;
extern crate warp;

use base64::{encode};

#[derive(Deserialize)]
struct Auth {
    access_token: String,
    //token_type: String,
    //scope: String,
    //expires_in: Number,
}


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

fn get_token() -> Result<String, Box<std::error::Error>> {
    let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap();

    let token = format!("{}:{}", client_id, client_secret);

    let auth_header = format!("Basic {}", encode(&token));

    let mut headers = reqwest::header::HeaderMap::new();
    let val = reqwest::header::HeaderValue::from_str(&auth_header).unwrap();

    headers.insert(reqwest::header::AUTHORIZATION, val);

    let params = [("grant_type", "client_credentials")];
    let client = reqwest::Client::new();
    let mut res = client.post("https://accounts.spotify.com/api/token")
        .headers(headers)
        .form(&params)
        .send()?;

    let json: Auth = res.json()?;

    println!("{}", json.access_token);

    Ok(json.access_token)
}
