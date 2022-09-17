use std::future::Future;
use std::pin::Pin;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use chrono::Duration;
use serde::{Serialize, Deserialize};
use app_commons::app_commons::transfers::UserDto;
use app_commons::view_commons::jwt::{ClaimsGenerator, JWT_COOKIE_KEY, JwtDecoder};
use crate::WebAppError;

/// クレーム(認証に必要な個人情報)
/// JWTトークンのPayload
#[derive(Debug, Serialize, Deserialize)]
pub struct WebClaims {
    iat:        i64 ,      //  Token取得日時
    exp:        i64 ,      //  Tokenの有効期限
    sub:        String ,   //  リソースオーナーの識別子
    user_id:    String ,   //   ユーザーId(Uuid)
}
impl ClaimsGenerator<UserDto> for WebClaims {
    fn generate(user: &UserDto) -> Self {
        let now =  chrono::Utc::now();
        let _iat =  now.timestamp();
        // クレーム(Payload)の生成
        Self {
            iat: _iat , // 取得日時の設定
            exp: (now + Duration::minutes(5)).timestamp() , // 有効期限を5分に設定
            sub: String::from("M.Furukawa") , // オーナー識別子を設定
            user_id: user.user_id.clone() ,     // ユーザーidを設定
        }
    }
}
///
/// リクエスト受信時の前処理
///
impl FromRequest for WebClaims {
    type Error = WebAppError;
    type Future = Pin<Box<dyn Future<Output = anyhow::Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let request = req.clone();
        Box::pin(async move {
            let decoder = WebTwtDecoder::default();
            let token = decoder.decode_header(&request)?;
            match decoder.decode_jwt_token(token.as_str()) {
                Ok(token_data) =>  Ok(token_data.claims),
                Err(error) => Err(WebAppError::NotAuthenticateError(error.to_string()))
            }
        })
    }
}
///
/// Web用Jwtトークンのデコード
///
#[derive(Default)]
pub struct WebTwtDecoder;
impl JwtDecoder<WebClaims , WebAppError, HttpRequest> for WebTwtDecoder{
    fn decode_header(&self , request: &HttpRequest) -> Result<String, WebAppError> {
        match request.cookie(JWT_COOKIE_KEY) {
            Some(header) => Ok(String::from(header.name_value().1)) ,
            None => return Err(WebAppError::NotAuthenticateError(String::from("認証情報がない")))
        }
    }
}