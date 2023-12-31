mod middleware;

use actix_http::HttpService;
use actix_server::Server;
use actix_service::map_config;
use actix_web::{dev::AppConfig, get, App};
use middleware::say_hi::SayHi;

#[get("/banana")]
async fn index() -> &'static str {
    "Hello, world. From Actix Web!"
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
  let mut config = AppConfig::default();
    Server::build()
      .bind("hello-world", "127.0.0.1:8060", || {
          // construct actix-web app
          let app = App::new().wrap(SayHi).service(index);

          HttpService::build()
            // pass the app to service builder
            // map_config is used to map App's configuration to ServiceBuilder
            // h1 will configure server to only use HTTP/1.1
            .h1(map_config(app, |_| AppConfig::default()))
            .tcp()
      })?
      .run()
      .await
}