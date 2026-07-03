use std::{fs::create_dir_all, path::Path};

pub fn check_and_create_dir(path: &str) {
  if let Some(parent) = Path::new(path).parent() {
    let _ = create_dir_all(parent);
  }
}
