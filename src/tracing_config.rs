use tracing_subscriber::fmt::Subscriber;
use tracing::{info};

pub fn init_tracing() {
    let subscriber = Subscriber::builder()
        .with_env_filter("info") // Nivel de log configurado aquí
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("No se pudo configurar el suscriptor global");

    info!("Tracing inicializado");
}
