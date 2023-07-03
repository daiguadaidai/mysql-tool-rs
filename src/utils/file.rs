use std::fs;

#[allow(dead_code)]
pub fn create_dir(dir: &str) -> std::io::Result<()> {
    if dir.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(dir)
}
