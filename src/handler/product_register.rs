use std::sync::Arc;
use actix_session::Session;
use actix_web::{Responder, web};
use sea_orm::DatabaseConnection;
use app_commons::application::transfers::{ProductDto , CategoryDto};
use app_commons::presentation::forms::ProductRegisterForm;
use app_commons::presentation::validate::AppValidator;
use app_commons::application::sea_orm::provider::AppServiceProvider;
use crate::{Result, WebAppError};
use crate::jwt::WebClaims;
use crate::handler::view_helper::{SessionHelper, UiHelper};

///
/// 商品登録 リクエストハンドラ
///
pub struct ProductRegisterHandler;
impl ProductRegisterHandler {
    // HTML Redirect PATH
    const ENTER_PATH: &'static  str = "pages/register/enter.html";
    const FINISH_PATH: &'static str = "pages/register/finish.html";
    const ENTER_REDIRECT: &'static  str = "/web_sample/register/product";
    const FINISH_REDIRECT: &'static str = "/web_sample/register/product/finish";
    ///
    /// 商品登録　
    /// 商品入力画面要求への応答
    ///
    pub async fn enter(
        _claims: WebClaims ,
        session: Session ,
        tera: web::Data<tera::Tera> ,
        pool: web::Data<Arc<DatabaseConnection>> ,
        provider: web::Data<Arc<AppServiceProvider>>) -> Result<impl Responder> {
        // セッションから商品カテゴリを取得する
        let session_categories = SessionHelper::get::<Vec<CategoryDto>>(&session,"categories")?;
        let categories = match session_categories {
            Some(categories) => categories ,
            None => {
                // 永続化層から商品カテゴリを取得する
                let categories = match provider.register_service.categories(&pool).await {
                    Ok(categories) => categories ,
                    Err(error) => return Err(WebAppError::InternalError(error.to_string()))
                };
                // セッションに商品カテゴリを登録
                SessionHelper::insert::<Vec<CategoryDto>>(&session , "categories" , categories.clone())?;
                categories
            }
        };
        // TeraのContextに商品カテゴリを登録する
        let mut context = tera::Context::new();
        context.insert("categories" , &categories);
        Ok(UiHelper::create_resp(&tera , &context ,Self::ENTER_PATH))
    }

    ///
    /// 商品登録　
    /// 入力値検証と登録処理
    ///
    pub async fn complete(
        _claims: WebClaims ,
        session: Session ,
        form: web::Form<ProductRegisterForm> ,
        tera: web::Data<tera::Tera>  ,
        pool: web::Data<Arc<DatabaseConnection>> ,
        provider: web::Data<Arc<AppServiceProvider>>) -> Result<impl Responder> {
        // セッションからカテゴリを取得
        let categories = match SessionHelper::get::<Vec<CategoryDto>>(&session,"categories")?{
            Some(categories) => categories ,
            None => //　入力画面にリダイレクトする
                return Ok(UiHelper::found(Self::ENTER_REDIRECT, None))
        };
        // 入力値の検証
        match form.validate_value() {
            Ok(_) => (),
            Err(error) => {
                let mut context = tera::Context::new();
                // 検証エラー、Form、カテゴリをContextに格納
                context.insert("form" , &form);
                context.insert("categories" , &categories);
                context.insert("errors", &error.errors);
                //　入力画面に遷移する
                return Ok(UiHelper::create_resp(&tera, &context, Self::ENTER_PATH));
            }
        };
        // 入力された商品を永続化する
        match provider.register_service.execute(&pool , &form).await{
            Ok(new_product) => {
                // 登録結果をSessionに格納する
                SessionHelper::insert::<ProductDto>(&session , "new_product" , new_product)?;
                // 登録結果へリダイレクト
                Ok(UiHelper::found(Self::FINISH_REDIRECT , None))
            },
            Err(error) => {
                //　登録済みの場合、入力画面に戻る
                let mut context = tera::Context::new();
                context.insert("categories" , &categories);
                context.insert("exists" , &WebAppError::error_message(error)?);
                context.insert("form" , &form);
                Ok(UiHelper::create_resp(&tera, &context , Self::ENTER_PATH))
            }
        }
    }
    ///
    /// 商品登録　登録結果の出力
    ///
    pub async fn finish(
        session: Session ,
        tera: web::Data<tera::Tera>) -> Result<impl Responder> {
        //  セッションから登録された商品情報を取得する
        match SessionHelper::get::<ProductDto>(&session, "new_product")?{
            Some(new_product) => {
                // 商品情報をセッションから削除する
                SessionHelper::remove(&session , "new_product");
                // TeraのContextに登録する
                let mut context = tera::Context::new();
                context.insert("new_product" , &new_product);
                // 完了画面を返す
                Ok(UiHelper::create_resp(&tera , &context,Self::FINISH_PATH))
            },
            None =>
                // 入力画面にリダイレクトする
                Ok(UiHelper::found(Self::ENTER_REDIRECT, None))
        }
    }
}