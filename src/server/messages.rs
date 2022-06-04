use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BindsInformation {
    pub secret_key: String,
    pub binds: Vec<Bind>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bind {
    pub local_port: u16,
    pub remote_port: u16,
}

impl std::fmt::Display for BindsInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BindsInformation {{",)?;
        writeln!(f, "  SecretKey: {}", self.secret_key)?;
        writeln!(f, "  Bind: {{")?;
        for bind in &self.binds {
            writeln!(f, "    {{")?;
            writeln!(f, "      LocalPort: {}", bind.local_port)?;
            writeln!(f, "      RemotePort: {}", bind.remote_port)?;
            writeln!(f, "    }},")?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}
