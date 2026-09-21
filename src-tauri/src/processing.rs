//! Port of the processing pipeline in `film_converter_v2.py`.
//!
//! The original does three things, in order:
//!
//! ```python
//! img = 255 - img                                     # invert_image
//! low  = np.percentile(channel, 0.5)                  # stretch_channel
//! high = np.percentile(channel, 99.5)
//! (channel - low) * (255.0 / (high - low))            # clipped to 0..255
//! ```
//!
//! Percentiles are taken per channel *after* inversion, so each of R, G and B
//! gets its own black and white point. That is what removes the orange mask of
//! a colour negative.

use image::RgbImage;

/// Percentile of the darkest pixels that gets clipped to pure black.
pub const LOW_PERCENTILE: f64 = 0.5;
/// Percentile above which pixels get clipped to pure white.
pub const HIGH_PERCENTILE: f64 = 99.5;

/// Per-channel clipping points chosen for one image, in inverted-image space.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct ChannelLevels {
    pub low: f64,
    pub high: f64,
}

/// The black/white points picked for each of the three channels.
pub type Levels = [ChannelLevels; 3];

/// Inverts `img` in place. Equivalent to numpy's `255 - img`.
fn invert(img: &mut RgbImage) {
    for byte in img.iter_mut() {
        *byte = 255 - *byte;
    }
}

/// Counts how often each 0..=255 value occurs, separately per channel.
fn histograms(img: &RgbImage) -> [[u64; 256]; 3] {
    let mut hists = [[0u64; 256]; 3];
    // `as_chunks` yields fixed-size arrays, so the indexing below needs no
    // bounds checks.
    for pixel in img.as_chunks::<3>().0 {
        hists[0][pixel[0] as usize] += 1;
        hists[1][pixel[1] as usize] += 1;
        hists[2][pixel[2] as usize] += 1;
    }
    hists
}

/// numpy's `_lerp`: interpolate from `a` below the midpoint and from `b` above
/// it. The two forms are algebraically equal but round differently, and numpy
/// picks the one that keeps the result inside `[a, b]`.
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    let difference = b - a;
    if t >= 0.5 {
        b - difference * (1.0 - t)
    } else {
        a + difference * t
    }
}

/// `np.percentile(channel, q)` for `u8` data, using numpy's default "linear"
/// interpolation method.
///
/// numpy sorts the samples and looks up the (possibly fractional) position
/// `(q / 100) * (n - 1)`, interpolating between its two neighbours. A
/// cumulative histogram gives us the sorted value at any rank without ever
/// materialising the sorted array.
///
/// The arithmetic here is deliberately written to match numpy bit for bit.
/// Both the association in the virtual index and the two-sided interpolation
/// in [`lerp`] change the last bit of the result, which is enough to shift a
/// clipping point by one and alter the developed pixels.
fn percentile(hist: &[u64; 256], count: u64, q: f64) -> f64 {
    if count == 0 {
        return 0.0;
    }

    // numpy scales to a quantile first, then multiplies: `(q / 100) * (n - 1)`.
    let virtual_index = (q / 100.0) * (count - 1) as f64;
    let lower_rank = virtual_index.floor() as u64;
    let fraction = virtual_index - lower_rank as f64;

    // Walk the cumulative histogram once, picking up the values at `lower_rank`
    // and `lower_rank + 1` as we pass them.
    let mut cumulative = 0u64;
    let mut lower_value: Option<u8> = None;
    for (value, &occurrences) in hist.iter().enumerate() {
        cumulative += occurrences;
        if lower_value.is_none() && cumulative > lower_rank {
            lower_value = Some(value as u8);
            // An exact rank needs no neighbour, and neither does a bucket wide
            // enough to contain both ranks.
            if fraction == 0.0 || cumulative > lower_rank + 1 {
                return value as f64;
            }
        } else if lower_value.is_some() && occurrences > 0 {
            return lerp(lower_value.unwrap() as f64, value as f64, fraction);
        }
    }

    // Only reachable if `lower_rank` is the very last sample.
    lower_value.map_or(0.0, f64::from)
}

/// Builds the 256-entry lookup table that `stretch_channel` amounts to.
///
/// Returns `None` when the channel is too flat to stretch, mirroring the
/// Python's `if high - low < 1: return channel`.
fn stretch_lut(low: f64, high: f64) -> Option<[u8; 256]> {
    if high - low < 1.0 {
        return None;
    }

    let scale = 255.0 / (high - low);
    let mut lut = [0u8; 256];
    for (value, entry) in lut.iter_mut().enumerate() {
        let stretched = (value as f64 - low) * scale;
        // `np.clip(...).astype(np.uint8)` truncates rather than rounds, so
        // `as u8` on a clamped f64 reproduces it exactly.
        *entry = stretched.clamp(0.0, 255.0) as u8;
    }
    Some(lut)
}

/// Picks black and white points for each channel of an already-inverted image.
fn measure_levels(img: &RgbImage) -> Levels {
    let hists = histograms(img);
    let count = (img.width() as u64) * (img.height() as u64);

    let mut levels = Levels::default();
    for (channel, hist) in hists.iter().enumerate() {
        levels[channel] = ChannelLevels {
            low: percentile(hist, count, LOW_PERCENTILE),
            high: percentile(hist, count, HIGH_PERCENTILE),
        };
    }
    levels
}

/// Applies previously measured levels to an inverted image, in place.
fn apply_levels(img: &mut RgbImage, levels: &Levels) {
    // A channel too flat to stretch keeps its values, like the Python's early
    // `return channel`. Expressing that as an identity table keeps the inner
    // loop branch-free.
    let identity: [u8; 256] = std::array::from_fn(|value| value as u8);
    let luts: [[u8; 256]; 3] =
        std::array::from_fn(|c| stretch_lut(levels[c].low, levels[c].high).unwrap_or(identity));

    for pixel in img.as_chunks_mut::<3>().0 {
        pixel[0] = luts[0][pixel[0] as usize];
        pixel[1] = luts[1][pixel[1] as usize];
        pixel[2] = luts[2][pixel[2] as usize];
    }
}

/// Inverts `img` and measures the clipping points, without stretching yet.
///
/// Splitting the pipeline here lets a preview reuse the levels measured from
/// the full-resolution scan, so what the user sees matches what gets written.
pub fn invert_and_measure(img: &mut RgbImage) -> Levels {
    invert(img);
    measure_levels(img)
}

/// The full `process_image` pipeline: invert, then stretch every channel.
pub fn develop(img: &mut RgbImage) -> Levels {
    let levels = invert_and_measure(img);
    apply_levels(img, &levels);
    levels
}

/// Inverts and stretches using levels measured elsewhere — used for previews,
/// where the levels come from the full-resolution image but the pixels being
/// mapped are a downscaled copy.
pub fn develop_with_levels(img: &mut RgbImage, levels: &Levels) {
    invert(img);
    apply_levels(img, levels);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference implementation: sort the samples and interpolate, exactly the
    /// way numpy's default method is documented to behave.
    fn naive_percentile(samples: &[u8], q: f64) -> f64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();

        let virtual_index = (q / 100.0) * (sorted.len() - 1) as f64;
        let lower = virtual_index.floor() as usize;
        let fraction = virtual_index - lower as f64;
        if fraction == 0.0 {
            return sorted[lower] as f64;
        }
        lerp(sorted[lower] as f64, sorted[lower + 1] as f64, fraction)
    }

    fn histogram_of(samples: &[u8]) -> [u64; 256] {
        let mut hist = [0u64; 256];
        for &s in samples {
            hist[s as usize] += 1;
        }
        hist
    }

    fn check(samples: &[u8], q: f64) {
        let hist = histogram_of(samples);
        let got = percentile(&hist, samples.len() as u64, q);
        let want = naive_percentile(samples, q);
        assert!(
            (got - want).abs() < 1e-9,
            "percentile({q}) over {samples:?}: got {got}, want {want}"
        );
    }

    #[test]
    fn percentile_matches_numpy_on_small_samples() {
        let cases: &[&[u8]] = &[
            &[0],
            &[7, 7, 7, 7],
            &[0, 255],
            &[1, 2, 3, 4],
            &[1, 2, 3, 4, 5],
            &[0, 0, 0, 10, 200, 255],
            &[9, 3, 3, 250, 17, 17, 17, 40],
        ];
        for samples in cases {
            for q in [0.0, 0.5, 25.0, 50.0, 99.5, 100.0] {
                check(samples, q);
            }
        }
    }

    #[test]
    fn percentile_matches_numpy_on_a_spread_sample() {
        // Deterministic pseudo-random spread, wide enough to exercise the
        // cumulative-histogram walk across many buckets.
        let samples: Vec<u8> = (0..1000u32)
            .map(|i| ((i * 7919 + 13) % 256) as u8)
            .collect();
        for q in [0.0, 0.5, 1.0, 33.3, 50.0, 99.5, 100.0] {
            check(&samples, q);
        }
    }

    #[test]
    fn percentile_handles_a_single_dominant_bucket() {
        // 990 zeroes then 10 high values: the 0.5 and 99.5 percentiles land on
        // opposite sides of a very lopsided distribution.
        let mut samples = vec![0u8; 990];
        samples.extend_from_slice(&[200, 201, 202, 203, 204, 205, 206, 207, 208, 209]);
        for q in [0.0, 0.5, 50.0, 99.0, 99.5, 99.9, 100.0] {
            check(&samples, q);
        }
    }

    /// Both of these lock in arithmetic that a "simplification" would break.
    /// The clipping point moves by a fraction of a level, which is enough to
    /// shift `astype(uint8)` truncation and change the developed pixels.
    #[test]
    fn virtual_index_keeps_numpys_association() {
        // n = 2275, q = 0.5. numpy computes `(q / 100) * (n - 1)`, giving a
        // gamma of 0.370000000000001; `(n - 1) * q / 100` gives
        // 0.3699999999999992, and the interpolated result differs.
        let mut hist = [0u64; 256];
        hist[96] = 12; // ranks 0..=11
        hist[98] = 1; //  rank 12
        hist[200] = 2275 - 13;

        let got = percentile(&hist, 2275, 0.5);
        assert_eq!(
            got, 96.74000000000001,
            "percentile must match np.percentile bit for bit"
        );
        assert_ne!(got, 96.73999999999998, "that is the wrong association");
    }

    #[test]
    fn lerp_switches_formula_above_the_midpoint() {
        // numpy interpolates from `b` once t >= 0.5. For these values the two
        // algebraically-equal forms round differently.
        let t = 0.9950000000000001;
        assert_eq!(lerp(7.0, 38.0, t), 37.845000000000006);
        assert_ne!(lerp(7.0, 38.0, t), 37.845, "must not interpolate from `a`");

        // Below the midpoint it interpolates from `a`.
        assert_eq!(lerp(0.0, 10.0, 0.25), 2.5);
    }

    #[test]
    fn flat_channel_is_left_alone() {
        assert!(stretch_lut(10.0, 10.5).is_none());
        assert!(stretch_lut(10.0, 11.0).is_some());
    }

    #[test]
    fn stretch_maps_endpoints_to_full_range() {
        let lut = stretch_lut(20.0, 220.0).expect("wide enough to stretch");
        assert_eq!(lut[0], 0, "below the black point clips to 0");
        assert_eq!(lut[20], 0, "the black point becomes 0");
        // 200 * (255/200) lands a hair under 255 in f64 and truncates to 254 --
        // numpy's `astype(np.uint8)` does exactly the same.
        assert_eq!(lut[220], 254, "the white point reaches the top of the range");
        assert_eq!(lut[255], 255, "above the white point clips to 255");
        assert!(lut[120] > 0 && lut[120] < 255, "midtones stay in range");
    }

    #[test]
    fn stretch_truncates_like_astype_uint8() {
        // (v - 0) * 255/254 for v = 1 is 1.00393…, which numpy truncates to 1.
        let lut = stretch_lut(0.0, 254.0).expect("wide enough to stretch");
        assert_eq!(lut[1], 1);
        // v = 128 gives 128.503…, which truncates to 128 rather than rounding to 129.
        assert_eq!(lut[128], 128);
    }

    #[test]
    fn develop_inverts_and_expands_contrast() {
        // A low-contrast "negative": values bunched between 100 and 150.
        let mut img = RgbImage::from_fn(10, 10, |x, _| {
            let v = 100 + (x as u8) * 5;
            image::Rgb([v, v, v])
        });

        let levels = develop(&mut img);
        assert!(levels[0].high > levels[0].low);

        let values: Vec<u8> = img.pixels().map(|p| p.0[0]).collect();
        let min = *values.iter().min().unwrap();
        let max = *values.iter().max().unwrap();
        assert_eq!(min, 0, "darkest pixel reaches black");
        assert_eq!(max, 255, "brightest pixel reaches white");

        // Inversion means the originally-brightest column is now the darkest.
        let first_column = img.get_pixel(0, 0).0[0];
        let last_column = img.get_pixel(9, 0).0[0];
        assert!(first_column > last_column, "the image was inverted");
    }

    #[test]
    fn preview_levels_match_a_full_develop() {
        let mut full = RgbImage::from_fn(16, 16, |x, y| {
            image::Rgb([(x * 11) as u8, (y * 13) as u8, ((x + y) * 7) as u8])
        });
        let mut copy = full.clone();

        let levels = develop(&mut full);
        develop_with_levels(&mut copy, &levels);

        assert_eq!(full.as_raw(), copy.as_raw());
    }
}
