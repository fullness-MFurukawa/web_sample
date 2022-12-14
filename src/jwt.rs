use std::future::Future;
use std::pin::Pin;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use chrono::Duration;
use serde::{Serialize, Deserialize};
use app_commons::application::transfers::UserDto;
use app_commons::presentation::jwt::{ClaimsGenerator, JWT_COOKIE_KEY, JwtDecoder, JwtEncoder};
use crate::WebAppError;

/// クレーム(認証に必要な個人情報)
/// JWTトークンのPayload
#[derive(Debug, Serialize, Deserialize)]
pub struct WebClaims {
    iat:        i64 ,      //  Token取得日時
    exp:        i64 ,      //  Tokenの有効期限
    sub:        String ,   //  リソースオーナーの識別子
    user_id:    String ,   //  ユーザーId(Uuid)
    user_name:  String,    //  ユーザー名
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
            user_name: user.user_name.clone(),  // ユーザー名
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
            // JWTデーコード機能を生成する
            let decoder = WebJwt::default();
            // リクエストヘッダーを解析する
            let token = decoder.parse_header(&request)?;
            match decoder.decode(token.as_str()) {
                // 取得したClaimsを返す
                Ok(token_data) =>  Ok(token_data.claims),
                // ヘッダーが存在しない場合は認証へリダイレクトさせる
                Err(error) => Err(WebAppError::AuthorizationError(error.to_string()))
            }
        })
    }
}
///
/// Web用Jwtトークンのデコード
///
#[derive(Default)]
pub struct WebJwt;
// トークンのエンコード デフォルト実装をそのまま利用する
impl JwtEncoder for WebJwt{}
impl JwtDecoder<WebClaims , WebAppError, HttpRequest> for WebJwt{
    fn parse_header(&self , request: &HttpRequest) -> Result<String, WebAppError> {
        // CookieからJWTトークンを取得する
        match request.cookie(JWT_COOKIE_KEY) {
            Some(cookie_value) => Ok(String::from(cookie_value.name_value().1)),
            None => return Err(WebAppError::AuthorizationError(String::from("token does not exist.")))
        }
    }
}
