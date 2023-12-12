use std::future::{ready, Ready};
use actix_http::header::{HeaderValue};

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, http};
use futures_util::future::LocalBoxFuture;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct SayHi;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for SayHi
	where
		S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
		S::Future: 'static,
		B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = Error;
	type InitError = ();
	type Transform = SayHiMiddleware<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(SayHiMiddleware { service }))
	}
}

pub struct SayHiMiddleware<S> {
	service: S,
}

impl<S, B> Service<ServiceRequest> for SayHiMiddleware<S>
	where
		S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
		S::Future: 'static,
		B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = Error;
	type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	forward_ready!(service);

	fn call(&self, req: ServiceRequest) -> Self::Future {
		println!("Hi from start. You requested: {}", req.path());
		let mut prefix = "".to_string();
		match  req.headers().get("X-Forwarded-Prefix") {
			None => {}
			Some(header) => {
				prefix = header.to_str().unwrap().to_string();
			}
		}
		let path = (&req).path().to_string();
		let location = if prefix.len() == 0 {path} else {prefix + path.as_str()};

		let fut = self.service.call(req);

		Box::pin(async move {
			let mut res = fut.await?;
			res.headers_mut()
				.insert(http::header::LOCATION, HeaderValue::from_str(location.as_str()).unwrap());

			println!("Hi from response, {}", location);
			Ok(res)
		})
	}
}

#[cfg(test)]
mod tests {
	use actix_service::{IntoService, IntoServiceFactory, ServiceFactory};
	use actix_web::{App, http, HttpResponse, test, web};
	use actix_web::dev::AppConfig;
	use actix_web::test::TestRequest;
	use super::*;

	#[actix_rt::test]
	async fn add_handlers() {
		let srv = test::status_service(http::StatusCode::OK);
		let mw = SayHi{}.new_transform(srv.into_service()).await.unwrap();
		let req = TestRequest::with_uri("/products");
		let res = test::call_service(&mw, req.to_srv_request()).await;
		assert_eq!(res.status(), http::StatusCode::OK);
		assert_eq!("/products", res.response().headers().get(http::header::LOCATION).unwrap());
	}

	#[actix_rt::test]
	async fn call_service() {
		let app = App::new()
			.wrap(SayHi)
			.service(web::resource("/products").to(HttpResponse::Ok));
		let app_init = app.into_factory();
		let srv = app_init.new_service(AppConfig::default()).await;
		let srv = srv.unwrap();

		let req = TestRequest::with_uri("/products")
			.insert_header(("X-Forwarded-Prefix", "/api")).to_request();
		let res = test::call_service(&srv, req).await;

		assert_eq!(res.status(), http::StatusCode::OK);
		assert_eq!("/api/products", res.response().headers().get(http::header::LOCATION).unwrap());
	}
}
