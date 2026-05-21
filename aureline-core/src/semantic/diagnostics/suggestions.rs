/// Cheap edit-distance for "did you mean" suggestions. Levenshtein with a
/// distance cap of 3 - anything further isn't really a typo.
pub(crate) fn closest_match(target: &str, candidates: &[&str]) -> Option<String> {
    let mut best: Option<(&str, usize)> = None;
    for candidate in candidates {
        let distance = levenshtein(target, candidate);
        if distance <= 3 && best.is_none_or(|(_, best_distance)| distance < best_distance) {
            best = Some((candidate, distance));
        }
    }
    best.map(|(candidate, _)| candidate.to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let mut prev = (0..=b.len()).collect::<Vec<_>>();
    let mut curr = vec![0; b.len() + 1];

    for (i, ac) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, bc) in b.iter().enumerate() {
            let cost = usize::from(ac != bc);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b.len()]
}
