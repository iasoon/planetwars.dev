use std::path::PathBuf;

use clap::Parser;
use futures::io;

#[derive(Parser)]
pub struct InitCommand {
    /// workspace root directory
    path: String,
}

macro_rules! copy_asset {
    ($path:expr, $file_name:literal) => {
        ::std::fs::write(
            $path.join($file_name),
            include_bytes!(concat!("../../assets/", $file_name)),
        )?;
    };
}

impl InitCommand {
    pub async fn run(self) -> io::Result<()> {
        let path = PathBuf::from(&self.path);

        // create directories
        std::fs::create_dir_all(&path)?;
        std::fs::create_dir(path.join("maps"))?;
        std::fs::create_dir(path.join("matches"))?;
        std::fs::create_dir_all(path.join("bots/simplebot"))?;

        // create files
        copy_asset!(path, "pw_workspace.toml");
        copy_asset!(path.join("maps"), "hex.json");
        copy_asset!(path.join("bots/simplebot"), "simplebot.py");

        Ok(())
    }
}
