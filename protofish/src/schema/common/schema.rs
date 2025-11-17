#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone)]
pub struct StreamCreateMeta {
    pub integrity_type: IntegrityType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityType {
    Reliable,
    Unreliable,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Unspecified,
    Timeout,
}
