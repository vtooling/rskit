//! Basic descriptive statistics over `f64` slices.

/// Sum of the values. Returns `0.0` for empty input.
pub fn sum(data: &[f64]) -> f64 {
    data.iter().copied().sum()
}

/// Minimum value, or `None` if empty.
pub fn min(data: &[f64]) -> Option<f64> {
    data.iter()
        .copied()
        .fold(None, |acc, x| Some(acc.map_or(x, |a| a.min(x))))
}

/// Maximum value, or `None` if empty.
pub fn max(data: &[f64]) -> Option<f64> {
    data.iter()
        .copied()
        .fold(None, |acc, x| Some(acc.map_or(x, |a| a.max(x))))
}

/// Arithmetic mean, or `None` if empty.
pub fn mean(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    Some(sum(data) / data.len() as f64)
}

/// Median value (interpolated for even-length inputs), or `None` if empty.
pub fn median(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let mut v: Vec<f64> = data.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len().is_multiple_of(2) {
        Some((v[mid - 1] + v[mid]) / 2.0)
    } else {
        Some(v[mid])
    }
}

/// Population variance, or `None` if empty.
pub fn variance(data: &[f64]) -> Option<f64> {
    let m = mean(data)?;
    if data.is_empty() {
        return None;
    }
    let acc: f64 = data.iter().map(|x| (x - m).powi(2)).sum();
    Some(acc / data.len() as f64)
}

/// Population standard deviation, or `None` if empty.
pub fn stddev(data: &[f64]) -> Option<f64> {
    variance(data).map(|v| v.sqrt())
}

/// Percentile using linear interpolation. `p` is in `0.0..=100.0`.
/// Returns `None` if empty or `p` is out of range.
pub fn percentile(data: &[f64], p: f64) -> Option<f64> {
    if data.is_empty() || !(0.0..=100.0).contains(&p) {
        return None;
    }
    let mut v: Vec<f64> = data.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if v.len() == 1 {
        return Some(v[0]);
    }
    let rank = (p / 100.0) * (v.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        Some(v[lo])
    } else {
        let frac = rank - lo as f64;
        Some(v[lo] + (v[hi] - v[lo]) * frac)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<f64> {
        vec![1.0, 2.0, 3.0, 4.0, 5.0]
    }

    #[test]
    fn sum_min_max_mean() {
        let d = sample();
        assert_eq!(sum(&d), 15.0);
        assert_eq!(min(&d), Some(1.0));
        assert_eq!(max(&d), Some(5.0));
        assert_eq!(mean(&d), Some(3.0));
    }

    #[test]
    fn median_odd_and_even() {
        assert_eq!(median(&sample()), Some(3.0));
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
    }

    #[test]
    fn variance_stddev() {
        let d = sample();
        assert_eq!(variance(&d), Some(2.0));
        assert!((stddev(&d).unwrap() - 2f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn percentile_cases() {
        let d = sample();
        assert_eq!(percentile(&d, 0.0), Some(1.0));
        assert_eq!(percentile(&d, 100.0), Some(5.0));
        assert_eq!(percentile(&d, 50.0), Some(3.0));
        assert_eq!(percentile(&d, 25.0), Some(2.0));
        assert_eq!(percentile(&d, 75.0), Some(4.0));
    }

    #[test]
    fn percentile_interpolation() {
        // 0,10,20,30,40,50  -> p=25 => 12.5
        let d = vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&d, 25.0), Some(12.5));
    }

    #[test]
    fn empty_returns_none() {
        let empty: Vec<f64> = vec![];
        assert_eq!(min(&empty), None);
        assert_eq!(mean(&empty), None);
        assert_eq!(median(&empty), None);
        assert_eq!(percentile(&empty, 50.0), None);
    }

    #[test]
    fn percentile_out_of_range() {
        assert_eq!(percentile(&sample(), 150.0), None);
        assert_eq!(percentile(&sample(), -1.0), None);
    }

    #[test]
    fn single_element() {
        let d = vec![7.0];
        assert_eq!(mean(&d), Some(7.0));
        assert_eq!(median(&d), Some(7.0));
        assert_eq!(variance(&d), Some(0.0));
        assert_eq!(percentile(&d, 99.0), Some(7.0));
    }
}
