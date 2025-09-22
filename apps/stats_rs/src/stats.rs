pub fn mean(xs: &[f64]) -> f64 {
    xs.iter().copied().sum::<f64>() / xs.len() as f64
}

pub fn median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

pub fn sample_std_dev(xs: &[f64], mean: f64) -> f64 {
    let n = xs.len();
    if n < 2 {
        return 0.0;
    }
    let var = xs
        .iter()
        .map(|x| {
            let d = *x - mean;
            d * d
        })
        .sum::<f64>()
        / (n as f64 - 1.0);
    var.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_small() {
        let xs = vec![1.0, 2.0, 3.0, 4.0];
        let m = mean(&xs);
        assert!((m - 2.5).abs() < 1e-12);
        let med = median(&xs);
        assert!((med - 2.5).abs() < 1e-12);
        let sd = sample_std_dev(&xs, m);
        assert!((sd - 1.2909944487358056).abs() < 1e-12);
    }
}
