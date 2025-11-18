use crate::{
    harris_corner::{harris_corner_detector, Descriptor},
    io::Image,
};

pub fn l1_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

#[derive(Debug, Clone)]
pub struct Match {
    pub a_index: usize,
    pub b_index: usize,
    pub distance: f32,
}

pub fn match_descriptors(a: &Vec<Descriptor>, b: &Vec<Descriptor>) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::with_capacity(a.len());
    for (i, desc_a) in a.iter().enumerate() {
        let mut best_distance = f32::MAX;
        let mut best_index = 0;

        // Compare desc_a against every descriptor in b
        for (j, desc_b) in b.iter().enumerate() {
            let dist = l1_distance(&desc_a.data, &desc_b.data);

            if dist < best_distance {
                best_distance = dist;
                best_index = j;
            }
        }

        // Record the raw match
        matches.push(Match {
            a_index: i,
            b_index: best_index,
            distance: best_distance,
        });
    }

    matches.sort_by(|m1, m2| {
        m1.distance
            .partial_cmp(&m2.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut seen_b = vec![false; b.len()];
    let mut unique_matches = Vec::new();

    for m in matches {
        // If this spot in 'b' hasn't been taken yet...
        if !seen_b[m.b_index] {
            // ...claim it!
            seen_b[m.b_index] = true;
            unique_matches.push(m);
        }
        // If it WAS taken, we ignore this match. Because we sorted first,
        // the previous match that took it was definitely "better" (smaller distance).
    }

    unique_matches
}
