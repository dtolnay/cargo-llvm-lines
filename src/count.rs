use rustc_demangle::demangle;
use std::collections::HashMap as Map;

#[derive(Default)]
pub struct Instantiations {
    pub copies: usize,
    pub total_lines: usize,
}

impl Instantiations {
    fn record_lines(&mut self, lines: usize) {
        self.copies += 1;
        self.total_lines += lines;
    }
}

pub fn count_lines(instantiations: &mut Map<String, Instantiations>, ir: &[u8]) {
    let mut current_function = None;
    let mut count = 0;

    for line in String::from_utf8_lossy(ir).lines() {
        if line.starts_with("define ") {
            current_function = parse_function_name(line);
        } else if line == "}" {
            if let Some(name) = current_function.take() {
                instantiations
                    .entry(name)
                    .or_insert_with(Default::default)
                    .record_lines(count);
            }
            count = 0;
        } else if line.starts_with("  ") && !line.starts_with("   ") {
            count += 1;
        }
    }
}

fn parse_function_name(line: &str) -> Option<String> {
    let start = line.find('@')? + 1;
    let end = line[start..].find('(')?;
    let mangled = line[start..start + end].trim_matches('"');
    let mut name = demangle(mangled).to_string();
    if has_hash(&name) {
        let len = name.len() - 19;
        name.truncate(len);
    }
    Some(name)
}

fn has_hash(name: &str) -> bool {
    let mut bytes = name.bytes().rev();
    for _ in 0..16 {
        if !bytes.next().map_or(false, is_ascii_hexdigit) {
            return false;
        }
    }
    bytes.next() == Some(b'h') && bytes.next() == Some(b':') && bytes.next() == Some(b':')
}

fn is_ascii_hexdigit(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}
