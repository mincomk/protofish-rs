use crate::{
    prost_generated::common::{self},
    schema::common::schema::{ErrorType, IntegrityType, StreamCreateMeta, Version},
};

impl From<common::v1::Version> for Version {
    fn from(value: common::v1::Version) -> Self {
        Version {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl From<Version> for common::v1::Version {
    fn from(value: Version) -> Self {
        common::v1::Version {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl From<common::v1::StreamCreateMeta> for StreamCreateMeta {
    fn from(value: common::v1::StreamCreateMeta) -> Self {
        StreamCreateMeta {
            integrity_type: common::v1::IntegrityType::try_from(value.stream_integrity)
                .unwrap()
                .into(),
        }
    }
}

impl From<common::v1::IntegrityType> for IntegrityType {
    fn from(value: common::v1::IntegrityType) -> Self {
        match value {
            // default to reliable
            common::v1::IntegrityType::Unspecified => IntegrityType::Reliable,
            common::v1::IntegrityType::Reliable => IntegrityType::Reliable,
            common::v1::IntegrityType::Unreliable => IntegrityType::Unreliable,
        }
    }
}

impl From<IntegrityType> for common::v1::IntegrityType {
    fn from(value: IntegrityType) -> Self {
        match value {
            IntegrityType::Reliable => common::v1::IntegrityType::Reliable,
            IntegrityType::Unreliable => common::v1::IntegrityType::Unreliable,
        }
    }
}

impl From<common::v1::ErrorType> for ErrorType {
    fn from(value: common::v1::ErrorType) -> Self {
        match value {
            common::v1::ErrorType::Unspecified => ErrorType::Unspecified,
            common::v1::ErrorType::Timeout => ErrorType::Timeout,
        }
    }
}

impl From<ErrorType> for common::v1::ErrorType {
    fn from(value: ErrorType) -> Self {
        match value {
            ErrorType::Unspecified => common::v1::ErrorType::Unspecified,
            ErrorType::Timeout => common::v1::ErrorType::Timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_conversion() {
        let proto_version = common::v1::Version {
            major: 1,
            minor: 2,
            patch: 3,
        };
        let schema_version: Version = proto_version.clone().into();
        assert_eq!(schema_version.major, 1);
        assert_eq!(schema_version.minor, 2);
        assert_eq!(schema_version.patch, 3);

        let converted_proto: common::v1::Version = schema_version.into();
        assert_eq!(converted_proto, proto_version);
    }

    #[test]
    fn test_stream_create_meta_conversion() {
        let proto_meta = common::v1::StreamCreateMeta {
            stream_integrity: common::v1::IntegrityType::Reliable.into(),
        };
        let schema_meta: StreamCreateMeta = proto_meta.clone().into();
        assert!(matches!(schema_meta.integrity_type, IntegrityType::Reliable));

        // The into() call for StreamCreateMeta is not implemented, so we skip that part of the test
    }

    #[test]
    fn test_integrity_type_conversion() {
        let proto_unspecified = common::v1::IntegrityType::Unspecified;
        let schema_unspecified: IntegrityType = proto_unspecified.into();
        assert!(matches!(schema_unspecified, IntegrityType::Reliable));

        let proto_reliable = common::v1::IntegrityType::Reliable;
        let schema_reliable: IntegrityType = proto_reliable.into();
        assert!(matches!(schema_reliable, IntegrityType::Reliable));

        let proto_unreliable = common::v1::IntegrityType::Unreliable;
        let schema_unreliable: IntegrityType = proto_unreliable.into();
        assert!(matches!(schema_unreliable, IntegrityType::Unreliable));

        let schema_reliable_back: common::v1::IntegrityType = IntegrityType::Reliable.into();
        assert_eq!(schema_reliable_back, common::v1::IntegrityType::Reliable);

        let schema_unreliable_back: common::v1::IntegrityType = IntegrityType::Unreliable.into();
        assert_eq!(
            schema_unreliable_back,
            common::v1::IntegrityType::Unreliable
        );
    }

    #[test]
    fn test_error_type_conversion() {
        let proto_unspecified = common::v1::ErrorType::Unspecified;
        let schema_unspecified: ErrorType = proto_unspecified.into();
        assert!(matches!(schema_unspecified, ErrorType::Unspecified));

        let proto_timeout = common::v1::ErrorType::Timeout;
        let schema_timeout: ErrorType = proto_timeout.into();
        assert!(matches!(schema_timeout, ErrorType::Timeout));

        let schema_unspecified_back: common::v1::ErrorType = ErrorType::Unspecified.into();
        assert_eq!(
            schema_unspecified_back,
            common::v1::ErrorType::Unspecified
        );

        let schema_timeout_back: common::v1::ErrorType = ErrorType::Timeout.into();
        assert_eq!(schema_timeout_back, common::v1::ErrorType::Timeout);
    }
}
