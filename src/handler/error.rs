use std::fmt::{Display, Formatter};
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use log::error;
use thiserror::Error;
use app_commons::error::AppError;
use crate::handler::view_helper::UiHelper;
use crate::Result;

///
/// アプリケーション処理　エラー型
///
#[derive(Debug , Error)]
pub enum WebAppError{
    // 未認証エラー
    NotAuthenticateError(String) ,
    // 永続化層のエラー、ドメインルールエラー
    InternalError(anyhow::Error)
}
impl Display for WebAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f , "{}" , self)
    }
}
impl WebAppError {
    pub fn from(error: AppError) -> Result<String> {
        let msg = match error {
            AppError::SearchError(msg)      |
            AppError::RegisterError(msg)    |
            AppError::AuthenticateError(msg) => msg ,
            AppError::InternalError(error) => return Err(WebAppError::InternalError(error)),
        };
        Ok(msg)
    }
}

// エラーのハンドリング
impl ResponseError for WebAppError {
    // ステータスコードの設定
    fn status_code(&self) -> StatusCode {
        match self{
            // データベース、ドメインルールエラー
            WebAppError::InternalError(..)  => StatusCode::INTERNAL_SERVER_ERROR ,
            // 認証処理エラー
            WebAppError::NotAuthenticateError(..) => StatusCode::UNAUTHORIZED
        }
    }
    // エラーレスポンスの生成
    fn error_response(&self) -> HttpResponse {
        let path = match self {
            WebAppError::InternalError(error) => {
                error!("{:?}" , error) ;
                "/sample_web/error"
            },
            WebAppError::NotAuthenticateError(message) =>{
                error!("{}" , message);
                "/sample_web/login"
            }
        };
        UiHelper::found(path , None)
    }
}
