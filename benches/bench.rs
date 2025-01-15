// extern crate test;
// use test::Bencher;
// use std::time::Instant;
//
// fn funcion_a_evaluar(n: u64) -> u64 {
//     (0..n).sum()
// }
//
// #[bench]
// fn bench_funcion_a_evaluar(b: &mut Bencher) {
//     b.iter(|| {
//         let texto :&str = "Aa";
//         let start = Instant::now();
//         let result = funcion_a_evaluar(1000);
//         let duration = start.elapsed();
//         println!("Tiempo transcurrido: {:.2?}", duration);
//         result
//     });
// }