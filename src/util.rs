use anyhow::Result;
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub fn echo(s: &str, path: &Path) -> Result<()> {
    let mut f = fs::File::create(path)?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()>{
    // ディレクトリを作る（存在していてもOK）
    let path = path.as_ref();
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_if_empty<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    // ファイルを「読み書き」で開く（なければ作る）
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;

    // 中身をチェック
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    // 空なら書き込む
    if buf.is_empty() {
        file.write_all(content.as_bytes())?;
    }
    Ok(())
}
