# skynet-rs

Rust implementation of the [Sia Skynet](https://siasky.net) API.

```rust
use skynet::{SkynetClient, UploadOptions, DownloadOptions, MetadataOptions};

let client = SkynetClient::default();
let skylink = client.upload_file("hello.txt", UploadOptions::default()).await?;
let data = client.download_data(&skylink, DownloadOptions::default()).await?;
let metadata = client.get_metadata(&skylink, MetadataOptions::default()).await?;
```

## Features

- Upload files
- Download files
- Metadata
