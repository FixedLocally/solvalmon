use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub rpc_port: u16,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}