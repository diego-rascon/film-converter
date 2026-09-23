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
//!
//! The port never builds the inverted image. Inversion maps a value `v` to
//! `255 - v`, so the inverted image's histogram is the scan's own read
//! backwards, and its percentiles come straight off that. The stretch is a
//! table from 256 values to 256 values, so the inversion folds into the same
//! table. That leaves one read of the scan to measure it and one pass through
//! a table to develop it — and every pixel comes out exactly as the three
//! steps above would leave it, which `develop_matches_the_three_step_pipeline`
//! checks byte for byte.

use image::RgbImage;
use rayon::prelude::*;

/// Percentile of the darkest pixels that gets clipped to pure black.
pub const LOW_PERCENTILE: f64 = 0.5;
/// Percentile above which pixels get clipped to pure white.
pub const HIGH_PERCENTILE: f64 = 99.5;

/// Pixels per piece when measuring is split across the pool: hundreds of
/// pieces for a full scan, and never so small that merging the per-piece
/// histograms costs more than counting them did.
const CHUNK_PIXELS: usize = 1 << 16;

/// Per-channel clipping points chosen for one image, in inverted-image space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChannelLevels {
    pub low: f64,
    pub high: f64,
}

/// The black/white points picked for each of the three channels.
pub type Levels = [ChannelLevels; 3];

/// How often each 0..=255 value occurs, separately per channel.
type Histograms = [[u64; 256]; 3];

/// Counts the scan's values per channel.
///
/// A preview measures one scan from outside the pool, so the count is split
/// across every core. A run already develops a scan on each of the pool's
/// workers, so there it stays on the worker's own thread: splitting would
/// buy nothing on busy cores, and a worker left waiting on its pieces would
/// pick up a whole other scan meanwhile and hold both in memory.
///
/// Split, the counts live on the heap. Rayon carries each piece's
/// accumulator down its recursion, and a waiting worker runs stolen pieces of
/// other previews on top of its own stack: 6 KB of counts per level, nested
/// that way, overflowed it.
fn histograms(img: &RgbImage) -> Histograms {
    if rayon::current_thread_index().is_some() {
        let mut hists = [[0; 256]; 3];
        count(&mut hists, img);
        return hists;
    }

    let empty = || Box::new([[0; 256]; 3]);
    let total = img
        .par_chunks(3 * CHUNK_PIXELS)
        .fold(empty, |mut hists: Box<Histograms>, chunk| {
            count(&mut hists, chunk);
            hists
        })
        .reduce(empty, |mut total, part| {
            for (total, part) in total.iter_mut().zip(part.iter()) {
                for (sum, count) in total.iter_mut().zip(part) {
                    *sum += count;
                }
            }
            total
        });
    *total
}

/// Adds `pixels`, whole RGB triples, to `hists`.
fn count(hists: &mut Histograms, pixels: &[u8]) {
    // `as_chunks` yields fixed-size arrays, so the indexing below needs no
    // bounds checks.
    for pixel in pixels.as_chunks::<3>().0 {
        hists[0][usize::from(pixel[0])] += 1;
        hists[1][usize::from(pixel[1])] += 1;
        hists[2][usize::from(pixel[2])] += 1;
    }
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
    let mut lower_value = None;
    for (value, &occurrences) in hist.iter().enumerate() {
        cumulative += occurrences;
        match lower_value {
            None if cumulative > lower_rank => {
                // An exact rank needs no neighbour, and neither does a bucket
                // wide enough to contain both ranks.
                if fraction == 0.0 || cumulative > lower_rank + 1 {
                    return value as f64;
                }
                lower_value = Some(value as f64);
            }
            Some(lower) if occurrences > 0 => return lerp(lower, value as f64, fraction),
            _ => {}
        }
    }

    // Only reachable if `lower_rank` is the very last sample.
    lower_value.unwrap_or(0.0)
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
    Some(std::array::from_fn(|value| {
        let stretched = (value as f64 - low) * scale;
        // `np.clip(...).astype(np.uint8)` truncates rather than rounds, so
        // `as u8` on a clamped f64 reproduces it exactly.
        stretched.clamp(0.0, 255.0) as u8
    }))
}

/// Picks each channel's black and white point: the 0.5th and 99.5th
/// percentiles of the *inverted* scan, read off the scan's own histogram.
pub fn measure(img: &RgbImage) -> Levels {
    let count = u64::from(img.width()) * u64::from(img.height());
    histograms(img).map(|mut hist| {
        // A count of value `v` in the scan is a count of `255 - v` in its
        // inversion, so the inversion's histogram is this one reversed.
        hist.reverse();
        ChannelLevels {
            low: percentile(&hist, count, LOW_PERCENTILE),
            high: percentile(&hist, count, HIGH_PERCENTILE),
        }
    })
}

/// The table one channel develops through: each scan value's inversion,
/// stretched between the channel's levels. A channel too flat to stretch is
/// only inverted, like the Python's early `return channel` after `255 - img`.
fn develop_table(levels: ChannelLevels) -> [u8; 256] {
    match stretch_lut(levels.low, levels.high) {
        Some(stretch) => std::array::from_fn(|value| stretch[255 - value]),
        None => std::array::from_fn(|value| (255 - value) as u8),
    }
}

/// Develops `img` in place with levels measured elsewhere — used for
/// previews, where the levels come from the full-resolution scan but the
/// pixels being mapped are a downscaled copy.
///
/// One thread is enough: a preview's copy is small, and a run already keeps
/// every core busy with a scan each.
pub fn develop_with_levels(img: &mut RgbImage, levels: &Levels) {
    let tables = levels.map(develop_table);
    for pixel in img.as_chunks_mut::<3>().0 {
        pixel[0] = tables[0][usize::from(pixel[0])];
        pixel[1] = tables[1][usize::from(pixel[1])];
        pixel[2] = tables[2][usize::from(pixel[2])];
    }
}

/// The full `process_image` pipeline: invert, then stretch every channel.
pub fn develop(img: &mut RgbImage) {
    let levels = measure(img);
    develop_with_levels(img, &levels);
}

#[cfg(test)]
// The pinned values are written the way numpy prints them, so they can be
// checked against its output by eye.
#[allow(clippy::unreadable_literal)]
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
            return f64::from(sorted[lower]);
        }
        lerp(
            f64::from(sorted[lower]),
            f64::from(sorted[lower + 1]),
            fraction,
        )
    }

    /// The Python, step by step: build the inverted image, take each
    /// channel's percentiles off its sorted samples, stretch with numpy's
    /// formula. Nothing is folded together or read backwards.
    fn three_step_pipeline(img: &RgbImage) -> RgbImage {
        let mut inverted = img.clone();
        for byte in inverted.iter_mut() {
            *byte = 255 - *byte;
        }

        let mut developed = inverted.clone();
        for channel in 0..3 {
            let samples: Vec<u8> = inverted.pixels().map(|p| p.0[channel]).collect();
            let low = naive_percentile(&samples, LOW_PERCENTILE);
            let high = naive_percentile(&samples, HIGH_PERCENTILE);
            if high - low < 1.0 {
                continue;
            }
            for (pixel, &value) in developed.pixels_mut().zip(&samples) {
                let stretched = (f64::from(value) - low) * (255.0 / (high - low));
                pixel.0[channel] = stretched.clamp(0.0, 255.0) as u8;
            }
        }
        developed
    }

    /// Deterministic noise, so a failure reproduces.
    fn noise(width: u32, height: u32, seed: u64) -> RgbImage {
        let mut state = seed | 1;
        RgbImage::from_fn(width, height, |_, _| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let [r, g, b, ..] = state.to_le_bytes();
            image::Rgb([r, g, b])
        })
    }

    fn histogram_of(samples: &[u8]) -> [u64; 256] {
        let mut hist = [0u64; 256];
        for &s in samples {
            hist[usize::from(s)] += 1;
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
        assert_eq!(
            lut[220], 254,
            "the white point reaches the top of the range"
        );
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
    fn develop_matches_the_three_step_pipeline() {
        let low_contrast = RgbImage::from_fn(40, 30, |x, y| {
            image::Rgb([
                150 + (x % 40) as u8,
                110 + (y % 30) as u8,
                70 + ((x + y) % 20) as u8,
            ])
        });
        // Blue is a single value, so it takes the too-flat branch and is only
        // inverted while red and green are stretched.
        let one_flat_channel = RgbImage::from_fn(33, 17, |x, y| {
            image::Rgb([(x * 7) as u8, (y * 15) as u8, 77])
        });
        // More pixels than one parallel chunk holds, so the histograms are
        // counted in pieces and merged.
        let spans_chunks = noise(300, 300, 0x2545_f491_4f6c_dd1d);

        for (name, scan) in [
            ("low contrast", low_contrast),
            ("one flat channel", one_flat_channel),
            ("spans chunks", spans_chunks),
            ("all black", RgbImage::new(8, 8)),
        ] {
            let mut developed = scan.clone();
            develop(&mut developed);
            assert_eq!(
                developed.as_raw(),
                three_step_pipeline(&scan).as_raw(),
                "{name}: the single pass must reproduce the Python's three steps"
            );
        }
    }

    #[test]
    fn measuring_matches_the_inverted_images_percentiles() {
        let scan = noise(64, 48, 7);
        let levels = measure(&scan);
        for (channel, level) in levels.iter().enumerate() {
            let inverted: Vec<u8> = scan.pixels().map(|p| 255 - p.0[channel]).collect();
            assert_eq!(level.low, naive_percentile(&inverted, LOW_PERCENTILE));
            assert_eq!(level.high, naive_percentile(&inverted, HIGH_PERCENTILE));
        }
    }

    #[test]
    fn a_run_measures_what_a_preview_measures() {
        // A preview measures from outside the pool, split across it; a run
        // measures on one of the pool's workers, in one piece. `join` from
        // outside the pool runs its closure on a worker.
        let scan = noise(300, 300, 11);
        let (on_a_worker, ()) = rayon::join(|| measure(&scan), || ());
        assert_eq!(on_a_worker, measure(&scan));
    }

    #[test]
    fn measuring_many_scans_at_once_stays_off_the_workers_stacks() {
        // Previews measure several scans at once on the shared rayon pool, so
        // a worker waiting on one piece steals pieces of another and nests
        // them on its own stack. Histograms carried by value through that
        // recursion overflowed it; this is the load that found it.
        let scan = noise(2048, 2048, 3);
        let expected = measure(&scan);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    for _ in 0..4 {
                        assert_eq!(measure(&scan), expected);
                    }
                });
            }
        });
    }

    #[test]
    fn develop_inverts_and_expands_contrast() {
        // A low-contrast "negative": values bunched between 100 and 150.
        let mut img = RgbImage::from_fn(10, 10, |x, _| {
            let v = 100 + (x as u8) * 5;
            image::Rgb([v, v, v])
        });

        let levels = measure(&img);
        assert!(levels[0].high > levels[0].low);
        develop(&mut img);

        let values: Vec<u8> = img.pixels().map(|p| p.0[0]).collect();
        assert_eq!(values.iter().min(), Some(&0), "darkest pixel reaches black");
        assert_eq!(
            values.iter().max(),
            Some(&255),
            "brightest pixel reaches white"
        );

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

        let levels = measure(&full);
        develop(&mut full);
        develop_with_levels(&mut copy, &levels);

        assert_eq!(full.as_raw(), copy.as_raw());
    }
}
