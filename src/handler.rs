use std::{path::{PathBuf}, sync::Arc, collections::HashMap};


use pathdiff::diff_paths;
use tokio::{sync::{mpsc, Mutex}, fs::create_dir_all};
use tracing::{warn, info, error, debug};

use crate::compressor::compress;

pub struct Handler {
    in_dir: String,
    out_dir: String,

    #[allow(unused)]
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

    pub fn get_output_path(&self, input: &PathBuf) -> Option<PathBuf> {
        let path = diff_paths(input, &self.in_dir);
        path.map(|diffs|{
            PathBuf::from(&self.out_dir).join(diffs)
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let pattern = [&self.in_dir, "**", "*"].join("/");
        info!("searching on: {pattern}");
        let files = glob::glob(&pattern)?.collect::<Result<Vec<_>, _>>()?;

        let recorder = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel(8);
        let rx = Arc::new(Mutex::new(rx));
        for input_path in files {
            if input_path.is_dir() {
                continue
            }
            if !is_image(&input_path) {
                warn!("not image, skipping: {input_path:?}");
                continue
            }

            let output_path = self.get_output_path(&input_path);
            if output_path == None {
                error!("cannot calculate output_path, input: {input_path:?}, base: {}", self.in_dir);
                continue
            }
            let mut output_path = output_path.unwrap();
            output_path.set_extension(".webp");
            tx.send(()).await?;

            let rx = rx.clone();
            let recorder = recorder.clone();
            tokio::spawn(async move {
                info!("processing: {:?}", input_path);
                make_dir(&output_path, recorder).await.unwrap();
                let res = compress(input_path, output_path, 1.0).await;
                if let Err(e) = res {
                    error!("failed to convert, err: {:?}", e);
                }
                let mut rx = rx.lock().await;
                rx.recv().await;
                debug!("done");
            });
        }
        Ok(())
    }
}

async fn make_dir(output_path: &PathBuf, recorder: Arc<Mutex<HashMap<PathBuf, ()>>>) -> tokio::io::Result<()> {
    let dir = output_path.parent().unwrap();
    let mut recorder = recorder.lock().await;
    if let Some(_) = recorder.get(dir) {
        return Ok(())
    }
    recorder.insert(dir.to_path_buf(), ());
    create_dir_all(dir).await
}

fn is_image(input_path: &PathBuf) -> bool {
    if let Some(ext) = input_path.extension() {
        if let Some(ext_str) = ext.to_str() {
            ["jpg", "jpeg", "png", "bmp", "webp"].contains(&ext_str)
        } else {
            false
        }
    } else {
        false
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
    use std::fs::canonicalize;

    use super::*;

    #[tokio::test]
    async fn test_run() {
        let h = Handler::new("/Users/jonnywong/Pictures/compressor-test/origin", "/Users/jonnywong/Pictures/compressor-test/dist", HandlerConfig::new());
        h.run().await.unwrap();
    }

    use glob::glob;
    #[test]
    fn test_glob() {
        let p = "/Users/jonnywong/Pictures/test-compress/origin/**/*.txt";
        // for entry in glob(p).expect("Failed to read glob pattern") {
        //     match entry {
        //         Ok(path) => println!("{:?}", path.display()),
        //         Err(e) => println!("{:?}", e),
        //     }
        // }
        let result: Result<Vec<_>, _> = glob(p).unwrap().collect();
        let paths = result.unwrap();
        let cp = canonicalize(&paths[10]).unwrap();
        println!("{:?}", cp);
        println!("{:?}", paths);
    }
}