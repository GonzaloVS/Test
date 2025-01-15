use std::io;
use std::path::Path;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

// Combinar todos los archivos CSS en uno
pub(crate) async fn combine_css(css_dir: &str, output_file: &str) -> io::Result<()> {
    let mut output = tokio::fs::File::create(output_file).await?;
    let mut stack = vec![Path::new(css_dir).to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("css") {
                let content = tokio::fs::read_to_string(&path).await?;
                output
                    .write_all(format!("/* {} */\n", path.display()).as_bytes())
                    .await?;
                output.write_all(content.as_bytes()).await?;
                println!("CSS añadido: {}", path.display());
            }
        }
    }

    println!("CSS combinado en '{}'", output_file);
    Ok(())
}

// Monitorear cambios en CSS
pub(crate) async fn monitor_changes(css_dir: &str, output_file: &str) -> io::Result<()> {
    println!("Monitoreando cambios en '{}'", css_dir);

    let (tx, mut rx) = mpsc::channel(1);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.try_send(event);
            }
        },
        Default::default(),
    )
        .unwrap();

    watcher
        .watch(Path::new(css_dir), RecursiveMode::Recursive)
        .unwrap();

    while let Some(event) = rx.recv().await {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                println!("Detectado cambio: {:?}", event);
                if let Err(e) = combine_css(css_dir, output_file).await {
                    eprintln!("Error al combinar CSS: {}", e);
                }
            }
            _ => {
                println!("Evento ignorado: {:?}", event.kind);
            }
        }
    }

    Ok(())
}