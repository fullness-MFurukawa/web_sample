use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::HttpResponse;
use actix_web::http::header;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tera::{Context, Tera};
use crate::{Result, WebAppError};

///
/// HTMLレスポンス生成、リダイレクト操作
///
pub struct UiHelper;
impl UiHelper {
    // HTMLレスポンスを生成する
    pub fn create_resp(tera: &Tera,context: &Context , path: &str) -> HttpResponse {
        let body = tera.render(path, context).unwrap();
        HttpResponse::Ok().content_type(mime::TEXT_HTML).body(body)
    }
    // リダイレクトする
    pub fn found(path: &str , cookie: Option<Cookie>) -> HttpResponse {
        if cookie.is_some(){
            HttpResponse::Found().cookie(cookie.unwrap()).insert_header((header::LOCATION , path)).finish()
        }else {
            HttpResponse::Found().insert_header((header::LOCATION , path)).finish()
        }
    }
}

///
/// Session操作
///
pub struct SessionHelper;
impl SessionHelper {
    // 指定された値をSessionに登録する
    pub fn insert<T: Serialize>(session: &Session, key: &str , value: &T) -> Result<()> {
        match session.insert(key, value) {
            Ok(()) => Ok(()) ,
            Err(error) => Err(WebAppError::InternalError(error.to_string()))
        }
    }
    // Sessionに登録された値を削除する
    pub fn remove(session: &Session , key: &str) -> () {
        match session.remove(key) {
            Some(_) => () , None => ()
        }
    }
    // セッションから指定された値を取得する
    pub fn get<T: DeserializeOwned>(session: &Session , key: &str) -> Result<Option<T>>{
        match session.get(key){
            Ok(value) => Ok(value) ,
            Err(error) => Err(WebAppError::InternalError(error.to_string()))
        }
    }
}