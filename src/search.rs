use std::collections::HashSet;

pub struct Match {
    pub row: usize,
    pub col: usize,
    pub len: usize,
    pub label: Option<String>,
}

/// Non-overlapping substring search, per physical line. Smartcase: the search
/// is ASCII case-insensitive until the query contains an uppercase character.
/// `col` is a character index, which is exactly the `cursor-right` count tmux
/// copy-mode needs to land on the match.
pub fn find(grid: &[Vec<char>], query: &[char]) -> Vec<Match> {
    let mut out = Vec::new();
    let m = query.len();
    if m == 0 {
        return out;
    }
    let sensitive = query.iter().any(|c| c.is_ascii_uppercase());
    let eq = |a: &char, b: &char| {
        if sensitive {
            a == b
        } else {
            a.eq_ignore_ascii_case(b)
        }
    };
    for (row, chars) in grid.iter().enumerate() {
        let n = chars.len();
        if m > n {
            continue;
        }
        let mut i = 0;
        while i + m <= n {
            if (0..m).all(|k| eq(&chars[i + k], &query[k])) {
                out.push(Match {
                    row,
                    col: i,
                    len: m,
                    label: None,
                });
                i += m;
            } else {
                i += 1;
            }
        }
    }
    out
}

/// Give every match a jump label from `pool`. Characters that immediately
/// follow a match are excluded so that typing the next letter of the word
/// always extends the search instead of being swallowed as a label (flash's
/// trick).
pub fn assign_labels(grid: &[Vec<char>], mut matches: Vec<Match>, pool: &[char]) -> Vec<Match> {
    if matches.is_empty() {
        return matches;
    }
    let mut forbidden = HashSet::new();
    for m in &matches {
        if let Some(c) = grid[m.row].get(m.col + m.len) {
            forbidden.insert(c.to_ascii_lowercase());
        }
    }
    let pool: Vec<char> = pool
        .iter()
        .filter(|c| !forbidden.contains(&c.to_ascii_lowercase()))
        .copied()
        .collect();
    let labels = gen_labels(&pool, matches.len());
    for (m, l) in matches.iter_mut().zip(labels) {
        m.label = Some(l);
    }
    matches
}

fn gen_labels(pool: &[char], n: usize) -> Vec<String> {
    if pool.is_empty() {
        return Vec::new();
    }
    if n <= pool.len() {
        return pool.iter().take(n).map(|c| c.to_string()).collect();
    }
    // Overflow: two-char labels give pool.len()^2 capacity.
    let mut v = Vec::with_capacity(n);
    'outer: for a in pool {
        for b in pool {
            v.push(format!("{}{}", a, b));
            if v.len() >= n {
                break 'outer;
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_LABELS;

    fn g(lines: &[&str]) -> Vec<Vec<char>> {
        lines.iter().map(|l| l.chars().collect()).collect()
    }

    fn pool() -> Vec<char> {
        DEFAULT_LABELS.chars().collect()
    }

    #[test]
    fn finds_case_insensitive_nonoverlapping() {
        let grid = g(&["Foo foo FOO", "bar"]);
        let q: Vec<char> = "foo".chars().collect();
        let m = find(&grid, &q);
        assert_eq!(m.len(), 3);
        assert_eq!((m[0].row, m[0].col), (0, 0));
        assert_eq!((m[1].row, m[1].col), (0, 4));
        assert_eq!((m[2].row, m[2].col), (0, 8));
    }

    #[test]
    fn smartcase_uppercase_query_is_sensitive() {
        let grid = g(&["Foo foo FOO"]);
        let q: Vec<char> = "Foo".chars().collect();
        let m = find(&grid, &q);
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].row, m[0].col), (0, 0));
    }

    #[test]
    fn labels_avoid_continuation_char() {
        // "the" followed by 'm' and ' ' -> 'm' must not be a label.
        let grid = g(&["them the theory"]);
        let q: Vec<char> = "the".chars().collect();
        let m = assign_labels(&grid, find(&grid, &q), &pool());
        assert!(m.iter().all(|x| x.label.as_deref() != Some("m")));
        assert!(m.iter().all(|x| x.label.is_some()));
    }

    #[test]
    fn empty_query_no_matches() {
        assert!(find(&g(&["abc"]), &[]).is_empty());
    }
}
