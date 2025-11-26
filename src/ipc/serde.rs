pub use bincode::config;
pub const BINCODE_CONFIG: config::Configuration = config::standard();

pub mod __macro_helpers {
    pub use super::BINCODE_CONFIG;
    pub use bincode::error::{DecodeError, EncodeError};
    pub use bincode::{decode_from_slice, encode_to_vec};
    pub use bytes::Bytes;
}

#[macro_export]
macro_rules! impl_serialize {
    ($type:ty) => {
        use $crate::ipc::serde::__macro_helpers::*;

        impl $type {
            pub fn serialize(&self) -> Result<Bytes, EncodeError> {
                let vec = encode_to_vec(self, BINCODE_CONFIG)?;
                let bytes = Bytes::from(vec);
                return Ok(bytes);
            }
        }
    };
}

#[macro_export]
macro_rules! impl_transform {
    ($impl_type:ty => $return_type:ty) => {
        impl $impl_type {
            pub fn deserialize_to(data: &[u8]) -> Result<($return_type, usize), DecodeError> {
                decode_from_slice(data, BINCODE_CONFIG)
            }
        }

        impl $impl_type {
            pub fn deserialize(data: &[u8]) -> Result<(Self, usize), DecodeError> {
                decode_from_slice(data, BINCODE_CONFIG)
            }
        }
    };

    ($type:ty) => {
        impl_transform!($type => $type);
    };
}

#[macro_export]
macro_rules! impl_command {
    ($impl_type:ty => $return_type:ty) => {
        $crate::impl_serialize!($impl_type);
        $crate::impl_transform!($impl_type => $return_type);
    };

    ($type:ty) => {
        $crate::impl_serialize!($type);
    };
}
