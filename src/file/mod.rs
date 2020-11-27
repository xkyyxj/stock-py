use async_std::fs::File;
use futures::AsyncReadExt;

pub async fn read_txt_file(file_path: String) -> String {
    let mut file = File::open("table_config").await.unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await.unwrap();
    String::from_utf8(contents).unwrap()
}