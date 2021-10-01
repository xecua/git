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
pub fn compress_object(objtype: ObjectType, data: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
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
pub fn decompress_object(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read as _;

    let mut decoder = ZlibDecoder::new(data);
    let mut res = Vec::new();
    decoder.read_to_end(&mut res)?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::{compress_object, decompress_object};

    #[test]
    fn compress_simple() {
        let dat = "abc".as_bytes();
        let (hash, content) = compress_object(super::ObjectType::Blob, dat).unwrap();
        #[rustfmt::skip]
        assert_eq!( hash, [ 242, 186, 143, 132, 171, 92, 27, 206, 132, 167, 180, 65, 203, 25, 89, 207, 199, 9, 59, 127 ]);
        #[rustfmt::skip]
        assert_eq!(content, [120, 156, 75, 202, 201, 79, 82, 48, 102, 72, 76, 74, 6, 0, 17, 217, 3, 25]);
    }

    #[test]
    fn decompress_simple() {
        #[rustfmt::skip]
        let dat = [120, 156, 75, 202, 201, 79, 82, 48, 102, 72, 76, 74, 6, 0, 17, 217, 3, 25];
        let res = decompress_object(&dat).unwrap();
        assert_eq!(res, "blob 3\0abc".as_bytes());
    }
}
