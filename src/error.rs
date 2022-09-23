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
    InternalError(String) ,     // 内部エラー
    AuthorizationError(String)  // 利用認可エラー
}
impl WebAppError {
    // AppErrorからメッセージを取得する
    pub fn error_message(error: AppError) -> Result<String> {
        match error {
            // 内部エラーはWebAppErrorに変換して通知する
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
    fn error_response(&self) -> HttpResponse {
        let path = match self {
            WebAppError::InternalError(msg) => {
                error!("{:?}" , msg) ;
                "/web_sample/error" // エラー画面にリダイレクトする
            },
            WebAppError::AuthorizationError(msg) =>{
                info!("{:?}" , msg);
                "/web_sample/login" // ログイン認証へリダイレクトする
            }
        };
        UiHelper::found(path , None)
    }
}
