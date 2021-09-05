#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::response::Redirect;
use rocket::State;
use rocksdb::{WriteBatch, DB};
use std::convert::TryInto;
use std::env;

mod key;
use key::*;

#[derive(Debug, Responder)]
enum Response {
    PlainText(String),
    Redirect(Redirect),
}

#[get("/")]
fn get_index() -> Response {
    match env::var("INDEX_LINK") {
        Ok(link) => Response::Redirect(Redirect::to(link)),
        _ => Response::PlainText(
            "欢迎使用 Pasty！具体的用法请参考：https://github.com/darkyzhou/pasty".to_string(),
        ),
    }
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
            Response::PlainText("此短链接不存在".to_string()),
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
        None => (Status::NotFound, "此短链接不存在".to_string()),
    }
}

#[post("/<id>?<type>&<pwd>", data = "<content>")]
fn post_by_id(
    db: &State<DB>,
    id: &str,
    r#type: &str,
    pwd: &str,
    content: &str,
) -> (Status, &'static str) {
    if id.is_empty() || r#type.is_empty() || pwd.is_empty() {
        return (Status::BadRequest, "三个参数不能为空");
    }

    if !vec!["link", "plain"].contains(&r#type) {
        return (Status::BadRequest, "不支持的短链接类型");
    }

    match db.get(type_key(id)).unwrap() {
        Some(_) => match db.get(password_key(id)).unwrap() {
            Some(val) => {
                let password_str = String::from_utf8(val).unwrap();
                if password_str == pwd.to_string() {
                    let mut batch = WriteBatch::default();
                    batch.put(content_key(id), content.as_bytes());
                    batch.put(type_key(id), r#type.as_bytes());
                    db.write(batch).unwrap();
                    (Status::Ok, "更新短链接成功")
                } else {
                    (
                        Status::BadRequest,
                        "此短链接已经存在，需要指定正确的密码来更新它",
                    )
                }
            }
            None => panic!("missing password for id {}", id),
        },
        None => {
            let mut batch = WriteBatch::default();
            batch.put(content_key(id), content.as_bytes());
            batch.put(type_key(id), r#type.as_bytes());
            batch.put(password_key(id), pwd.as_bytes());
            batch.put(stat_count_key(id), 0u64.to_be_bytes());
            db.write(batch).unwrap();
            (Status::Created, "短链接创建成功")
        }
    }
}

#[delete("/<id>?<password>")]
fn delete_by_id(db: &State<DB>, id: &str, password: &str) -> (Status, &'static str) {
    match db.get(password_key(id)).unwrap() {
        Some(val) => {
            let real_password = String::from_utf8(val).unwrap();
            if real_password != password {
                return (Status::BadRequest, "密码错误");
            }

            let mut batch = WriteBatch::default();
            batch.delete(type_key(id));
            batch.delete(content_key(id));
            batch.delete(password_key(id));
            batch.delete(stat_count_key(id));
            db.write(batch).unwrap();
            (Status::Ok, "短链接已删除")
        }
        None => (Status::NotFound, "此短链接不存在"),
    }
}

#[catch(404)]
fn not_found() -> &'static str {
    "访问的链接无效"
}

#[catch(500)]
fn internal_error() -> &'static str {
    "服务器内部出错"
}

#[rocket::main]
async fn main() {
    let db_path = env::var("DB_FILE_PATH").unwrap_or("data".to_string());
    let db = DB::open_default(db_path.clone()).unwrap();

    let rocket_result = rocket::build()
        .manage(db)
        .register("/", catchers![not_found, internal_error])
        .mount(
            "/",
            routes![
                get_index,
                get_by_id,
                get_stat_by_id,
                post_by_id,
                delete_by_id
            ],
        )
        .launch()
        .await;

    rocket_result.expect("error shutting down http server");
}
