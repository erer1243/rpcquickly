#[macro_export]
macro_rules! signature {
    ($domain:expr => $range:expr) => {
        fn signature(&self) -> $crate::Signature {
            $crate::Signature {
                domain: $domain,
                range: $range,
            }
        }
    };
    (infer) => {
        fn signature(&self) -> $crate::Signature {
            $crate::Signature {
                domain: <Self::Domain as $crate::InferType>::infer_type(),
                range: <Self::Range as $crate::InferType>::infer_type(),
            }
        }
    };
}

#[macro_export]
macro_rules! call {
    (async fn call(& $self:ident, $domain_ident:ident : $domain_ty:ty) -> $range_ty:ty { $($body:tt)* }) => {
        type Domain = $domain_ty;
        type Range = $range_ty;

        fn call<'call>(&'call $self, $domain_ident: $domain_ty) -> $crate::BoxFuture<'call, Self::Range> {
            let body = async move {
                $($body)*
            };
            Box::pin(body)
        }
    };
}

#[macro_export]
macro_rules! name {
    ($name:expr) => {
        fn name(&self) -> &str {
            $name
        }
    };
}
