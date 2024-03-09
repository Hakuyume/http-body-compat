mod compat_1_as_04;

use bytes::Buf;

pub trait Body1CompatExt {
    type Data: Buf;
    type Error;
    fn compat_04(self) -> impl http_body_04::Body<Data = Self::Data, Error = Self::Error>;
}

impl<T> Body1CompatExt for T
where
    T: http_body_1::Body,
{
    type Data = <Self as http_body_1::Body>::Data;
    type Error = <Self as http_body_1::Body>::Error;

    fn compat_04(self) -> impl http_body_04::Body<Data = Self::Data, Error = Self::Error> {
        compat_1_as_04::Compat::new(self)
    }
}
