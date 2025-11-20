use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Execute o exercício com: cargo run <exercise_number>");
        println!("  01 - Collision attack on xor32_hash");
        println!("  02 - Second pre-image attack");
        // println!("  03 - First pre-image attack");
        // println!("  04 - Collision with birthday paradox");
        // println!("  05 - Second pre-image brute force");
        // println!("  06 - Three partial collision using SHA256");
        return;
    }

    match args[1].as_str() {
        "01" => exercise01::run(),
        "02" => exercise02::run(),
        _ => println!("Exercício não encontrado!"),
    }
}

fn xor32_hash(s: &str) -> u32 {
    let mut h: u32 = 0;
    for (i, byte) in s.bytes().enumerate() {
        let shift = (i % 4) * 8;
        h ^= (byte as u32) << shift;
    }
    h
}

mod exercise01 {
    use super::*;

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

mod exercise02 {
    use super::*;

    /// Second Pre-image Attack:
    ///
    /// "bitcoin0" tem 8 bytes e produz um hash específico.
    /// Método 1: Se "bitcoin0" tiver exatamente 8 caracteres, trocamos os blocos
    fn find_second_preimage(target: &str) -> Option<String> {
        let target_hash = xor32_hash(target);

        println!("Target: '{}'", target);
        println!("Target hash: {:08x}", target_hash);
        println!("Target length: {} bytes\n", target.len());

        // Método 1: Se tiver 8 bytes, trocar blocos de 4
        if target.len() == 8 {
            let candidate = format!("{}{}", &target[4..8], &target[0..4]);
            let candidate_hash = xor32_hash(&candidate);

            println!("  Candidate: '{}'", candidate);
            println!("  Candidate hash: {:08x}", candidate_hash);

            if candidate_hash == target_hash && candidate != target {
                return Some(candidate);
            }
        }

        None
    }

    fn save_solution(solution: &str) {
        let content = format!("{}\n", solution);
        let path = "../solutions/exercise02.txt";

        fs::write(path, content)
            .expect("Erro ao escrever o arquivo");
    }

    pub fn run() {
        println!("Second Pre-image Attack");
        println!();

        let target = "bitcoin0";

        if let Some(solution) = find_second_preimage(target) {
            println!("  Segunda pré-imagem encontrada!");
            println!("  Original: '{}'", target);
            println!("  Encontrada: '{}'", solution);
            println!("  Hash: {:08x}", xor32_hash(&solution));

            save_solution(&solution);
        } else {
            println!("Não foi possível encontrar segunda pré-imagem com métodos simples.");
        }
    }
}