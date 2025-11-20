use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Execute o exercício com: cargo run <exercise_number>");
        println!("  01 - Collision attack on xor32_hash");
        // println!("  02 - Second pre-image attack");
        // println!("  03 - First pre-image attack");
        // println!("  04 - Collision with birthday paradox");
        // println!("  05 - Second pre-image brute force");
        // println!("  06 - Three partial collision using SHA256");
        return;
    }

    match args[1].as_str() {
        "01" => exercise01::run(),
        _ => println!("Exercício não encontrado!"),
    }
}

mod exercise01 {
    use super::*;

    /// xor32_hash
    ///
    /// vulnerabilidade: o padrão de shift se repete a cada 4 bytes!
    /// - Posição 0,4: shift 0  (bits 0-7)
    /// - Posição 1,5: shift 8  (bits 8-15)
    /// - Posição 2,6: shift 16 (bits 16-23)
    /// - Posição 3,7: shift 24 (bits 24-31)
    fn xor32_hash(s: &str) -> u32 {
        let mut h: u32 = 0;
        for (i, byte) in s.bytes().enumerate() {
            let shift = (i % 4) * 8;
            h ^= (byte as u32) << shift;
        }
        h
    }

    /// Analisa e mostra como cada byte contribui para o hash
    fn analyze_hash(s: &str) -> u32 {
        println!("\nString: '{}'", s);
        let mut h: u32 = 0;

        for (i, byte) in s.bytes().enumerate() {
            let shift = (i % 4) * 8;
            let contribution = (byte as u32) << shift;
            h ^= contribution;

            println!(
                "  Byte {}: '{}' (ASCII {:3}) << {:2} = 0x{:08x} -> hash = 0x{:08x}",
                i,
                byte as char,
                byte,
                shift,
                contribution,
                h
            );
        }

        println!("  Hash final: 0x{:08x}", h);
        h
    }

    /// Como o padrão de shift se repete a cada 4 bytes, podemos trocar
    /// os blocos de 4 caracteres e obter o mesmo hash!
    fn swap_blocks() -> (String, String) {
        let str1 = "ehosguri".to_string();
        // Troca os blocos: posições 0-3 com 4-7
        let str2 = format!("{}{}", &str1[4..8], &str1[0..4]);

        println!("\nString 1: {}", str1);
        println!("String 2: {}", str2);

        let hash1 = analyze_hash(&str1);
        let hash2 = analyze_hash(&str2);

        println!("\n hash1={:08x}, hash2={:08x}", hash1, hash2);
        println!("Colisão? {}", if hash1 == hash2 { "SIM" } else { "NÃO" });

        (str1, str2)
    }

    /// Verifica se duas strings colidem
    fn verify_collision(str1: &str, str2: &str) -> bool {
        let hash1 = xor32_hash(str1);
        let hash2 = xor32_hash(str2);

        println!("\n\nVERIFICAÇÃO COLISÃO:");
        
        println!("String 1: '{}' -> {:08x}", str1, hash1);
        println!("String 2: '{}' -> {:08x}", str2, hash2);
        println!("Diferentes? {}", if str1 != str2 { "SIM" } else { "NÃO" });
        println!("Hash igual? {}", if hash1 == hash2 { "SIM" } else { "NÃO" });
        println!("8 caracteres? {}", if str1.len() == 8 && str2.len() == 8 { "SIM" } else { "NÃO" });
        println!("ASCII? {}", if str1.is_ascii() && str2.is_ascii() { "SIM" } else { "NÃO" });

        if hash1 == hash2 && str1 != str2 {
            println!("\nCOLISÃO ENCONTRADA!");
            true
        } else {
            println!("\nNão é uma colisão válida");
            false
        }
    }

    fn save_solution(str1: &str, str2: &str) {
        let content = format!("{},{}\n", str1, str2);
        let path = "../solutions/exercise01.txt";

        fs::write(path, content)
            .expect("Erro ao escrever o arquivo de solução");
    }

    pub fn run() {
        println!("Ataque de Colisão em xor32_hash");

        let (str1, str2) = swap_blocks();

        if verify_collision(&str1, &str2) {
            save_solution(&str1, &str2);
        }
    }
}
