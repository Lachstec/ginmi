pub mod client;
mod error;

pub(crate) mod gen {
    pub mod gnmi {
        tonic::include_proto!("gnmi");
    }

    pub mod gnmi_ext {
        tonic::include_proto!("gnmi_ext");
    }

    pub mod target {
        tonic::include_proto!("target");
    }

    pub mod google {
        pub mod protobuf {
            tonic::include_proto!("google.protobuf");
        }
    }
}