#[allow(dead_code)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag, // あとなんかあるっけ
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ObjectType::*;
        write!(
            f,
            "{}",
            match self {
                Blob => "blob",
                Tree => "tree",
                Commit => "commit",
                Tag => "tag",
            }
        )
    }
}

#[must_use]
pub fn compress_object(
    objtype: ObjectType,
    data: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    use flate2::{write::ZlibEncoder, Compression};
    use sha1::Digest as _;
    use std::io::prelude::*;

    let content = format!(
        "{} {}\0{}",
        objtype,
        data.len(),
        std::str::from_utf8(&data)?
    );

    let hash = sha1::Sha1::digest(content.as_bytes());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content.as_bytes())?;
    let content = encoder.finish()?;
    Ok((hash.as_slice().to_owned(), content))
}

#[must_use]
pub fn decompress_object(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read as _;

    let mut decoder = ZlibDecoder::new(data);
    let mut res = Vec::new();
    decoder.read_to_end(&mut res)?;

    Ok(res)
}
