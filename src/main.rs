use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<()> {
    let wordfile = BufReader::new(File::open("words_alpha.txt")?);
    let all_words: Vec<String> = wordfile.lines().try_collect()?;

    // Only length 5 words.
    let five_words = all_words
        .iter()
        .map(|w| w.trim())
        .filter(|w| w.len() == 5)
        .sorted()
        .collect_vec();

    // One bit per letter as bitset.
    let word_masks = five_words
        .iter()
        .map(|w| {
            let mut mask = 0u32;
            let mut duplicate_letter = false;
            for c in w.bytes() {
                let letter_idx = c as i32 - b'a' as i32;
                assert!(letter_idx >= 0 && letter_idx < 26);
                duplicate_letter |= mask & (1 << letter_idx) > 0;
                mask |= 1 << letter_idx;
            }
            (mask, duplicate_letter)
        })
        .collect_vec();

    // Deduplicate and sort.
    let uniq_word_masks = word_masks
        .iter()
        .filter_map(|(mask, dup)| (!dup).then_some(*mask))
        .sorted()
        .dedup()
        .collect_vec();

    // Find valid solutions using only the bitsets.
    let mask_solutions: Vec<[u32; 5]> = uniq_word_masks
        .par_iter()
        .flat_map(|&m1| {
            let mask1 = m1;
            let mut uniq_masks2 = Vec::with_capacity(uniq_word_masks.len());
            let mut uniq_masks3 = Vec::with_capacity(uniq_word_masks.len());
            let mut uniq_masks4 = Vec::with_capacity(uniq_word_masks.len());

            let filter = |mask: u32, m: u32, uniq: &[u32], filtered: &mut Vec<u32>| {
                filtered.clear();
                filtered.extend(
                    uniq
                        .iter()
                        .copied()
                        .take_while(move |m2| *m2 < m) // Strictly descending to avoid permutations.
                        .filter(move |m2| m2 & mask == 0) // Empty intersection to avoid duplicate letters.
                );
            };

            let mut solutions = Vec::new();
            filter(mask1, m1, &uniq_word_masks, &mut uniq_masks2);
            for &m2 in &uniq_masks2 {
                let mask2 = mask1 | m2;
                filter(mask2, m2, &uniq_masks2, &mut uniq_masks3);
                for &m3 in &uniq_masks3 {
                    let mask3 = mask2 | m3;
                    filter(mask3, m3, &uniq_masks3, &mut uniq_masks4);
                    for &m4 in &uniq_masks4 {
                        let mask4 = mask3 | m4;
                        for &m5 in &uniq_masks4 {
                            if m5 > m4 {
                                break;
                            }

                            if m5 & mask4 == 0 {
                                solutions.push([m1, m2, m3, m4, m5]);
                            }
                        }
                    }
                }
            }
            solutions
        })
        .collect();

    let solutions = mask_solutions
        .into_iter()
        .flat_map(|mask_solution| {
            // Gather word solutions from mask.
            let word_solutions = mask_solution
                .into_iter()
                .map(|sol_mask| {
                    five_words
                        .iter()
                        .zip(word_masks.iter())
                        .filter_map(move |(w, (m, _dup))| (sol_mask == *m).then_some(*w))
                })
                .multi_cartesian_product();
            
            // Sort the 5 words in each solution.
            word_solutions.map(|sol| sol.into_iter().sorted().collect_vec())
        })
        .sorted();

    for sol in solutions {
        println!("{:?}", sol);
    }
    Ok(())
}
