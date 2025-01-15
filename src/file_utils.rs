use std::{fs, io};
use sha2::{Sha256, Digest};

pub(crate) fn generate_etag(vec_file: &Vec<u8>) -> Result<String, io::Error> {
//Result<String, es tipo String porque se tiene que saber el tamaño al compilar
    let mut hasher = Sha256::new();

    hasher.update(&vec_file);

    let etag = hasher.finalize();

    //Recordar dejarlo en minúsculas porque librerías, navegadores, etc
    // trabajan con el etag convertido a minúscula

    //return
    Ok(format!("{:x}", etag))
}

pub(crate) fn load_file(file_path: &str) -> Result<(String, Vec<u8>), std::io::Error> {

    let file_content = fs::read(file_path)
        .map_err(|e
        | std::io::Error::new(e.kind(),
                              format!("Failed to read file: {}", e)))?;

    let etag  = generate_etag(&file_content)?;

    //return
    Ok((etag, file_content))
}