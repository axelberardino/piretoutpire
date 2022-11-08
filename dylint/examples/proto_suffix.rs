use crate::proto::MyType;

fn main() {
    let value = proto::MyType {
        field: proto::inner::Enum::Start,
    };

    foo(proto::MyType {
        field: proto::inner::Enum::Start,
    });
}

fn foo(_value: proto::MyType) {}

fn gen(_value: Box<proto::MyType>) {}

impl From<proto::MyType> for u32 {
    fn from(_: proto::MyType) -> Self {
        unreachable!()
    }
}

impl From<String> for proto::MyType {
    fn from(_: String) -> Self {
        unreachable!()
    }
}

impl From<String> for Box<proto::MyType> {
    fn from(_: String) -> Self {
        unreachable!()
    }
}

use crate::proto::MyType as MyTypeProto;

mod proto {
    #[derive(Debug)]
    pub struct MyType {
        pub field: inner::Enum,
    }

    impl prost::Message for MyType {
        fn encode_raw<B>(&self, _buf: &mut B)
        where
            B: prost::bytes::BufMut,
            Self: Sized,
        {
            unreachable!()
        }

        fn merge_field<B>(
            &mut self,
            _tag: u32,
            _wire_type: prost::encoding::WireType,
            _buf: &mut B,
            _ctx: prost::encoding::DecodeContext,
        ) -> Result<(), prost::DecodeError>
        where
            B: prost::bytes::Buf,
            Self: Sized,
        {
            unreachable!()
        }

        fn encoded_len(&self) -> usize {
            unreachable!()
        }

        fn clear(&mut self) {
            unreachable!()
        }
    }

    pub mod inner {
        #[derive(Debug)]
        pub enum Enum {
            Start,
            End,
        }

        impl prost::Message for Enum {
            fn encode_raw<B>(&self, _buf: &mut B)
            where
                B: prost::bytes::BufMut,
                Self: Sized,
            {
                unreachable!()
            }

            fn merge_field<B>(
                &mut self,
                _tag: u32,
                _wire_type: prost::encoding::WireType,
                _buf: &mut B,
                _ctx: prost::encoding::DecodeContext,
            ) -> Result<(), prost::DecodeError>
            where
                B: prost::bytes::Buf,
                Self: Sized,
            {
                unreachable!()
            }

            fn encoded_len(&self) -> usize {
                unreachable!()
            }

            fn clear(&mut self) {
                unreachable!()
            }
        }
    }
}
