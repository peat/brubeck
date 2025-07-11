//! Fuzzy string matching for better error suggestions

/// Calculate the Levenshtein distance between two strings
/// This measures how many single-character edits are needed to change one string into another
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1 = s1.to_uppercase();
    let s2 = s2.to_uppercase();

    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];

    for (i, ch1) in s1.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, ch2) in s2.chars().enumerate() {
            let cost = if ch1 == ch2 { 0 } else { 1 };
            curr_row[j + 1] = std::cmp::min(
                std::cmp::min(
                    prev_row[j + 1] + 1, // deletion
                    curr_row[j] + 1,     // insertion
                ),
                prev_row[j] + cost, // substitution
            );
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len2]
}

/// Find the closest matching instruction from a list of valid instructions
pub fn find_closest_instruction(unknown: &str, valid_instructions: &[&str]) -> Option<String> {
    let unknown_upper = unknown.to_uppercase();

    // First check for exact matches (case-insensitive)
    for &inst in valid_instructions {
        if inst.to_uppercase() == unknown_upper {
            return Some(inst.to_string());
        }
    }

    // Find instructions with minimum edit distance
    let mut best_matches: Vec<(&str, usize)> = valid_instructions
        .iter()
        .map(|&inst| (inst, levenshtein_distance(unknown, inst)))
        .filter(|(_, dist)| *dist <= 3) // Only consider reasonably close matches
        .collect();

    // Sort by distance
    best_matches.sort_by_key(|(_, dist)| *dist);

    // If we have matches with the same minimum distance, prefer shorter ones
    if let Some((_, min_dist)) = best_matches.first() {
        let min_dist = *min_dist;
        let same_dist_matches: Vec<&str> = best_matches
            .iter()
            .filter(|(_, d)| *d == min_dist)
            .map(|(inst, _)| *inst)
            .collect();

        if same_dist_matches.len() == 1 {
            return Some(same_dist_matches[0].to_string());
        } else if same_dist_matches.len() > 1 {
            // Multiple equally good matches - return them all
            return Some(same_dist_matches.join(" or "));
        }
    }

    None
}

/// Get all valid RISC-V instructions for fuzzy matching
pub fn get_all_instructions() -> Vec<&'static str> {
    vec![
        // R-type
        "ADD", "SUB", "SLL", "SLT", "SLTU", "XOR", "SRL", "SRA", "OR", "AND", // I-type
        "ADDI", "SLTI", "SLTIU", "XORI", "ORI", "ANDI", "SLLI", "SRLI", "SRAI", "LB", "LH", "LW",
        "LBU", "LHU", "JALR", // S-type
        "SB", "SH", "SW", // B-type
        "BEQ", "BNE", "BLT", "BGE", "BLTU", "BGEU", // U-type
        "LUI", "AUIPC", // J-type
        "JAL",   // System
        "ECALL", "EBREAK", "FENCE", // CSR
        "CSRRW", "CSRRS", "CSRRC", "CSRRWI", "CSRRSI", "CSRRCI", // Pseudo-instructions
        "MV", "NOT", "SEQZ", "SNEZ", "J", "JR", "RET", "LI", "LA", "NOP",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("a", ""), 1);
        assert_eq!(levenshtein_distance("", "a"), 1);
        assert_eq!(levenshtein_distance("cat", "cat"), 0);
        assert_eq!(levenshtein_distance("cat", "hat"), 1);
        assert_eq!(levenshtein_distance("cat", "cut"), 1);
        assert_eq!(levenshtein_distance("cat", "car"), 1);
        assert_eq!(levenshtein_distance("cat", "dog"), 3);

        // Case insensitive
        assert_eq!(levenshtein_distance("ADD", "add"), 0);
        assert_eq!(levenshtein_distance("AdD", "ADD"), 0);
    }

    #[test]
    fn test_find_closest_instruction() {
        let instructions = get_all_instructions();

        // Exact match (case insensitive)
        assert_eq!(
            find_closest_instruction("add", &instructions),
            Some("ADD".to_string())
        );
        assert_eq!(
            find_closest_instruction("ADD", &instructions),
            Some("ADD".to_string())
        );

        // Common typos - ADI might match multiple instructions
        let adi_result = find_closest_instruction("ADI", &instructions);
        assert!(adi_result.is_some());
        let adi_match = adi_result.unwrap();
        assert!(adi_match.contains("ADD") || adi_match.contains("ADDI"));
        assert_eq!(
            find_closest_instruction("ANDI", &instructions),
            Some("ANDI".to_string())
        );
        assert_eq!(
            find_closest_instruction("BEG", &instructions),
            Some("BEQ".to_string())
        );
        // JAMP might match JAL or JALR
        let jamp_result = find_closest_instruction("JAMP", &instructions);
        assert!(jamp_result.is_some());
        let jamp_match = jamp_result.unwrap();
        assert!(jamp_match.contains("JAL"));

        // Multiple good matches
        let result = find_closest_instruction("ST", &instructions).unwrap();
        assert!(result.contains("SB") || result.contains("SH") || result.contains("SW"));

        // Too different - no match
        assert_eq!(find_closest_instruction("XYZZY", &instructions), None);
    }
}
