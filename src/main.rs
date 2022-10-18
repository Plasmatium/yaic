mod compressor;
mod handler;

use handler::{HandlerConfig, Handler};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let h = Handler::new("/Users/jonnywong/Pictures/test-compress/origin", "/Users/jonnywong/Pictures/test-compress/dist", HandlerConfig::new());
    h.run().await.unwrap();
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_read_dir() {
        let dirs = std::fs::read_dir("/Users/jonnywong/Pictures/compressor-test/origin").unwrap();
        // let mut files = vec![];
        for p in dirs {
            let p = p.unwrap();
            println!("{}", p.path().display());
        }
    }
}