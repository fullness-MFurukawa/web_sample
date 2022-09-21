use std::fmt::{Display, Formatter};
use actix_web::{HttpResponse, ResponseError};
use log::{error,info};
use thiserror::Error;
use app_commons::error::AppError;
use crate::handler::view_helper::UiHelper;
use crate::Result;


///
/// アプリケーション処理　エラー型
///
#[derive(Debug , Error)]
pub enum WebAppError {
    InternalError(String) ,
    AuthorizationError(String)
}
impl WebAppError {
    pub fn error_message(error: AppError) -> Result<String> {
        match error {
            AppError::InternalError(..) => Err(Self::InternalError(error.to_string())),
            AppError::AuthenticateError(msg) |
            AppError::RegisterError(msg) |
            AppError::SearchError(msg) => Ok(msg)
        }
    }
}
impl Display for WebAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f , "{}" , self)
    }
}
// エラーのハンドリング
impl ResponseError for WebAppError {
    // エラーレスポンスの生成
    fn error_response(&self) -> HttpResponse {
        let path = match self {
            WebAppError::InternalError(msg) => {
                error!("{:?}" , msg) ;
                "/web_sample/error"
            },
            WebAppError::AuthorizationError(msg) =>{
                info!("{:?}" , msg);
                "/web_sample/login"
            }
        };
        UiHelper::found(path , None)
    }
}
