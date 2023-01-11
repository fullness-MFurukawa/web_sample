use tera::Tera;
use actix_web::{App, HttpServer, middleware, web};
use actix_session::config::PersistentSession;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::time::Duration;
use actix_web::web::resource;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use app_commons::infrastructure::pool::PoolProvider;
use app_commons::infrastructure::sea_orm::pool_impl::SeaOrmPool;
use app_commons::application::sea_orm::provider_impl::AppServiceProvider;


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
    // ランダムな署名/暗号化キーを生成
    let key = cookie::Key::generate();
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
    config.service(web::scope("/web_sample")
            //   ログイン認証
            .service(resource("/login")
                .route(web::get().to(AuthenticateHandler::enter))
                .route(web::post().to(AuthenticateHandler::authenticate)))
            // メニュー
            .route("/menu",web::get().to(MenuHandler::menu))
            // 商品キーワード検索
            .service(resource("/search/product")
                .route(web::get().to(ProductSearchHandler::enter))
                .route(web::post().to(ProductSearchHandler::result)))
            // 商品登録
            .service(resource("/register/product")
                .route(web::get().to(ProductRegisterHandler::enter))
                .route(web::post().to(ProductRegisterHandler::complete)))
                .route("/register/product/finish" , web::get().to(ProductRegisterHandler::finish))
            // 内部エラー
            .route("/error" , web::get().to(ErrorHandler::error))
        )
        // デフォルトページ
        .default_service(web::get().to(MenuHandler::menu)
    );
}
