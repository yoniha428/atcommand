use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub fn echo(s: &str, path: &Path) {
    let mut f = fs::File::create(path)
        .unwrap_or_else(|_| panic!("Failed to open {}", path.to_string_lossy()));

    f.write_all(s.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to write to {}", path.to_string_lossy()));
}

pub fn ensure_dir<P: AsRef<Path>>(path: P) {
    // ディレクトリを作る（存在していてもOK）
    let path = path.as_ref();
    fs::create_dir_all(path)
        .unwrap_or_else(|_| panic!("Failed to ensure directory {}", path.to_string_lossy()));
}

pub fn write_if_empty<P: AsRef<Path>>(path: P, content: &str) {
    let path = path.as_ref();

    // ファイルを「読み書き」で開く（なければ作る）
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .unwrap_or_else(|_| panic!("Failed to open file {}", path.to_string_lossy()));

    // 中身をチェック
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .unwrap_or_else(|_| panic!("Failed to check if {} is empty", path.to_string_lossy()));

    // 空なら書き込む
    if buf.is_empty() {
        file.write_all(content.as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write to {}", path.to_string_lossy()));
    }
}
