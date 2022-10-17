use log::{info, warn};
use secio::PeerId;
use std::fs;
use std::io::{Error, ErrorKind, Read};
use std::path::PathBuf;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let secret_path: PathBuf = PathBuf::from(std::env::args().nth(1).unwrap());
    let key_pair = read_secret_key(secret_path).unwrap().unwrap();
    let peer_id = PeerId::from(key_pair.public_key());
    info!("peer id: {}", peer_id.to_base58())
}

fn read_secret_key(path: PathBuf) -> Result<Option<secio::SecioKeyPair>, Error> {
    info!("read secret key from {:?}", path);
    let mut file = match fs::File::open(path.clone()) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };
    let warn = |m: bool, d: &str| {
        if m {
            warn!(
                "Your network secret file's permission is not {}, path: {:?}, \
                please fix it as soon as possible",
                d, path
            )
        }
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        warn(
            file.metadata()?.permissions().mode() & 0o177 != 0,
            "less than 0o600",
        );
    }
    #[cfg(not(unix))]
    {
        warn(!file.metadata()?.permissions().readonly(), "readonly");
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).and_then(|_read_size| {
        secio::SecioKeyPair::secp256k1_raw_key(&buf)
            .map(Some)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid secret key data"))
    })
}
