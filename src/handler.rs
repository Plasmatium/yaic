use std::{path::{Path, PathBuf}, sync::Arc};

use futures::future::join_all;
use tokio::join;
use tracing::{warn, info};

use crate::compressor::compress;

pub struct Handler {
    in_dir: String,
    out_dir: String,
    cfg: HandlerConfig,
}

impl Handler {
    pub fn new<S: Into<String>>(in_dir: S, out_dir: S, cfg: HandlerConfig) -> Self {
        Self {
            in_dir: in_dir.into(),
            out_dir: out_dir.into(),
            cfg,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let dirs = std::fs::read_dir(&self.in_dir)?;
        let files: Vec<PathBuf> = dirs
            .map(|p| p.map(|de| de.path()))
            .collect::<Result<_, _>>()?;
        let mut futs = vec![];
        let count: i32 = files.iter().map(|pb| {
            if let Some(stem) = pb.file_stem() {
                let out_dir: &Path = self.out_dir.as_ref();
                let mut fout = PathBuf::new();
                fout.push(out_dir);
                fout = fout.join(stem);
                fout.set_extension("webp");

                let fin = pb;

                let fut = compress(fin, fout, 1.0);
                futs.push(fut);
                1
            } else {
                warn!("ignore path: {}", pb.display());
                0
            }
        }).sum();
        info!("dispatched: {count}");
        let results = join_all(futs).await;
        results.into_iter().collect::<anyhow::Result<Vec<_>>>()?;
        Ok(())
    }
}

pub struct HandlerConfig {}

impl HandlerConfig {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run() {
        let h = Handler::new("/Users/jonnywong/Pictures/compressor-test/origin", "/Users/jonnywong/Pictures/compressor-test/dist", HandlerConfig::new());
        h.run().await.unwrap();
    }
}