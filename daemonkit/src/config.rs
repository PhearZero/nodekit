use std::path::{Path, PathBuf};
use std::env;
use anyhow::{Context, Result};
use std::fs;

pub struct AlgorandConfig {
    pub token: String,
    pub endpoint: String,
}

pub fn get_data_dir() -> Result<PathBuf> {
    if let Ok(data_dir) = env::var("ALGORAND_DATA") {
        return Ok(PathBuf::from(data_dir));
    }
    
    #[cfg(target_os = "linux")]
    {
        Ok(PathBuf::from("/var/lib/algorand"))
    }
    
    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME").context("HOME env var not set")?;
        Ok(PathBuf::from(home).join(".algorand"))
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        anyhow::bail!("Unsupported OS")
    }
}

pub fn load_algorand_config(data_dir: &Path) -> Result<AlgorandConfig> {
    let token_path = data_dir.join("algod.admin.token");
    let token = fs::read_to_string(&token_path)
        .with_context(|| format!("Failed to read token from {:?}", token_path))?
        .trim()
        .to_string();
        
    let net_path = data_dir.join("algod.net");
    let net_content = fs::read_to_string(&net_path)
        .with_context(|| format!("Failed to read endpoint from {:?}", net_path))?
        .trim()
        .to_string();
    
    let endpoint = if net_content.contains("://") {
        net_content
    } else {
        format!("http://{}", net_content.replace("0.0.0.0", "127.0.0.1").replace("[::]", "127.0.0.1"))
    };
    
    Ok(AlgorandConfig { token, endpoint })
}
