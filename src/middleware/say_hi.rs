use std::future::{ready, Ready};

use actix_web::{
	dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
	Error,
};
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

		let fut = self.service.call(req);

		Box::pin(async move {
			let res = fut.await?;

			println!("Hi from response");
			Ok(res)
		})
	}
}

#[cfg(test)]
mod tests {
	use actix_service::IntoService;
	use actix_web::{http, test};
	use actix_web::test::TestRequest;
	use super::*;

	#[actix_rt::test]
	async fn add_handlers() {
		let srv = test::status_service(http::StatusCode::OK);
		let mw = SayHi{}.new_transform(srv.into_service()).await.unwrap();
		let resp = test::call_service(&mw, TestRequest::default().to_srv_request()).await;
		assert_eq!(resp.status(), http::StatusCode::OK);
	}
}
