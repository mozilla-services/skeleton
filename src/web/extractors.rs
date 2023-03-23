//! Request header/body/query extractors
//!
//! Handles ensuring the header's, body, and query parameters are correct, extraction to
//! relevant types, and failing correctly with the appropriate errors if issues arise.
use actix_web::{dev::Payload, web::Data, Error, FromRequest, HttpRequest};
use futures::future::{FutureExt, LocalBoxFuture};

use crate::{error::HandlerErrorKind, server::ServerState};

#[derive(Clone, Debug)]
pub struct ExampleRequest;

impl FromRequest for ExampleRequest {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();

        async move {
            let _state = match req.app_data::<Data<ServerState>>() {
                Some(s) => s,
                None => {
                    error!("⚠️ Could not load the app state");
                    return Err(HandlerErrorKind::General("Bad state".to_owned()).into());
                }
            };
            Ok(ExampleRequest)
        }
        .boxed_local()
    }
}
