use core::str;
use std::env;
use std::fs;
use rand::Rng;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sha2::{Sha256, Digest};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Execute o exercício com: cargo run <exercise_number>");
        println!("  01 - Collision attack on xor32_hash");
        println!("  02 - Second pre-image attack");
        println!("  03 - First pre-image attack");
        println!("  04 - Collision with birthday paradox");
        println!("  05 - Second pre-image brute force");
        println!("  06 - Three partial collision using SHA256");
        return;
    }

    match args[1].as_str() {
        "01" => exercise01::run(),
        "02" => exercise02::run(),
        "03" => exercise03::run(),
        "04" => exercise04::run(),
        "05" => exercise05::run(),
        "06" => exercise06::run(),
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

fn simple_hash_bytes(bytes: &[u8]) -> u32 {
    let mut hash_value = 0_u32;
    for &byte in bytes {
        hash_value = hash_value
            .wrapping_shl(5)
            .wrapping_sub(hash_value)
            .wrapping_add(byte as u32);
    }
    hash_value
}

fn simple_hash(s: &str) -> u32 {
    simple_hash_bytes(s.as_bytes())
}


fn save_solution(exercise_number: &str, str1: &str, str2: Option<&str>, str3: Option<&str>) {
    let content = match (str2, str3) {
        (Some(s2), Some(s3)) => format!("{},{},{}", str1, s2, s3),
        (Some(s2), None) => format!("{},{}", str1, s2),
        (None, None) => format!("{}", str1),
        (None, Some(_)) => panic!("str2 deve estar presente se str3 estiver presente"),
    };

    let path = format!("../solutions/exercise{}.txt", exercise_number);

    fs::write(path, content)
        .expect("Erro ao escrever o arquivo");
}

fn generate_string(_len: usize) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<u8> = (b'a'..=b'z')
        .chain(b'A'..=b'Z')
        .chain(b'0'..=b'9')
        .collect();

    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx] as char
        })
        .collect()
}

fn starts_with_hex_pattern(hash: &[u8], pattern: &str) -> bool {
    let hex_hash = hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    hex_hash.starts_with(pattern)
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

    pub fn run() {
        println!("Ataque de Colisão em xor32_hash");

        let (str1, str2) = swap_blocks();

        if verify_collision(&str1, &str2) {
            save_solution("01", &str1, Some(&str2), None);
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

    pub fn run() {
        println!("Second Pre-image Attack");
        println!();

        let target = "bitcoin0";

        if let Some(solution) = find_second_preimage(target) {
            println!("  Segunda pré-imagem encontrada!");
            println!("  Original: '{}'", target);
            println!("  Encontrada: '{}'", solution);
            println!("  Hash: {:08x}", xor32_hash(&solution));

            save_solution("02", &solution, None, None);
        } else {
            println!("Não foi possível encontrar segunda pré-imagem com métodos simples.");
        }
    }
}

mod exercise03 {
    use super::*;

    fn find_preimage(target_hash: u32) -> Option<String> {
        println!("Target hash: {:08x}", target_hash);

        // 4 bytes do hash
        let b0 = (target_hash & 0x000000FF) as u8;
        let b1 = ((target_hash & 0x0000FF00) >> 8) as u8;
        let b2 = ((target_hash & 0x00FF0000) >> 16) as u8;
        let b3 = ((target_hash & 0xFF000000) >> 24) as u8;

        println!("Bytes do hash: 0x{:02x} 0x{:02x} 0x{:02x} 0x{:02x}", b0, b1, b2, b3);
        
        // primeiros 4 qualquer coisa (ASCII gráfico)
        
        let c0 = b'1';
        let c1 = b'2'; 
        let c2 = b'3';
        let c3 = b'4';
        
        // Hash parcial dos primeiros 4 bytes
        let partial = (c0 as u32) | ((c1 as u32) << 8) | ((c2 as u32) << 16) | ((c3 as u32) << 24);
        
        // target_hash = partial XOR (c4 | c5<<8 | c6<<16 | c7<<24)
        // remaining = target_hash XOR partial
        let remaining = target_hash ^ partial;
        
        let c4 = (remaining & 0xFF) as u8;
        let c5 = ((remaining >> 8) & 0xFF) as u8;
        let c6 = ((remaining >> 16) & 0xFF) as u8;
        let c7 = ((remaining >> 24) & 0xFF) as u8;
        
        println!("Primeiros 4: {} {} {} {}", c0 as char, c1 as char, c2 as char, c3 as char);
        println!("Calculados: 0x{:02x} 0x{:02x} 0x{:02x} 0x{:02x}", c4, c5, c6, c7);
    
        let result = format!("{}{}{}{}{}{}{}{}", 
            c0 as char, c1 as char, c2 as char, c3 as char,
            c4 as char, c5 as char, c6 as char, c7 as char);
        println!("✓ Encontrado: '{}'", result);
        return Some(result);
    }

    pub fn run() {
        println!("Pre-image Attack '1b575451'");
        println!();

        let target_hash: u32 = 0x1b575451;
        
        if let Some(solution) = find_preimage(target_hash) {
            println!("  Pré-imagem encontrada!");
            println!("  Encontrada: '{}'", solution);
            println!("  Hash: {:08x}", xor32_hash(&solution));

            save_solution("03", &solution, None, None);
        } else {
            println!("Não foi possível encontrar pré-imagem com métodos simples.");
        }
    }
}

mod exercise04 {
    use super::*;
    use std::collections::HashMap;


    fn find_collision_birthday() -> Option<(String, String)> {

        let mut hash_map: HashMap<u32, String> = HashMap::new();
        let mut attempts = 0;

        loop {
            attempts += 1;

            let candidate = generate_string(8);
            let hash = simple_hash(&candidate);

            if let Some(original) = hash_map.get(&hash) {
                println!("\n✓ COLISÃO ENCONTRADA após {} tentativas!", attempts);
                println!("  String 1: '{}'", original);
                println!("  String 2: '{}'", candidate);
                println!("  Hash: {:08x}", hash);

                return Some((original.clone(), candidate));
            }

            hash_map.insert(hash, candidate);

        }
    }

    pub fn run() {
        println!();

        if let Some((str1, str2)) = find_collision_birthday() {
            let hash1 = simple_hash(&str1);
            let hash2 = simple_hash(&str2);
            
            println!("  '{}' -> {:08x}", str1, hash1);
            println!("  '{}' -> {:08x}", str2, hash2);
            println!("  Iguais? {}", hash1 == hash2);

            save_solution("04", &str1, Some(&str2), None);
        }
    }

}

mod exercise05 {
    use super::*;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    fn find_name_second_preimage(name: &str) -> Option<String> {
        let target_hash = simple_hash(name);
        println!("Buscando colisão para: '{}' (Hash: {:08x})", name, target_hash);
        
        let found_flag = Arc::new(AtomicBool::new(false));

        let result = (0..u64::MAX).into_par_iter().find_map_any(|seed| {
            if found_flag.load(Ordering::Relaxed) {
                return None;
            }

            let mut buffer = [0u8; 8]; 
            let mut n = seed;
            let len = CHARSET.len() as u64;

            for i in 0..8 {
                buffer[i] = CHARSET[(n % len) as usize];
                n /= len;
            }

            let hash = simple_hash_bytes(&buffer);

            if hash == target_hash {
                let candidate = String::from_utf8_lossy(&buffer).to_string();
                if candidate != name {
                    found_flag.store(true, Ordering::Relaxed);
                    return Some(candidate);
                }
            }
            None
        });

        if let Some(ref s) = result {
            println!("SEGUNDA PRÉ-IMAGEM ENCONTRADA: '{}'", s);
        }
        
        result
    }

    pub fn run() {
        println!();

        let start = std::time::Instant::now();
        
        if let Some(str_random) = find_name_second_preimage("emmanuel") {
            let hash_name = simple_hash("emmanuel");
            let hash_str = simple_hash(&str_random);
            
            save_solution("05", "emmanuel", Some(&str_random), None);

            println!("  '{}' -> {:08x}", "emmanuel", hash_name);
            println!("  '{}' -> {:08x}", str_random, hash_str);
            println!("  Tempo decorrido: {:?}", start.elapsed());
        }
    }
}

mod exercise06 {
    use super::*;

    fn find_partial_collision(prefix: &str, hex_pattern: &str) -> String {
        let found = Arc::new(AtomicBool::new(false));
        let num_threads = rayon::current_num_threads();

        let result = (0..num_threads)
            .into_par_iter()
            .find_map_any(|thread_id| {
                let mut counter = thread_id as u64;
                let max_iterations = 100_000_000_u64;
                
                while counter < max_iterations {
                    if found.load(Ordering::Relaxed) {
                        return None;
                    }
                    
                    let candidate = format!("{}{}", prefix, counter);
                    let hash = Sha256::digest(candidate.as_bytes());
                    
                    if starts_with_hex_pattern(&hash, hex_pattern) {
                        found.store(true, Ordering::Relaxed);
                        return Some(candidate);
                    }
                    
                    counter += num_threads as u64;
                }
                
                None
        });

        result.expect("Nenhuma colisão encontrada")
    }

    pub fn run () {
        println!();

        let start = std::time::Instant::now();

        let result1 = find_partial_collision("bitcoin", "cafe");
        println!("Encontrado para 0xcafe: {}", result1);

        let result2 = find_partial_collision("bitcoin", "faded");
        println!("Encontrado para 0xfaded: {}", result2);

        let result3 = find_partial_collision("bitcoin", "decade");
        println!("Encontrado para 0xdecade: {}", result3);

        println!("\n\n  Tempo decorrido: {:?}", start.elapsed());

        save_solution("06", &result1, Some(&result2), Some(&result3));
    }
}