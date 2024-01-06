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

    let k = 8;
    let iter_count = 10;
    let mut points: Vec<Point> = Vec::new();

    let file_name = "birb.png";
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
        println!("Iteration: {}",i);
        selections = generate_selections(&points, &selection_centers);
        selection_centers = calculate_centers(&selections);
    }

    dither_floyd_steinberg(&mut img,&selection_centers);

    img.export_image(&*format!("output{}.png", k));
    let texture = rl.load_texture_from_image(&thread, &img).expect("Failed to load texture!");

    while !rl.window_should_close() {
        let mut g: RaylibDrawHandle  = rl.begin_drawing(&thread);
        g.clear_background(Color::BLACK);
        draw_texture_to_rec(&mut g,&texture, Rectangle::new(0.0,0.0,width as f32,height as f32));
    }
}

fn dither_floyd_steinberg(img:&mut Image,colors:&Vec<Point>){
    let mut pixel_data: ImageColors = img.get_image_data();
    for y in 0..img.height()-1 {
        for x in 0..img.width()-1 {
            let old_pixel = pixel_data.get_mut((x + y * img.width()) as usize).unwrap();
            let mut smallest_distance = f64::MAX;
            let mut closest_color = Color::BLACK;
            for color in colors{
                let distance = distance_heuristic(&Point{r:old_pixel.r as f64,g:old_pixel.g as f64,b:old_pixel.b as f64,x:0,y:0}
                                                     ,&Point{r:color.r as f64,g:color.g as f64,b:color.b as f64,x:0,y:0});
                if distance < smallest_distance {
                    smallest_distance = distance;
                    closest_color = Color::new(color.r as u8,color.g as u8,color.b as u8,255);
                }
            }
            let error_r = old_pixel.r as i32 - closest_color.r as i32;
            let error_g = old_pixel.g as i32 - closest_color.g as i32;
            let error_b = old_pixel.b as i32 - closest_color.b as i32;
            img.draw_pixel(x,y,closest_color);
            let right_pixel = pixel_data.get_mut((x + 1 + y * img.width()) as usize).unwrap();
            right_pixel.r = clamp(right_pixel.r as i32 + error_r * 7 / 16,0,255) as u8;
            right_pixel.g = clamp(right_pixel.g as i32 + error_g * 7 / 16,0,255) as u8;
            right_pixel.b = clamp(right_pixel.b as i32 + error_b * 7 / 16,0,255) as u8;
            let down_left_pixel = pixel_data.get_mut((x - 1 + (y + 1) * img.width()) as usize).unwrap();
            down_left_pixel.r = clamp(down_left_pixel.r as i32 + error_r * 3 / 16,0,255) as u8;
            down_left_pixel.g = clamp(down_left_pixel.g as i32 + error_g * 3 / 16,0,255) as u8;
            down_left_pixel.b = clamp(down_left_pixel.b as i32 + error_b * 3 / 16,0,255) as u8;
            let down_pixel = pixel_data.get_mut((x + (y + 1) * img.width()) as usize).unwrap();
            down_pixel.r = clamp(down_pixel.r as i32 + error_r * 5 / 16,0,255) as u8;
            down_pixel.g = clamp(down_pixel.g as i32 + error_g * 5 / 16,0,255) as u8;
            down_pixel.b = clamp(down_pixel.b as i32 + error_b * 5 / 16,0,255) as u8;
            let down_right_pixel = pixel_data.get_mut((x + 1 + (y + 1) * img.width()) as usize).unwrap();
            down_right_pixel.r = clamp(down_right_pixel.r as i32 + error_r * 1 / 16,0,255) as u8;
            down_right_pixel.g = clamp(down_right_pixel.g as i32 + error_g * 1 / 16,0,255) as u8;
            down_right_pixel.b = clamp(down_right_pixel.b as i32 + error_b * 1 / 16,0,255) as u8;
        }
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

fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}