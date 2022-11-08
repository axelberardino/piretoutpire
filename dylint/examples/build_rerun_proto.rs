use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR environment variable"));

    let proto_dir = Path::new("../../../platform/libraries/proto/proto");
    let protodefs = proto_dir.join("github.com/znly/protodefs");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("service_descriptor.bin"))
        .compile(
            &[protodefs.join("core/carewatchers/services/backend/services.proto")],
            &[protodefs, proto_dir.to_path_buf()],
        )
        .expect("unable to compile service");
}
