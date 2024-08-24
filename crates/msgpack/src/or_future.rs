use std::{future::Future, pin::Pin};

pub enum OrFuture<F0, F1>
where
    F0: Future,
    F1: Future<Output = F0::Output>,
{
    F0(F0),
    F1(F1),
}

impl<F0, F1> Future for OrFuture<F0, F1>
where
    F0: Future,
    F1: Future<Output = F0::Output>,
{
    type Output = F0::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            match self.get_unchecked_mut() {
                OrFuture::F0(f) => Pin::new_unchecked(f).poll(cx),
                OrFuture::F1(f) => Pin::new_unchecked(f).poll(cx),
            }
        }
    }
}
