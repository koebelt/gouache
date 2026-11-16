//! Deterministic gradient noise and fBm (integer hash lattice, no platform dependence).

use gouache_math::hash3;

fn grad(ix: i32, iz: i32, seed: u32) -> (f32, f32) {
    let h = hash3(ix, iz, seed as i32);
    // 16 evenly spaced unit gradients (exact table avoids libm differences).
    const S: f32 = std::f32::consts::FRAC_1_SQRT_2;
    const G: [(f32, f32); 16] = [
        (1.0, 0.0),
        (0.92388, 0.38268),
        (S, S),
        (0.38268, 0.92388),
        (0.0, 1.0),
        (-0.38268, 0.92388),
        (-S, S),
        (-0.92388, 0.38268),
        (-1.0, 0.0),
        (-0.92388, -0.38268),
        (-S, -S),
        (-0.38268, -0.92388),
        (0.0, -1.0),
        (0.38268, -0.92388),
        (S, -S),
        (0.92388, -0.38268),
    ];
    G[(h & 15) as usize]
}

#[inline]
fn quintic(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// 2D gradient noise in roughly [-0.7, 0.7].
pub fn gradient2(x: f32, z: f32, seed: u32) -> f32 {
    let (fx, fz) = (x.floor(), z.floor());
    let (ix, iz) = (fx as i32, fz as i32);
    let (tx, tz) = (x - fx, z - fz);
    let dot = |dx: i32, dz: i32| {
        let g = grad(ix + dx, iz + dz, seed);
        g.0 * (tx - dx as f32) + g.1 * (tz - dz as f32)
    };
    let (u, v) = (quintic(tx), quintic(tz));
    let a = dot(0, 0) + (dot(1, 0) - dot(0, 0)) * u;
    let b = dot(0, 1) + (dot(1, 1) - dot(0, 1)) * u;
    a + (b - a) * v
}

/// Fractal Brownian motion of [`gradient2`].
pub fn fbm2(x: f32, z: f32, seed: u32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let (mut f, mut a, mut sum, mut norm) = (1.0, 1.0, 0.0, 0.0);
    for o in 0..octaves {
        // Rotate each octave slightly to hide lattice alignment.
        let (rx, rz) = (x * f + z * f * 0.13, z * f - x * f * 0.13);
        sum += a * gradient2(rx + o as f32 * 17.3, rz - o as f32 * 9.1, seed.wrapping_add(o));
        norm += a;
        f *= lacunarity;
        a *= gain;
    }
    sum / norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_bounded() {
        let a = fbm2(3.3, -7.1, 9, 5, 2.0, 0.5);
        assert_eq!(a, fbm2(3.3, -7.1, 9, 5, 2.0, 0.5));
        for i in 0..1000 {
            let v = gradient2(i as f32 * 0.37, i as f32 * -0.21, 1);
            assert!(v.abs() <= 1.0);
        }
    }

    #[test]
    fn zero_on_lattice() {
        assert_eq!(gradient2(3.0, 4.0, 7), 0.0);
    }
}
