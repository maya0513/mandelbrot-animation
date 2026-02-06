use clap::Parser;
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn add(self, other: Complex) -> Complex {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    fn mul(self, other: Complex) -> Complex {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

#[derive(Parser, Debug)]
#[command(name = "mandelbrot-animation")]
#[command(about = "Render Mandelbrot animation frames.")]
struct Args {
    #[arg(long, default_value_t = 1920)]
    width: u32,
    #[arg(long, default_value_t = 1080)]
    height: u32,
    #[arg(long, default_value_t = 300)]
    frames: u32,
    #[arg(long, default_value_t = 30)]
    fps: u32,
    #[arg(long, default_value_t = 2000)]
    max_iter: u32,
    #[arg(long, default_value_t = 1.0)]
    zoom_start: f64,
    #[arg(long, default_value_t = 1e-6)]
    zoom_end: f64,
    #[arg(long, default_value = "out/frames")]
    out_dir: String,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let out_dir = PathBuf::from(&args.out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| format!("create out_dir: {e}"))?;

    let path = fixed_path();

    let total_frames = args.frames.max(1);
    for frame in 0..total_frames {
        let t = if total_frames <= 1 {
            0.0
        } else {
            frame as f64 / (total_frames - 1) as f64
        };
        let path_center = path_position(&path, t);
        let zoom = exp_lerp(args.zoom_start, args.zoom_end, t);
        let center = dampened_center(path[0], path_center, zoom, args.zoom_start);
        let img = render_frame(
            args.width,
            args.height,
            center,
            zoom,
            args.max_iter,
        );

        let filename = format!("frame_{:06}.png", frame);
        let filepath = out_dir.join(filename);
        img.save(&filepath)
            .map_err(|e| format!("save {filepath:?}: {e}"))?;
        println!(
            "frame {}/{} -> {}",
            frame + 1,
            total_frames,
            filepath.display()
        );
    }

    println!();
    println!("ffmpeg example:");
    println!(
        "ffmpeg -framerate {} -i {}/frame_%06d.png -c:v libx264 -pix_fmt yuv420p out/mandelbrot.mp4",
        args.fps,
        args.out_dir
    );

    Ok(())
}

fn render_frame(
    width: u32,
    height: u32,
    center: Complex,
    zoom: f64,
    max_iter: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let buf = img.as_mut();
    let w = width as usize;
    let h = height as usize;
    let half_min = (w.min(h) as f64) / 2.0;
    let scale = zoom / half_min;

    buf.par_chunks_mut(3)
        .enumerate()
        .for_each(|(idx, pixel)| {
            let x = (idx % w) as f64;
            let y = (idx / w) as f64;
            let cx = (x - (w as f64 / 2.0)) * scale + center.re;
            let cy = (y - (h as f64 / 2.0)) * scale + center.im;
            let c = Complex { re: cx, im: cy };
            let color = mandelbrot_color(c, max_iter);
            pixel[0] = color[0];
            pixel[1] = color[1];
            pixel[2] = color[2];
        });

    img
}

fn mandelbrot_color(c: Complex, max_iter: u32) -> [u8; 3] {
    let mut z = Complex { re: 0.0, im: 0.0 };
    let mut iter = 0;

    while iter < max_iter && z.norm_sqr() <= 4.0 {
        z = z.mul(z).add(c);
        iter += 1;
    }

    if iter >= max_iter {
        return [0, 0, 0];
    }

    let zn = z.norm_sqr().sqrt();
    let smooth = iter as f64 + 1.0 - (zn.ln().ln() / 2.0_f64.ln());
    let t = (smooth / max_iter as f64).clamp(0.0, 1.0);
    palette_color(t)
}

fn palette_color(t: f64) -> [u8; 3] {
    let hue = (360.0 * (0.65 + 2.2 * t)) % 360.0;
    let sat = 0.95;
    let val = (0.25 + 0.85 * t).clamp(0.0, 1.0);
    hsv_to_rgb(hue, sat, val)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let h = (h % 360.0 + 360.0) % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8,
    ]
}

fn exp_lerp(a: f64, b: f64, t: f64) -> f64 {
    if a <= 0.0 || b <= 0.0 {
        return a + (b - a) * t;
    }
    a * (b / a).powf(t)
}

fn dampened_center(base: Complex, target: Complex, zoom: f64, zoom_start: f64) -> Complex {
    let ratio = if zoom_start > 0.0 {
        (zoom / zoom_start).clamp(0.0, 1.0)
    } else {
        1.0
    };
    Complex {
        re: base.re + (target.re - base.re) * ratio,
        im: base.im + (target.im - base.im) * ratio,
    }
}

fn fixed_path() -> Vec<Complex> {
    vec![
        Complex {
            re: -0.743643887037151,
            im: 0.13182590420533,
        },
        Complex {
            re: -0.743643135,
            im: 0.13182733,
        },
        Complex {
            re: -0.743642,
            im: 0.131829,
        },
        Complex {
            re: -0.74364085,
            im: 0.1318309,
        },
    ]
}

fn path_position(points: &[Complex], t: f64) -> Complex {
    if points.len() <= 1 {
        return points[0];
    }
    let segments = points.len() - 1;
    let scaled = (t.clamp(0.0, 1.0) * segments as f64).min(segments as f64 - 1e-9);
    let seg_idx = scaled.floor() as usize;
    let seg_t = scaled - seg_idx as f64;
    let a = points[seg_idx];
    let b = points[seg_idx + 1];
    Complex {
        re: a.re + (b.re - a.re) * seg_t,
        im: a.im + (b.im - a.im) * seg_t,
    }
}
