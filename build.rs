use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use chrono::{Utc};

fn main() -> io::Result<()> {
    log_message("---------------------------------------------");
    let css_dir = "./static"; // Carpeta raíz con tus archivos CSS
    let output_file = "./static/all.css"; // Nombre del archivo combinado

    // Crear o truncar el archivo all.css
    let mut output = File::create(output_file)?;

    // Función recursiva para buscar archivos CSS en todas las subcarpetas
    fn process_directory(dir: &Path, output: &mut File) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Si es un directorio, llamar recursivamente
                process_directory(&path, output)?;
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("css") {
                // Si es un archivo CSS, leer y escribir su contenido
                let content = fs::read_to_string(&path)?;
                writeln!(output, "/* {} */", path.display())?;
                writeln!(output, "{}", content)?;
                log_message(&format!("Añadido {}", path.display()));
            }
        }
        Ok(())
    }

    // Procesar la carpeta raíz
    process_directory(Path::new(css_dir), &mut output)?;

    println!("cargo:rerun-if-changed=static"); // Para recompilar si hay cambios

    log_message("Finalizado combinar CSS.");

    Ok(())
}

fn log_message(message: &str) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("build.log")
        .unwrap();
    writeln!(file, "[{}] {}", timestamp, message).unwrap();
}