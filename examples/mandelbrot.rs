extern crate clap;
extern crate num_cpus;
extern crate png;
extern crate wyvern;

use clap::{App, Arg};
use png::{BitDepth, ColorType, Encoder, HasParameters};
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Add, Div, Mul, Sub};
use std::path::PathBuf;
#[cfg(simd)]
use std::simd::f32x16;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use wyvern::core::builder::ProgramBuilder;
use wyvern::core::executor::{Executable, Executor, Resource, IO};
use wyvern::core::program::{ConstantVector, TokenValue};
use wyvern::core::types::{Array, Constant, Variable};
use wyvern::cpu::executor::CpuExecutor;
use wyvern::vk::executor::VkExecutor;

const WIDTH: u32 = 3840;
const HEIGHT: u32 = 2160;
const OUTFILE: &str = "out.png";
const CENTER_X: f32 = -0.75;
const CENTER_Y: f32 = 0.0;
const ZOOM: f32 = HEIGHT as f32 / 2.5;
const ITERATIONS: usize = 2000;

enum Mode {
    Native,
    Cpu,
    Vk,
    MtNative,
    MtSimdNative,
    SimdNative,
}

trait Number:
    Copy
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
{
}

impl Number for f32 {}
impl<'a> Number for Constant<'a, f32> {}

fn get_opts() -> (Mode, u32, u32, PathBuf, usize, usize) {
    let args = App::new("mandelbrot")
        .author("Dario Ostuni <dario.ostuni@studenti.unimi.it>")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .help("Compute engine")
                .possible_values(&["native", "cpu", "vulkan", "mt", "simd", "simd_mt"])
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("iterations")
                .short("i")
                .long("iterations")
                .help("Iterations per pixel")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cores")
                .short("c")
                .long("cores")
                .help("Number of cores")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .help("Width of the output")
                .requires("height")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .help("Height of the output")
                .requires("width")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Output file")
                .takes_value(true),
        )
        .get_matches();
    let mode = match args.value_of("mode").unwrap() {
        "native" => Mode::Native,
        "cpu" => Mode::Cpu,
        "vulkan" => Mode::Vk,
        "simd_mt" => Mode::MtSimdNative,
        "simd" => Mode::SimdNative,
        "mt" => Mode::MtNative,
        _ => unreachable!(),
    };
    let width = match args.value_of("width") {
        None => WIDTH,
        Some(s) => s.parse().unwrap_or(WIDTH),
    };
    let height = match args.value_of("height") {
        None => HEIGHT,
        Some(s) => s.parse().unwrap_or(HEIGHT),
    };
    let outfile = args.value_of("output").unwrap_or(OUTFILE);
    let iterations = match args.value_of("iterations") {
        None => ITERATIONS,
        Some(s) => s.parse().unwrap_or(ITERATIONS),
    };
    let cores = match args.value_of("cores") {
        None => num_cpus::get(),
        Some(s) => s.parse().unwrap_or(num_cpus::get()),
    };
    (mode, width, height, PathBuf::from(outfile), iterations, cores)
}

fn main() {
    let (mode, width, height, outfile_path, iterations, cores) = get_opts();
    let mut outfile = BufWriter::new(File::create(outfile_path).unwrap());
    let mut encoder = Encoder::new(&mut outfile, width, height);
    encoder.set(ColorType::Grayscale).set(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let (data, time) = &match mode {
        Mode::Native => native(width, height, 1, iterations),
        Mode::MtNative => native(width, height, cores, iterations),
        Mode::Cpu => cpu(width, height, iterations),
        Mode::Vk => vk(width, height, iterations),
        Mode::SimdNative => simd(width, height, 1, iterations),
        Mode::MtSimdNative => simd(width, height, cores, iterations),
    };
    let data = colorize(data);
    writer.write_image_data(&data).unwrap();
    println!("{}.{:09}", time.as_secs(), time.subsec_nanos());
}

fn colorize(data: &[f32]) -> Vec<u8> {
    data.iter()
        .map(|&x| if x <= 2.0 { 0 } else { 255 })
        .collect()
}

#[cfg(not(simd))]
fn simd(_: u32, _: u32, _: usize, _: usize) -> (Vec<f32>, Duration) {
    unreachable!("simd support not enabled!")
}

#[cfg(simd)]
fn simd(width: u32, height: u32, cores: usize, iterations: usize) -> (Vec<f32>, Duration) {
    let now = Instant::now();
    let width = width as usize;
    let height = height as usize;
    let half_width = f32x16::splat(width as f32 / 2.0);
    let half_height = f32x16::splat(height as f32 / 2.0);
    let vzoom = f32x16::splat(ZOOM);
    let vx0 = f32x16::splat(CENTER_X);
    let vy0 = f32x16::splat(CENTER_Y);
    let v2 = f32x16::splat(2.0);
    assert_eq!((width * height) % 16, 0);
    let mut out = vec![0.0; width * height];
    let next_id = Arc::new(AtomicUsize::new(0));
    let threads: Vec<_> = (0..cores)
        .map(|_| {
            let next_id = next_id.clone();
            thread::spawn(move || {
                let mut sol: Vec<(usize, Vec<f32>)> = Vec::new();
                loop {
                    let id = next_id.fetch_add(16, Ordering::Relaxed);
                    if id >= width * height {
                        break;
                    }
                    let lx: Vec<_> = (id..(id + 16)).map(|a| (a % width) as f32).collect();
                    let ly: Vec<_> = (id..(id + 16)).map(|a| (a / width) as f32).collect();
                    let mut x = f32x16::load_unaligned(&lx);
                    let mut y = f32x16::load_unaligned(&ly);
                    x -= half_width;
                    y -= half_height;
                    x /= vzoom;
                    y /= vzoom;
                    x += vx0;
                    y += vy0;
                    let mut a = f32x16::splat(0.0);
                    let mut b = f32x16::splat(0.0);
                    for _ in 0..iterations {
                        let tmp_a = a * a - b * b + x;
                        b = v2 * a * b + y;
                        a = tmp_a;
                    }
                    let out = a * a + b * b;
                    let mut outv = vec![0.0; 16];
                    out.store_unaligned(&mut outv);
                    sol.push((id, outv));
                }
                sol
            })
        })
        .collect();
    for t in threads {
        let r = t.join().unwrap();
        for s in r {
            for i in (s.0)..(s.0 + 16) {
                out[i] = s.1[i - s.0];
            }
        }
    }
    (out, now.elapsed())
}

fn native(width: u32, height: u32, cores: usize, iterations: usize) -> (Vec<f32>, Duration) {
    let now = Instant::now();
    let width = width as usize;
    let height = height as usize;
    let mut v = vec![0.0; width * height];
    let next_id = Arc::new(AtomicUsize::new(0));
    let threads: Vec<_> = (0..cores)
        .map(|_| {
            let next_id = next_id.clone();
            thread::spawn(move || {
                let mut sol: Vec<(usize, f32)> = Vec::new();
                loop {
                    let id = next_id.fetch_add(1, Ordering::Relaxed);
                    if id >= width * height {
                        break;
                    }
                    let (x, y) = pixel2coordinates(
                        (id % width) as f32,
                        (id / width) as f32,
                        CENTER_X,
                        CENTER_Y,
                        width as f32,
                        height as f32,
                        ZOOM,
                        2.0,
                    );
                    sol.push((id, mandelbrot(x, y, 0.0, 2.0, iterations)));
                }
                sol
            })
        })
        .collect();
    for t in threads {
        let r = t.join().unwrap();
        for s in r {
            v[s.0] = s.1;
        }
    }
    (v, now.elapsed())
}

fn cpu(width: u32, height: u32, iterations: usize) -> (Vec<f32>, Duration) {
    let now = Instant::now();
    let builder = ProgramBuilder::new();
    wyvern_program(&builder, iterations);
    let program = builder.finalize().unwrap();
    let executor = CpuExecutor::new(Default::default()).unwrap();
    let mut executable = executor.compile(program).unwrap();
    let input = executor.new_resource().unwrap();
    let output = executor.new_resource().unwrap();
    input.set_data(TokenValue::Vector(ConstantVector::U32(vec![width, height])));
    output.set_data(TokenValue::Vector(ConstantVector::F32(vec![
        0.0;
        (width * height)
            as usize
    ])));
    executable.bind("input", IO::Input, input.clone());
    executable.bind("output", IO::Output, output.clone());
    executable.run().unwrap();
    (
        match output.get_data() {
            TokenValue::Vector(ConstantVector::F32(x)) => x,
            _ => unreachable!(),
        },
        now.elapsed(),
    )
}

fn vk(width: u32, height: u32, iterations: usize) -> (Vec<f32>, Duration) {
    let now = Instant::now();
    let builder = ProgramBuilder::new();
    wyvern_program(&builder, iterations);
    let program = builder.finalize().unwrap();
    let executor = VkExecutor::new(Default::default()).unwrap();
    let mut executable = executor.compile(program).unwrap();
    let input = executor.new_resource().unwrap();
    let output = executor.new_resource().unwrap();
    input.set_data(TokenValue::Vector(ConstantVector::U32(vec![width, height])));
    output.set_data(TokenValue::Vector(ConstantVector::F32(vec![
        0.0;
        (width * height)
            as usize
    ])));
    executable.bind("input", IO::Input, input.clone());
    executable.bind("output", IO::Output, output.clone());
    executable.run().unwrap();
    (
        match output.get_data() {
            TokenValue::Vector(ConstantVector::F32(x)) => x,
            _ => unreachable!(),
        },
        now.elapsed(),
    )
}

fn wyvern_program(b: &ProgramBuilder, iterations: usize) {
    let iterations = Constant::new(iterations as u32, b);
    let zero = Constant::new(0_u32, b);
    let fzero = Constant::new(0_f32, b);
    let one = Constant::new(1_u32, b);
    let ftwo = Constant::new(2_f32, b);
    let input = Array::new(zero, 0, true, b).mark_as_input("input");
    let output = Array::new(zero, 0, true, b).mark_as_output("output");
    let width = input.at(zero).load();
    let height = input.at(one).load();
    let fwidth = Constant::from(width);
    let fheight = Constant::from(height);
    let center_x = Constant::new(CENTER_X, b);
    let center_y = Constant::new(CENTER_Y, b);
    let zoom = Constant::new(ZOOM, b);
    let cells = width * height;
    let id = b.worker_id();
    let size = b.num_workers();
    let cell = Variable::new(b);
    cell.store(id);
    b.while_loop(
        |_| cell.load().lt(cells),
        |_| {
            let cell_id = cell.load();
            let local_y = Constant::from(cell_id / width);
            let local_x = Constant::from(cell_id % width);
            let (local_x, local_y) = pixel2coordinates(
                local_x, local_y, center_x, center_y, fwidth, fheight, zoom, ftwo,
            );
            let value = mandelbrot_wy(local_x, local_y, fzero, ftwo, iterations);
            output.at(cell_id).store(value);
            cell.store(cell_id + size);
        },
    );
}

fn pixel2coordinates<T: Number>(
    mut x: T,
    mut y: T,
    x0: T,
    y0: T,
    width: T,
    height: T,
    zoom: T,
    two: T,
) -> (T, T) {
    x = x - width / two;
    y = y - height / two;
    x = x / zoom;
    y = y / zoom;
    (x + x0, y + y0)
}

fn mandelbrot<T: Number>(a0: T, b0: T, zero: T, two: T, iterations: usize) -> T {
    let mut a = zero;
    let mut b = zero;
    for _ in 0..iterations {
        let tmp_a = a * a - b * b + a0;
        b = two * a * b + b0;
        a = tmp_a;
    }
    a * a + b * b
}

fn mandelbrot_wy<'a>(
    a0: Constant<'a, f32>,
    b0: Constant<'a, f32>,
    zero: Constant<'a, f32>,
    two: Constant<'a, f32>,
    iterations: Constant<'a, u32>,
) -> Constant<'a, f32> {
    let builder = a0.info.builder;
    let a_storage = Variable::new(builder);
    a_storage.store(zero);
    let b_storage = Variable::new(builder);
    b_storage.store(zero);
    let i = Variable::new(builder);
    i.store(Constant::new(0, builder));
    builder.while_loop(
        |_| i.load().lt(iterations),
        |_| {
            let a = a_storage.load();
            let b = b_storage.load();
            let tmp_a = a * a - b * b + a0;
            b_storage.store(two * a * b + b0);
            a_storage.store(tmp_a);
            i.store(i.load() + 1);
        },
    );
    let a = a_storage.load();
    let b = b_storage.load();
    a * a + b * b
}
