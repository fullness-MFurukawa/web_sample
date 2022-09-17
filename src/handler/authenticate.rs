use std::sync::Arc;
use actix_web::{Responder, web};
use sea_orm::DatabaseConnection;
use tera::Tera;
use app_commons::presentation::forms::LoginForm;
use app_commons::presentation::jwt::{ClaimsGenerator, JWT_COOKIE_KEY, JwtEncoder, JwtEncoderImpl};
use app_commons::presentation::validate::AppValidator;
use app_commons::application::sea_orm::provider::AppServiceProvider;
use crate::handler::view_helper::UiHelper;
use crate::{Result, WebAppError};
use crate::handler::jwt::WebClaims;

///
/// 認証 リクエストハンドラ
///
pub struct AuthenticateHandler;
impl AuthenticateHandler {
    // HTML Redirect PATH
    const VIEW_PATH: &'static str = "pages/login/login.html";
    const MENU_REDIRECT: &'static str = "/web_sample/menu";
    ///
    /// 認証
    /// ログイン画面要求
    ///
    pub async fn enter(tera: web::Data<Tera>) -> Result<impl Responder>  {
        Ok(UiHelper::create_resp(&tera , &tera::Context::new() ,Self::VIEW_PATH))
    }
    ///
    /// 認証
    /// ログイン認証
    ///
    pub async fn authenticate(
        form: web::Form<LoginForm> ,
        tera: web::Data<Tera> ,
        pool: web::Data<Arc<DatabaseConnection>> ,
        provider: web::Data<Arc<AppServiceProvider>>) -> Result<impl Responder> {
        // 入力値の検証
        match form.validate_value() {
            Ok(_) => (),
            Err(error) => {
                let mut context = tera::Context::new();
                // 検証エラーをContextに格納
                context.insert("errors", &error.errors);
                return Ok(UiHelper::create_resp(&tera, &context, Self::VIEW_PATH));
            }
        };
        // 認証
        match provider.authenticate_service.execute(&pool,&form.0).await{
            Ok(user) => {
                // JWTトークンを生成する
                let claims = WebClaims::generate(&user);
                let token = JwtEncoderImpl::encode(&claims);
                let cookie = actix_web::cookie::Cookie::build(
                    JWT_COOKIE_KEY, token).http_only(true).finish();
                Ok(UiHelper::found(Self::MENU_REDIRECT , Some(cookie)))
            },
            Err(error) => {
                let error = WebAppError::from(error)?;
                let mut context = tera::Context::new();
                context.insert("error" , &error);
                Ok(UiHelper::create_resp(&tera, &context, Self::VIEW_PATH))
            }
        }
    }
}
