#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use std::io::Cursor;

use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::State;
use rocket_contrib::templates::Template;
use rocket_prometheus::PrometheusMetrics;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize)]
struct KratosRegistration {
    methods: KratosMethods,
}

#[derive(Serialize, Deserialize)]
struct KratosMethods {
    oidc: KratosOidc,
}

#[derive(Serialize, Deserialize)]
struct KratosOidc {
    config: KratosConfig,
}

#[derive(Serialize, Deserialize)]
struct KratosConfig {
    action: String,
    method: String,
    messages: Option<Vec<KratosMessage>>,
    fields: Vec<KratosField>,
}

#[derive(Serialize, Deserialize)]
struct KratosMessage {
    id: serde_json::Value, // is a number in examples, but I bet that could change
    #[serde(rename = "type")]
    type_: String,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct KratosField {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    required: Option<bool>,
    value: Option<String>,
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error(transparent)]
    RetrievalError(#[from] reqwest::Error),

    #[error(transparent)]
    DeserializationError(#[from] serde_json::Error),

    #[error("Unexpected issue likely due to misconfiguration: {0}")]
    MessagesError(String),
}

impl<'r> Responder<'r> for RequestError {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let error_string = self.to_string();

        Response::build()
            .sized_body(Cursor::new(error_string))
            .status(if let RequestError::MessagesError(_) = self {
                Status::BadRequest
            } else {
                Status::InternalServerError
            })
            .ok()
    }
}

struct RegistrationEndpoint(String);

#[get("/?<flow>")]
fn index(
    flow: String,
    registration: State<RegistrationEndpoint>,
) -> Result<Template, RequestError> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(&registration.0).query(&[("id", flow)]).send()?;
    let body = response.text()?;
    let kratos: KratosRegistration = serde_json::from_str(&body)?;

    if let Some(messages) = &kratos.methods.oidc.config.messages {
        let errors: Vec<_> = messages
            .iter()
            .filter(|message| message.type_.eq("error"))
            .map(|message| message.text.as_str()) // cannot just borrow this because then join doesn't work for some reason
            .collect();
        if !errors.is_empty() {
            let combined = errors.join(" ; ");
            return Err(RequestError::MessagesError(combined));
        }
    }
    Ok(Template::render("form", &kratos))
}

#[get("/healthz")]
fn healthz() -> String {
    "OK".to_string()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, healthz])
        .attach(Template::fairing())
}

fn main() {
    let prometheus = PrometheusMetrics::new();
    rocket().attach(prometheus.clone()).mount("/metrics", prometheus).attach(AdHoc::on_attach("Endpoint Configuration", |rocket| {
        let registration = rocket.config()
            .get_str("registration_endpoint")
            .expect("You must provide registration_endpoint in config or as ROCKET_REGISTRATION_ENDPOINT in env")
            .to_string();
        Ok(rocket.manage(RegistrationEndpoint(registration)))
    })).launch();
}
