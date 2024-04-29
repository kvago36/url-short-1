use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Value, json};
use rocket::response::Redirect;
use rocket::http::Status;
use rocket::fs::{FileServer, relative};
use rocket::form::Form;

use std::str;
use std::collections::HashMap;

use short_uuid::{short, ShortUuid};
use url::Url;

type UrlsList = Mutex<HashMap<String, String>>;
type Urls<'r> = &'r State<UrlsList>;

#[derive(FromForm)]
struct Task<'r> {
  url: &'r str,
}

#[derive(Debug, Responder)]
pub enum ExampleResponse {
    NotFound(String),
    Redirect(Redirect),
}

#[get("/<id>")]
async fn get(id: &str, urls: Urls<'_>)-> ExampleResponse {
    println!("{}", id);

    let short = ShortUuid::from_uuid_str(&id).unwrap();

    if let Some(val) = urls.lock().await.get(&short.to_string()) {
      ExampleResponse::Redirect(Redirect::to(val.clone()))
    } else {
      ExampleResponse::NotFound("not found".to_string())
    }
}

#[post("/", data = "<task>")]
async fn new(task: Form<Task<'_>>, urls: Urls<'_>) -> (Status, Value) {
  let result= Url::parse(task.url);

  match result {
      Ok(url) => {

        let short = short!();

        urls.lock().await.insert(short.clone().to_string(), url.to_string());

        (Status::Ok, json!({
          "success": true,
          "payload": {
            "short" : short.to_string()
          },
        }))
      },
      Err(e) => (Status::BadRequest, json!({
        "success": false,
        "payload": {
          "message" : e.to_string()
        },
      }))
  }
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket.mount("/", routes![get, new])
            .mount("/", FileServer::from(relative!("static")))
            .register("/", catchers![not_found])
            .manage(UrlsList::new(HashMap::new()))

    })
}