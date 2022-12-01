use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {}

#[derive(Copy, Clone)]
struct Pos {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Pos {
    fn new() -> Self {
        Pos {
            x: 0f64,
            y: 0f64,
            z: 0f64,
            w: 1f64,
        }
    }
}

struct ViewTrans {
    mx_shift: [[f64; 4]; 4],
    mx_rotate_xw: [[f64; 4]; 4],
    mx_rotate_yw: [[f64; 4]; 4],
    mx_rotate_zw: [[f64; 4]; 4],
    mx_reverse_zv: [[f64; 4]; 4],
    mx_trans: [[f64; 4]; 4],
}

struct DisplayPos {
    x: f64,
    y: f64,
}

impl DisplayPos {
    fn new() -> Self {
        DisplayPos { x: 0f64, y: 0f64 }
    }
}

struct Screen {
    width: usize,
    height: usize,
}

struct Stl {
    pos: [Pos; 3],
    normal_vec: Pos,
}

impl Stl {
    fn new() -> Self {
        Stl {
            pos: [Pos::new(); 3],
            normal_vec: Pos::new(),
        }
    }
}

fn cal_matrix_unit() -> [[f64; 4]; 4] {
    let mut mx_unit = [[0f64; 4]; 4];
    mx_unit
        .iter_mut()
        .enumerate()
        .for_each(|(i, row)| row[i] = 1f64);
    mx_unit
}

fn shift(view: Pos) -> [[f64; 4]; 4] {
    let mut mx_shift = cal_matrix_unit();
    mx_shift[0][3] = -view.x;
    mx_shift[1][3] = -view.y;
    mx_shift[2][3] = -view.z;
    mx_shift
}

fn rotate_yw(view: Pos, targ: Pos) -> [[f64; 4]; 4] {
    let delta_x = targ.x - view.x;
    let delta_z = targ.z - view.z;

    let (cos_alpha, sin_alpha) = if delta_x == 0f64 && delta_z == 0f64 {
        (1f64, 0f64)
    } else {
        (
            -delta_z / (delta_x.powi(2) + delta_z.powi(2)).sqrt(),
            delta_x / (delta_x.powi(2) + delta_z.powi(2)).sqrt(),
        )
    };

    let mut mx_rotate_yw = cal_matrix_unit();
    mx_rotate_yw[0][0] = cos_alpha;
    mx_rotate_yw[0][2] = sin_alpha;
    mx_rotate_yw[2][0] = -sin_alpha;
    mx_rotate_yw[2][2] = cos_alpha;
    mx_rotate_yw
}

fn rotate_xw(view: Pos, targ: Pos) -> [[f64; 4]; 4] {
    let delta_x = targ.x - view.x;
    let delta_y = targ.y - view.y;
    let delta_z = targ.z - view.z;

    let (cos_beta, sin_beta) = if delta_x == 0f64 && delta_y == 0f64 && delta_z == 0f64 {
        (
            (delta_x.powi(2) + delta_z.powi(2)).sqrt()
                / (delta_x.powi(2) + delta_y.powi(2) + delta_z.powi(2)).sqrt(),
            -delta_y / (delta_x.powi(2) + delta_y.powi(2) + delta_z.powi(2)).sqrt(),
        )
    } else {
        (1f64, 0f64)
    };

    let mut mx_rotate_xw = cal_matrix_unit();
    mx_rotate_xw[1][1] = cos_beta;
    mx_rotate_xw[1][2] = -sin_beta;
    mx_rotate_xw[2][1] = sin_beta;
    mx_rotate_xw[2][2] = cos_beta;

    mx_rotate_xw
}

fn rotate_zw(view: Pos, targ: Pos) -> [[f64; 4]; 4] {
    let mut mx_rotate_zw = cal_matrix_unit();

    let gamma = 0f64;

    mx_rotate_zw[0][0] = gamma.to_radians().cos();
    mx_rotate_zw[0][1] = -gamma.to_radians().sin();
    mx_rotate_zw[1][0] = gamma.to_radians().sin();
    mx_rotate_zw[1][1] = gamma.to_radians().cos();

    mx_rotate_zw
}

fn reverse_zv() -> [[f64; 4]; 4] {
    let mut mx_reverse_zv = cal_matrix_unit();
    mx_reverse_zv[2][2] = -1f64;
    mx_reverse_zv
}

fn cal_matrix(matrix_a: &[[f64; 4]; 4], matrix_b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut result_matrix = [[0f64; 4]; 4];

    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result_matrix[i][j] += matrix_a[i][k] * matrix_b[k][j];
            }
        }
    }

    result_matrix
}

fn cal_pos(matrix_a: &[[f64; 4]; 4], pos: &Pos) -> Pos {
    let matrix_b = [pos.x, pos.y, pos.z, pos.w];
    let mut result_matrix: [f64; 4] = [0f64; 4];
    // let mut result_matrix: [f64; 4] =  matrix_a.iter().map(|a| a.iter().zip(matrix_b.iter()).map(|(a, b)| a * b).collect::<Vec<f64>>()).collect::<Vec<Vec<f64>>>().try_into().unwrap();
    for i in 0..4 {
        for j in 0..4 {
            result_matrix[i] += matrix_a[i][j] * matrix_b[j];
        }
    }

    Pos {
        x: result_matrix[0],
        y: result_matrix[1],
        z: result_matrix[2],
        w: result_matrix[3],
    }
}
