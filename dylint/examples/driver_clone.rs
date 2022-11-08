mod backend_service {
    mod grpc {
        mod driver {
            #[derive(Clone)]
            pub struct Driver {
                pub inner: u8,
            }
        }
        pub use driver::Driver;
    }
    pub mod driver {
        pub use super::grpc::Driver;
    }
}

use backend_service::driver::Driver as GrpcDriver;

fn main() {
    let driver = GrpcDriver { inner: 0 };
    let _clone_ko = driver.clone();

    let ref_driver = &driver;
    let _ref_clone_ko = ref_driver.clone();

    let _clone_ok = GrpcDriver::clone(&driver);
}
