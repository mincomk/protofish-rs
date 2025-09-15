mod prost_generated {
    #![allow(warnings)]

    pub mod common {
        pub mod v1 {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/prost_generated/common.v1.rs"
            ));
        }
    }

    pub mod payload {
        pub mod v1 {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/prost_generated/payload.v1.rs"
            ));
        }
    }
}

mod constant;
mod core;
mod error;
mod internal;
mod macros;
mod schema;
mod utp;
