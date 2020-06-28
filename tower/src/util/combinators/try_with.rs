use futures_util::future::{ready, Either, Ready};
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;

#[derive(Debug)]
pub struct TryWith<S, F> {
    inner: S,
    f: F,
}

impl<S, F> TryWith<S, F> {
    pub fn new(inner: S, f: F) -> Self {
        TryWith { inner, f }
    }
}

impl<S, F, NewRequest, OldRequest> Service<NewRequest> for TryWith<S, F>
where
    S: Service<OldRequest>,
    F: FnOnce(NewRequest) -> Result<OldRequest, S::Error>,
    F: Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Either<S::Future, Ready<Result<S::Response, S::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: NewRequest) -> Self::Future {
        match (self.f.clone())(request) {
            Ok(ok) => Either::Left(self.inner.call(ok)),
            Err(err) => Either::Right(ready(Err(err))),
        }
    }
}

#[derive(Debug)]
pub struct TryWithLayer<F, OldRequest, NewRequest> {
    f: F,
    _p: PhantomData<fn(OldRequest, NewRequest)>,
}

impl<F, OldRequest, NewRequest> TryWithLayer<F, OldRequest, NewRequest> {
    pub fn new(f: F) -> Self {
        TryWithLayer { f, _p: PhantomData }
    }
}

impl<S, F, OldRequest, NewRequest> Layer<S> for TryWithLayer<F, OldRequest, NewRequest>
where
    S: Service<OldRequest>,
    F: FnOnce(NewRequest) -> OldRequest,
    F: Clone,
{
    type Service = TryWith<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        TryWith {
            f: self.f.clone(),
            inner,
        }
    }
}