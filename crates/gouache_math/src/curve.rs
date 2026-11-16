//! Centripetal Catmull-Rom splines with arc-length sampling (wall paths, rivers, roads).

use glam::Vec3;

/// A centripetal Catmull-Rom spline through control points.
#[derive(Clone, Debug)]
pub struct CatmullRom {
    pts: Vec<Vec3>,
    /// Cumulative arc length table: (t_global, length).
    table: Vec<(f32, f32)>,
}

impl CatmullRom {
    /// Build from at least two points. Endpoints are mirrored for the tangents.
    pub fn new(points: &[Vec3]) -> Option<Self> {
        if points.len() < 2 {
            return None;
        }
        let mut c = Self { pts: points.to_vec(), table: Vec::new() };
        let segs = c.segments();
        let n = segs * 32;
        let mut len = 0.0;
        let mut prev = c.eval(0.0);
        c.table.push((0.0, 0.0));
        for i in 1..=n {
            let t = i as f32 / n as f32 * segs as f32;
            let p = c.eval(t);
            len += p.distance(prev);
            prev = p;
            c.table.push((t, len));
        }
        Some(c)
    }

    /// Number of spline segments.
    pub fn segments(&self) -> usize {
        self.pts.len() - 1
    }

    /// Total arc length in meters.
    pub fn length(&self) -> f32 {
        self.table.last().map_or(0.0, |e| e.1)
    }

    fn ctrl(&self, i: isize) -> Vec3 {
        let n = self.pts.len() as isize;
        if i < 0 {
            2.0 * self.pts[0] - self.pts[1]
        } else if i >= n {
            2.0 * self.pts[(n - 1) as usize] - self.pts[(n - 2) as usize]
        } else {
            self.pts[i as usize]
        }
    }

    /// Evaluate at global parameter `t` in [0, segments].
    pub fn eval(&self, t: f32) -> Vec3 {
        let segs = self.segments();
        let t = t.clamp(0.0, segs as f32);
        let i = (t.floor() as usize).min(segs - 1);
        let u = t - i as f32;
        let i = i as isize;
        let (p0, p1, p2, p3) = (self.ctrl(i - 1), self.ctrl(i), self.ctrl(i + 1), self.ctrl(i + 2));
        // Centripetal parameterisation (alpha = 0.5) avoids cusps and self-intersections.
        let kt = |a: Vec3, b: Vec3| libm::powf(a.distance(b).max(1e-4), 0.5);
        let t0 = 0.0;
        let t1 = t0 + kt(p0, p1);
        let t2 = t1 + kt(p1, p2);
        let t3 = t2 + kt(p2, p3);
        let tt = t1 + (t2 - t1) * u;
        let a1 = p0 * ((t1 - tt) / (t1 - t0)) + p1 * ((tt - t0) / (t1 - t0));
        let a2 = p1 * ((t2 - tt) / (t2 - t1)) + p2 * ((tt - t1) / (t2 - t1));
        let a3 = p2 * ((t3 - tt) / (t3 - t2)) + p3 * ((tt - t2) / (t3 - t2));
        let b1 = a1 * ((t2 - tt) / (t2 - t0)) + a2 * ((tt - t0) / (t2 - t0));
        let b2 = a2 * ((t3 - tt) / (t3 - t1)) + a3 * ((tt - t1) / (t3 - t1));
        b1 * ((t2 - tt) / (t2 - t1)) + b2 * ((tt - t1) / (t2 - t1))
    }

    /// Global parameter at arc length `s` (meters).
    pub fn param_at_length(&self, s: f32) -> f32 {
        let s = s.clamp(0.0, self.length());
        let idx = self.table.partition_point(|e| e.1 < s).max(1).min(self.table.len() - 1);
        let (ta, la) = self.table[idx - 1];
        let (tb, lb) = self.table[idx];
        let f = if lb > la { (s - la) / (lb - la) } else { 0.0 };
        ta + (tb - ta) * f
    }

    /// Position and unit tangent at arc length `s`.
    pub fn frame_at_length(&self, s: f32) -> (Vec3, Vec3) {
        let t = self.param_at_length(s);
        let h = 1e-3;
        let a = self.eval(t - h);
        let b = self.eval(t + h);
        (self.eval(t), (b - a).normalize_or(Vec3::X))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_line_length() {
        let c = CatmullRom::new(&[Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0)]).unwrap();
        assert!((c.length() - 10.0).abs() < 1e-3);
        let (p, t) = c.frame_at_length(5.0);
        assert!((p.x - 5.0).abs() < 1e-2);
        assert!((t - Vec3::X).length() < 1e-3);
    }

    #[test]
    fn passes_through_control_points() {
        let pts = [Vec3::ZERO, Vec3::new(4.0, 0.0, 1.0), Vec3::new(8.0, 0.0, -2.0)];
        let c = CatmullRom::new(&pts).unwrap();
        assert!(c.eval(1.0).distance(pts[1]) < 1e-4);
        assert!(c.eval(2.0).distance(pts[2]) < 1e-4);
    }
}
