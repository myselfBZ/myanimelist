use std::pin::Pin;
use s3::{Bucket, Region, creds::Credentials, error::S3Error};
use tokio::io::AsyncRead;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ObjStore {
    bucket: Box<Bucket>,
}

pub struct Config {
    pub endpoint: String,
    pub region: String,
    pub bucket_name: String,
    pub secret_key: String,
    pub access_id: String,
}

pub struct PresignPutReq {
    pub content_type: String,
    pub dir: String,
    pub filename: String,
}

pub struct UploadReq<'a> {
    pub reader: Pin<&'a mut (dyn AsyncRead + Send)>,
    pub content_type: String,
    pub dir: String,
    pub filename: String,
}

impl ObjStore {

    pub fn new(
        cfg: Config
    ) -> Result<Self, S3Error> {
        let region = Region::Custom {
            region: cfg.region,
            endpoint: cfg.endpoint
        }; 
        let creds = Credentials::new(
            Some(&cfg.access_id), 
            Some(&cfg.secret_key), 
            None, 
            None, 
            None
            )?;
        let bucket = Bucket::new(&cfg.bucket_name, region, creds)?;
        Ok(Self { bucket: bucket })
    }

    pub async fn presign_put(&self, req: PresignPutReq) -> Result<String, S3Error> {
        let id = Uuid::new_v4();
        let path = format!("{}/{}-{}", req.dir, id, req.filename);

        self
            .bucket
            .presign_put(path, 60 * 60, None, None)
            .await
    }

    // Direct upload using streaming
    // Alternative to presigned url uploading
    pub async fn put_file<'a>( &self, mut req: UploadReq<'a>) -> Result<String, S3Error> {
        let id = Uuid::new_v4();
        let path = format!("{}/{}-{}", req.dir, id, req.filename);
        self.bucket
            .put_object_stream_with_content_type(
                &mut req.reader, 
                path.clone(),
                req.content_type
            )
            .await?;
        Ok(format!("/{}", path))
    }

    pub async fn presign_get(&self, key: String) -> Result<String, S3Error> {
        self
            .bucket
            // TODO: change it
            .presign_get(key, 60 * 60 * 24, None)
            .await
    }
}
