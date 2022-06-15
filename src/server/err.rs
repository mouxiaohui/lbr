#[derive(Debug)]
pub enum ConnectError {
    VerificationFailed,
    ClientDisconnected(String),
}

impl std::error::Error for ConnectError {}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectError::VerificationFailed => write!(f, "密钥错误, 认证失败"),
            ConnectError::ClientDisconnected(client) => write!(f, "与客户端断开连接: {}", client),
        }
    }
}
