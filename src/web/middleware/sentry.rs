use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    task::Context,
};

use actix_http::Extensions;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::{future::LocalBoxFuture, FutureExt};
use futures_util::future::{ok, Ready};
use sentry::protocol::Event;
use std::task::Poll;

use crate::{error::HandlerError, tags::Tags};

#[derive(Default)]
pub struct SentryWrapper;

impl<S, B> Transform<S, ServiceRequest> for SentryWrapper
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SentryWrapperMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SentryWrapperMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

#[derive(Debug)]
pub struct SentryWrapperMiddleware<S> {
    service: Rc<RefCell<S>>,
}

pub fn queue_report(mut ext: RefMut<'_, Extensions>, err: &Error) {
    let herr: Option<&HandlerError> = err.as_error();
    if let Some(herr) = herr {
        /*
        // example: Skip if the error shouldn't be reported
        if !herr.is_reportable() {
            trace!("Sentry Not reporting error: {:?}", err);
            return;
        }
        */
        let event = sentry::event_from_error(herr);
        if let Some(events) = ext.get_mut::<Vec<Event<'static>>>() {
            events.push(event);
        } else {
            let events: Vec<Event<'static>> = vec![event];
            ext.insert(events);
        }
    }
}

pub fn report(tags: &Tags, mut event: Event<'static>) {
    let tags = tags.clone();
    event.tags = tags.clone().tag_tree();
    event.extra = tags.extra_tree();
    trace!("Sentry: Sending error: {:?}", &event);
    sentry::capture_event(event);
}

impl<S, B> Service<ServiceRequest> for SentryWrapperMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, sreq: ServiceRequest) -> Self::Future {
        let mut tags = Tags::from_request_head(sreq.head());
        if let Some(rtags) = sreq.request().extensions().get::<Tags>() {
            trace!("Sentry: found tags in request: {:?}", &rtags.tags);
            for (k, v) in rtags.tags.clone() {
                tags.tags.insert(k, v);
            }
        };

        sreq.extensions_mut().insert(tags.clone());

        let fut = self.service.call(sreq);

        async move {
            let resp: Self::Response = match fut.await {
                Ok(resp) => {
                    if let Some(events) = resp
                        .request()
                        .extensions_mut()
                        .remove::<Vec<Event<'static>>>()
                    {
                        for event in events {
                            trace!("Sentry: found error stored in request: {:?}", &event);
                            report(&tags, event);
                        }
                    };
                    resp
                }
                Err(err) => {
                    if let Some(herr) = err.as_error::<HandlerError>() {
                        /*
                        // Call any special processing for a given error (e.g. record metrics)
                        if let Some(state) = sresp.request().app_data::<Data<ServerState>>() {
                            herr.on_response(state.as_ref());
                        };
                        */
                        /*
                        // skip reporting error if need be
                        if !herr.is_reportable() {
                            trace!("Sentry: Not reporting error: {:?}", herr);
                            return future::ok(sresp);
                        }
                        */
                        report(&tags, sentry::event_from_error(herr));
                    };
                    return Err(err);
                }
            };

            Ok(resp)
        }
        .boxed_local()
    }
}
