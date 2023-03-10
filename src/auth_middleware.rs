use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use std::env;
use std::env::VarError;
use std::pin::Pin;
// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct CheckForSecret;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S> Transform<S, ServiceRequest> for CheckForSecret
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = CheckForSecretMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckForSecretMiddleware { service })
    }
}

pub struct CheckForSecretMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for CheckForSecretMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret: String = req.match_info().get("secret").unwrap().parse().unwrap();
        return match env::var("SECRET") {
            saved_secret => match saved_secret {
                Ok(unwrapped_secret) => {
                    if unwrapped_secret == secret {
                        let fut = self.service.call(req);
                        Box::pin(async move {
                            let res = fut.await?;
                            return Ok(res);
                        })
                    } else {
                        Box::pin(async move {
                            return Ok(ServiceResponse::new(
                                req.request().clone(),
                                HttpResponse::Unauthorized()
                                    .body("You do not have the correct secret"),
                            ));
                        })
                    }
                }
                Err(_) => Box::pin(async move {
                    Ok(ServiceResponse::new(
                        req.request().clone(),
                        HttpResponse::Unauthorized()
                            .body("You do not have an env variable set for the secret"),
                    ))
                }),
            },
        };
    }
}
