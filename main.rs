use std::{fs, io};
use std::fmt::format;
use std::fs::*;
use std::io::*;
use rand::Rng;
use raylib::prelude::*;

#[derive(Clone)]
struct Point{
    r:f64,
    g:f64,
    b:f64,
    x:i32,
    y:i32,
}

fn main() {
    let width: i32 = 1200;
    let height: i32 = 800;
    let (mut rl, thread) = raylib::init()
        .size(width.clone(), height.clone())
        .title("Hello, World")
        .build();

    let k = 16;
    let mut rand_colors: Vec<Color> = Vec::new();
    for i in 0..k{
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..255);
        let g = rng.gen_range(0..255);
        let b = rng.gen_range(0..255);
        rand_colors.push(Color::new(r, g, b, 255));
    }
    let iter_count = 10;
    let mut points: Vec<Point> = Vec::new();

    let file_name = "doggo.png";
    let mut img: Image = Image::load_image(file_name).expect("Failed to load image!");

    let pixel_data = img.get_image_data();
    for x_pos in 0..img.width() {
        for y_pos in 0..img.height() {
            let pixel = pixel_data.get((x_pos + y_pos * img.width()) as usize).unwrap();
            let col = Point{
                r: pixel.r as f64,
                g: pixel.g as f64,
                b: pixel.b as f64,
                x:x_pos,
                y:y_pos,
            };
            points.push(col);
        }
    }

    let mut selection_centers: Vec<Point> = Vec::new();
    for i in 0..k {
        let mut rng = rand::thread_rng();
        let selection = rng.gen_range(0..points.len()-k);
        selection_centers.push(points[selection].clone());
        points.remove(selection);
        points.push(selection_centers[i].clone());
    }

    let mut selections: Vec<Vec<Point>> = Vec::new();
    for i in 0..iter_count {
        selections = generate_selections(&points, &selection_centers);
        selection_centers = calculate_centers(&selections);
    }

    draw_selection_to_image(&mut img, &selections);
    img.export_image(&*format!("output{}.png", k));
    let texture = rl.load_texture_from_image(&thread, &img).expect("Failed to load texture!");

    while !rl.window_should_close() {
        let mut g: RaylibDrawHandle  = rl.begin_drawing(&thread);
        g.clear_background(Color::BLACK);
        draw_texture_to_rec(&mut g,&texture, Rectangle::new(0.0,0.0,width as f32,height as f32));
    }
}

fn draw_selection_to_image(img: &mut Image, selections: &Vec<Vec<Point>>) {
    for sel in selections{
        if(sel.len() == 0) {
            continue;
        }
        let avg_r = sel.iter().map(|x| x.r as u32).sum::<u32>() / sel.len() as u32;
        let avg_g = sel.iter().map(|x| x.g as u32).sum::<u32>() / sel.len() as u32;
        let avg_b = sel.iter().map(|x| x.b as u32).sum::<u32>() / sel.len() as u32;
        for point in sel {
            img.draw_pixel(point.x,point.y,Color::new(avg_r as u8,avg_g as u8,avg_b as u8,255));
        }
    }
}

struct Closest{
    index: usize,
    distance: f64,
}

fn generate_selections(points: &Vec<Point>, selection_centers:&Vec<Point>) -> Vec<Vec<Point>> {
    let mut selections: Vec<Vec<Point>> = Vec::new();
    for i in 0..selection_centers.len() {
        selections.push(Vec::new());
    }
    let mut closest: Vec<Closest> = Vec::new();
    for i in 0..points.len(){
        closest.push(Closest{index: 0, distance: f64::MAX});
    }

    for i in 0..selection_centers.len() {
        let center = selection_centers[i].clone();
        for j in 0..points.len() {
            let distance = distance_heuristic(&center, &points[j]);
            if distance < closest[j].distance {
                closest[j].distance = distance;
                closest[j].index = i;
            }
        }
    }

    for i in 0..closest.len() {
        selections[closest[i].index].push(points[i].clone());
    }

    selections
}

fn calculate_centers(selections: &Vec<Vec<Point>>) -> Vec<Point> {
    let mut centers: Vec<Point> = Vec::new();
    for i in 0..selections.len() {
        let mut center = Point{r: 0.0, g: 0.0, b: 0.0,x:0,y:0};
        for j in 0..selections[i].len() {
            center.r += selections[i][j].r;
            center.g += selections[i][j].g;
            center.b += selections[i][j].b;
        }
        center.r /= selections[i].len() as f64;
        center.g /= selections[i].len() as f64;
        center.b /= selections[i].len() as f64;
        centers.push(center);
    }
    centers
}

fn distance_heuristic(p1:&Point, p2:&Point) -> f64 {
    (p1.r - p2.r).powf(2.0) + (p1.g - p2.g).powf(2.0) + (p1.b - p2.b).powf(2.0)
}

fn draw_texture_to_rec(g: &mut RaylibDrawHandle, texture: &Texture2D,rec:Rectangle) {
    g.draw_texture_pro(texture,Rectangle::new(0.0,0.0,texture.width() as f32,texture.height() as f32),rec,Vector2::new(0.0,0.0),0.0,Color::WHITE);
}