use std::env;

pub struct Config {
    pub server_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let server_addr = env::var("SERVER_ADDR")?;
        Ok(Self { server_addr })
    }
}
