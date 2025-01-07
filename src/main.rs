use std::time::Instant;

fn main() {
    let start = Instant::now();
    let resultado = funcion_a_evaluar(2_000_000);
    let duration = start.elapsed();
    println!("Resultado: {}", resultado);
    println!("Tiempo transcurrido: {:.2?}", duration);
}

fn funcion_a_evaluar(n: u64) -> u64 {
    (0..n).sum()
}
