use actix_session::config::PersistentSession;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::time::Duration;
use tera::Tera;
use actix_web::{App, HttpServer, middleware, web};
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use app_commons::infrastructure::pool::PoolProvider;
use app_commons::infrastructure::sea_orm::pool_impl::SeaOrmPool;
use app_commons::application::sea_orm::provider::AppServiceProvider;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ロガーの初期化
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Teraの生成
    let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/views/**/*")).unwrap();
    // SeaOrmのDatabaseConnectionを取得
    let pool = SeaOrmPool::get().await;
    // アプリケーションサービスプロバイダの生成
    let provider = AppServiceProvider::new();
    //// Cookieセッションの準備 ランダムな署名/暗号化キーを生成
    //let secret_key = actix_web::cookie::Key::generate();
    // RedisSessionStoreを生成する
    //let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379").await.unwrap();

    // Cookieを利用するSessionMiddlewareの生成
    /*let session_middleware = || {
        // ランダムな署名/暗号化キーを生成
        let key = actix_web::cookie::Key::generate();
        // SessionMiddlewareを生成する
        SessionMiddleware::builder(
            // CookieSessionStoreと　SessionId生成用のキーを設定する
            CookieSessionStore::default() , key)
            // SessionのライフサイクルをBrowsSessionに設定する　有効期間を5分にする
            .session_lifecycle(
                BrowserSession::default().state_ttl(Duration::minutes(5))
            )
            // SessionIdの名称をrssessionIdに設定する
            .cookie_name("rsessionid".to_string())
            .build()
    };*/

    // ランダムな署名/暗号化キーを生成
    let key = actix_web::cookie::Key::generate();
    // RedisSessionStoreを生成する
    let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379").await.unwrap();

    /*  サーバーの実行 */
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())// ロギングミドルウェアの登録
            /* セッションミドルウェア(Redis)の登録*/
            .wrap(
                SessionMiddleware::builder(
                    // RedisSessionStoreとKeyを設定する
                    redis_store.clone() , key.clone())
                    .session_lifecycle(
                        // SessionのライフサイクルをPersistenceSessionに設定する 有効期間を5分にする
                        PersistentSession::default().session_ttl(Duration::minutes(5))
                    )
                    // SessionIdの名称をrssessionIdに設定する
                    .cookie_name("rsessionid".to_string()).build()
            )
            // Teraの登録
            .app_data(web::Data::new(tera.clone()))
            // DatabaseConnectionの登録
            .app_data(web::Data::new(pool.clone()))
            // アプリケーションサービスプロバイダの登録
            .app_data(web::Data::new(provider.clone()))
            // サービスの登録
            .configure(set_config)
    }).bind_openssl("127.0.0.1:8081", create_ssl_acceptor_builder())?.run().await
}

///
/// OpenSSL SslAcceptorBuilderの生成
///
fn create_ssl_acceptor_builder() -> SslAcceptorBuilder {
    // OpenSSL構造を管理し、暗号スイート、セッションオプションなどを構成する
    let mut builder: SslAcceptorBuilder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
    // 秘密鍵の設定
    builder.set_private_key_file("localhost+2-key.pem", SslFiletype::PEM).unwrap();
    // 証明書の設定
    builder.set_certificate_chain_file("localhost+2.pem").unwrap();
    builder
}

///
/// サービスの設定
///
pub fn set_config(config: &mut web::ServiceConfig){
    use web_sample::handler::view_commons::{ErrorHandler, MenuHandler};
    use web_sample::handler::product_search::ProductSearchHandler;
    use web_sample::handler::product_register::ProductRegisterHandler;
    use web_sample::handler::authenticate::AuthenticateHandler;
    config.service(
        web::scope("/web_sample")
            .route("/login" , web::get().to(AuthenticateHandler::enter))
            .route("/login" , web::post().to(AuthenticateHandler::authenticate))
            .route("/menu" , web::get().to(MenuHandler::menu))
            .route("/search/product" , web::get().to(ProductSearchHandler::enter))
            .route("/search/product" , web::post().to(ProductSearchHandler::result))
            .route("/register/product" , web::get().to(ProductRegisterHandler::enter))
            .route("/register/product" , web::post().to(ProductRegisterHandler::complete))
            .route("/register/product/finish" , web::get().to(ProductRegisterHandler::finish))
            .route("/error" , web::get().to(ErrorHandler::error))
            .default_service(web::get().to(MenuHandler::menu))
    );
}
