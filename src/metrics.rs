// metrics.rs
use prometheus::{Counter, Encoder, Histogram, HistogramOpts, Opts, Registry, TextEncoder};
use std::sync::Arc;
use actix_web::{HttpResponse, Responder};

pub struct Metrics {
    pub registry: Arc<Registry>,
    pub http_requests_total: Counter,
    pub request_duration: Histogram,
}

impl Metrics {
    pub fn new(registry: Arc<Registry>) -> Self {
        let http_requests_total = Counter::new("http_requests_total", "Total de solicitudes HTTP")
            .expect("No se pudo crear el contador de solicitudes HTTP");

        let request_duration = Histogram::with_opts(
            HistogramOpts::from(Opts::new("http_request_duration_seconds", "Duración de las solicitudes HTTP en segundos"))
        )
            .expect("No se pudo crear el histograma de duración");

        registry
            .register(Box::new(http_requests_total.clone()))
            .expect("No se pudo registrar http_requests_total");
        registry
            .register(Box::new(request_duration.clone()))
            .expect("No se pudo registrar request_duration");

        Metrics {
            registry,
            http_requests_total,
            request_duration,
        }
    }
}

pub async fn export_metrics(registry: Arc<Registry>) -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(buffer)
}
