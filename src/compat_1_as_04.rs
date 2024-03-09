use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

#[pin_project::pin_project]
pub(crate) struct Compat<T>
where
    T: http_body_1::Body,
{
    #[pin]
    inner: T,
    buffer: Buffer<T::Data>,
}

impl<T> Compat<T>
where
    T: http_body_1::Body,
{
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner,
            buffer: Buffer {
                data: VecDeque::new(),
                trailers: VecDeque::new(),
            },
        }
    }
}

impl<T> http_body_04::Body for Compat<T>
where
    T: http_body_1::Body,
{
    type Data = T::Data;
    type Error = T::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut this = self.project();
        loop {
            if let Some(data) = this.buffer.data.pop_front() {
                break Poll::Ready(Some(Ok(data)));
            }
            if let Some(frame) = ready!(this.inner.as_mut().poll_frame(cx)?) {
                this.buffer.push_back(frame);
            } else {
                break Poll::Ready(None);
            }
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http_02::HeaderMap>, Self::Error>> {
        let mut this = self.project();
        loop {
            if let Some(trailers) = this.buffer.trailers.pop_front() {
                break Poll::Ready(Ok(Some(trailers)));
            }
            if let Some(frame) = ready!(this.inner.as_mut().poll_frame(cx)?) {
                this.buffer.push_back(frame);
            } else {
                break Poll::Ready(Ok(None));
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body_04::SizeHint {
        let mut size_hint = http_body_04::SizeHint::new();
        size_hint.set_lower(self.inner.size_hint().lower());
        if let Some(upper) = self.inner.size_hint().upper() {
            size_hint.set_upper(upper);
        }
        size_hint
    }
}

struct Buffer<T> {
    data: VecDeque<T>,
    trailers: VecDeque<http_02::HeaderMap>,
}

impl<T> Buffer<T> {
    fn push_back(&mut self, frame: http_body_1::Frame<T>) {
        if let Some(trailers) = frame.trailers_ref() {
            let trailers = trailers
                .iter()
                .filter_map(|(name, value)| {
                    Some((
                        http_02::HeaderName::from_bytes(name.as_str().as_bytes()).ok()?,
                        http_02::HeaderValue::from_bytes(value.as_bytes()).ok()?,
                    ))
                })
                .collect();
            self.trailers.push_back(trailers);
        }
        if let Ok(data) = frame.into_data() {
            self.data.push_back(data);
        }
    }
}
