use std::sync::Arc;
use actix_web::{Responder, web};
use actix_web::web::{Data, Form};
use sea_orm::DatabaseConnection;
use tera::Tera;
use app_commons::presentation::forms::ProductSearchForm;
use app_commons::presentation::validate::AppValidator;
use app_commons::application::sea_orm::provider::AppServiceProvider;
use crate::handler::view_helper::UiHelper;
use crate::{Result, WebAppError};
use crate::jwt::WebClaims;

///
/// 商品検索 リクエストハンドラ
///
pub struct ProductSearchHandler;
impl ProductSearchHandler {
    // HTML PATH
    const VIEW_PATH: &'static str = "pages/search/search.html";
    ///
    /// キーワード入力画面要求 GET
    ///
    pub async fn enter(_claims: WebClaims , tera: web::Data<Tera>) -> Result<impl Responder> {
        Ok(UiHelper::create_resp(&tera, &tera::Context::new(), Self::VIEW_PATH))
    }
    ///
    /// 検索要求　POST
    ///
    pub async fn result(
        _claims: WebClaims ,
        form: Form<ProductSearchForm>,
        tera: Data<Tera>,
        pool: Data<Arc<DatabaseConnection>>,
        provider: Data<Arc<AppServiceProvider>>) -> Result<impl Responder> {

        // 入力値の検証
        match form.validate_value() {
            Ok(_) => (),
            Err(error) => {
                let mut context = tera::Context::new();
                context.insert("errors", &error.errors);
                return Ok(UiHelper::create_resp(&tera, &context, Self::VIEW_PATH));
            }
        };
        // 商品キーワード検索
        let mut context = tera::Context::new();
        match provider.search_service.search(&pool , &form).await{
            // 結果をContextに格納
            Ok(results) => context.insert("results" , &results),
            // エラーメッセージを取得してContextに格納する　InternalErrorはエラーレスポンス
            Err(error) =>
                context.insert("not found" , &WebAppError::error_message(error)?)
        };
        Ok(UiHelper::create_resp(&tera, &context , Self::VIEW_PATH))
    }
}