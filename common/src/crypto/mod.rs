//! Простой шифратор

const SESSION_XOR_KEY: &[u8] = b"Robotics is the art of making dreams come true in metal and wires";

/// Простейшая шифровка/расшифровка с помощью XOR и статичного ключа
///
/// **Аргументы:**
/// - `data`: &[[u8]] - данные в байтовом виде
/// 
/// **Выходные данные:**
/// [Vec]<[u8]> - зашифрованные данные
pub fn xor_crypt(data: &[u8]) -> Vec<u8> {
    data.iter()
        .zip(SESSION_XOR_KEY.iter().cycle())
        .map(|(&b, &k)| b ^ k)
        .collect()
}
