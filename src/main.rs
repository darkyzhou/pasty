#[macro_use]
extern crate rocket;

use nanoid::nanoid;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::State;
use rocksdb::{WriteBatch, DB};
use std::convert::TryInto;
use url::Url;

mod key;
use key::*;

mod config;
use config::*;

#[derive(Debug, Responder)]
enum Response {
    PlainText(String),
    Redirect(Redirect),
}

#[get("/")]
fn get_index(config: &State<Config>) -> Response {
    if !config.index_link.is_empty() {
        Response::Redirect(Redirect::to(config.index_link.clone()))
    } else {
        Response::PlainText(config.index_text.clone())
    }
}

#[post("/?<type>&<pwd>&<access>", data = "<content>")]
fn post_index(
    db: &State<DB>,
    config: &State<Config>,
    r#type: Option<&str>,
    pwd: &str,
    access: &str,
    content: &str,
) -> (Status, String) {
    let length: usize = config.random_id_length.try_into().unwrap();
    let id = nanoid!(length);
    post_by_id(db, config, &id[..], r#type, pwd, access, content)
}

#[get("/<id>")]
fn get_by_id(db: &State<DB>, id: &str) -> (Status, Response) {
    match db.get(type_key(id)).unwrap() {
        Some(type_val) => {
            let type_str = String::from_utf8(type_val).unwrap();
            match db.get(content_key(id)).unwrap() {
                Some(content_val) => {
                    let content_str = String::from_utf8(content_val).unwrap();
                    let stat_count_val = db.get(stat_count_key(id)).unwrap().unwrap();
                    let stat_count = u64::from_be_bytes(stat_count_val.try_into().unwrap());
                    db.put(stat_count_key(id), (stat_count + 1).to_be_bytes())
                        .unwrap();
                    match &type_str[..] {
                        "link" => (Status::Found, Response::Redirect(Redirect::to(content_str))),
                        "plain" => (Status::Ok, Response::PlainText(content_str)),
                        _ => panic!("unknown type '{}' for id '{}'", type_str, id),
                    }
                }
                None => panic!("missing content for id '{}'", id),
            }
        }
        None => (
            Status::NotFound,
            Response::PlainText("?????????????????????".to_string()),
        ),
    }
}

#[get("/<id>/stat")]
fn get_stat_by_id(db: &State<DB>, id: &str) -> (Status, String) {
    match db.get(stat_count_key(id)).unwrap() {
        Some(val) => (
            Status::Ok,
            u64::from_be_bytes(val.try_into().unwrap()).to_string(),
        ),
        None => (Status::NotFound, "?????????????????????".to_string()),
    }
}

#[post("/<id>?<type>&<pwd>&<access>", data = "<content>")]
fn post_by_id(
    db: &State<DB>,
    config: &State<Config>,
    id: &str,
    r#type: Option<&str>,
    pwd: &str,
    access: &str,
    content: &str,
) -> (Status, String) {
    if !config.access_password.is_empty() && access != &config.access_password[..] {
        return (Status::BadRequest, "??????????????????".to_string());
    }

    let content_type = match r#type {
        Some(val) => val,
        None => "link",
    };

    if id.is_empty() || content_type.is_empty() || pwd.is_empty() {
        return (Status::BadRequest, "????????????????????????".to_string());
    }

    if !vec!["link", "plain"].contains(&content_type) {
        return (Status::BadRequest, "???????????????????????????".to_string());
    }

    if content_type == "link" && Url::parse(content).is_err() {
        return (Status::BadRequest, "?????????????????????????????? URL".to_string());
    }

    match db.get(type_key(id)).unwrap() {
        Some(_) => match db.get(password_key(id)).unwrap() {
            Some(val) => {
                let password_str = String::from_utf8(val).unwrap();
                if password_str == pwd.to_string() {
                    let mut batch = WriteBatch::default();
                    batch.put(content_key(id), content.as_bytes());
                    batch.put(type_key(id), content_type.as_bytes());
                    db.write(batch).unwrap();
                    (Status::Ok, format!("????????????????????????{}", id))
                } else {
                    (
                        Status::BadRequest,
                        "??????????????????????????????????????????????????????????????????".to_string(),
                    )
                }
            }
            None => panic!("missing password for id {}", id),
        },
        None => {
            let mut batch = WriteBatch::default();
            batch.put(content_key(id), content.as_bytes());
            batch.put(type_key(id), content_type.as_bytes());
            batch.put(password_key(id), pwd.as_bytes());
            batch.put(stat_count_key(id), 0u64.to_be_bytes());
            db.write(batch).unwrap();
            (Status::Created, format!("????????????????????????{}", id))
        }
    }
}

#[delete("/<id>?<password>")]
fn delete_by_id(db: &State<DB>, id: &str, password: &str) -> (Status, &'static str) {
    match db.get(password_key(id)).unwrap() {
        Some(val) => {
            let real_password = String::from_utf8(val).unwrap();
            if real_password != password {
                return (Status::BadRequest, "????????????");
            }

            let mut batch = WriteBatch::default();
            batch.delete(type_key(id));
            batch.delete(content_key(id));
            batch.delete(password_key(id));
            batch.delete(stat_count_key(id));
            db.write(batch).unwrap();
            (Status::Ok, "??????????????????")
        }
        None => (Status::NotFound, "?????????????????????"),
    }
}

#[catch(404)]
fn not_found() -> &'static str {
    "???????????????????????????????????????????????????????????????????????????"
}

#[catch(500)]
fn internal_error() -> &'static str {
    "?????????????????????"
}

#[rocket::main]
async fn main() {
    let rocket_instance = rocket::build();
    let figment = rocket_instance.figment();
    let config: Config = figment
        .extract_inner("pasty")
        .expect("error loading configuration");
    let db = DB::open_default(config.db_path.clone()).expect("error opening database");
    let result = rocket_instance
        .manage(db)
        .manage(config)
        .register("/", catchers![not_found, internal_error])
        .mount(
            "/",
            routes![
                get_index,
                post_index,
                get_by_id,
                get_stat_by_id,
                post_by_id,
                delete_by_id
            ],
        )
        .launch()
        .await;

    result.expect("error shutting down http server");
}
