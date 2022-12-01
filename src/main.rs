use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {}

#[derive(Copy, Clone)]
struct ThreeDPos {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl ThreeDPos {
    fn new() -> Self {
        ThreeDPos {
            x: 0f64,
            y: 0f64,
            z: 0f64,
            w: 1f64,
        }
    }
}

struct TwoDPos {
    x: f64,
    y: f64,
}

impl TwoDPos {
    fn new() -> Self {
        TwoDPos {
            x: 0f64,
            y: 0f64,
        }
    }
}

struct ViewTrans {
    mx_shift: [[f64; 4]; 4],
    mx_rotate_xw: [[f64; 4]; 4],
    mx_rotate_yw: [[f64; 4]; 4],
    mx_rotate_zw: [[f64; 4]; 4],
    mx_reverse_zv: [[f64; 4]; 4],
    mx_view_trans: [[f64; 4]; 4],
}

impl ViewTrans {
    fn new() -> Self {
        ViewTrans { mx_shift: cal_mx_unit(), mx_rotate_xw: cal_mx_unit(), mx_rotate_yw: cal_mx_unit(), mx_rotate_zw: cal_mx_unit(), mx_reverse_zv: cal_mx_unit(), mx_view_trans: cal_mx_unit() }
    }
    
    fn cal_mx_view_trans(mut self) {
        let params = [self.mx_reverse_zv, self.mx_rotate_zw,  self.mx_rotate_xw, self.mx_rotate_yw, self.mx_shift];
        self.mx_shift = params.iter().copied().reduce(|a, b| cal_matrix(&a, &b)).unwrap();
    }
}

struct ScreenTrans {    
    height: usize,
    width: usize,
    depth: f64,
    mx_screen: [[f64; 4]; 4],
}

impl ScreenTrans {
    fn new(height: usize, width: usize, depth: f64) -> Self {
        ScreenTrans { height, width, depth, mx_screen: cal_mx_unit() }
    }

    fn cal_mx_screen(mut self, view: &ThreeDPos) {
        let ratio = self.depth / view.z;
        self.mx_screen = [
            [ratio, 0f64, 0f64, 0f64],
            [0f64, ratio, 0f64, 0f64],
            [0f64, 0f64, ratio, 0f64],
            [0f64, 0f64, 0f64, 1f64]
        ];
    }
}

struct Stl {
    pos: [ThreeDPos; 3],
    normal_vec: ThreeDPos,
}

impl Stl {
    fn new() -> Self {
        Stl {
            pos: [ThreeDPos::new(); 3],
            normal_vec: ThreeDPos::new(),
        }
    }
}

fn shift(view: ThreeDPos) -> [[f64; 4]; 4] {
    let mut mx_shift = cal_mx_unit();
    mx_shift[0][3] = -view.x;
    mx_shift[1][3] = -view.y;
    mx_shift[2][3] = -view.z;
    mx_shift
}

fn rotate_yw(view: ThreeDPos, targ: ThreeDPos) -> [[f64; 4]; 4] {
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

    let mut mx_rotate_yw = cal_mx_unit();
    mx_rotate_yw[0][0] = cos_alpha;
    mx_rotate_yw[0][2] = sin_alpha;
    mx_rotate_yw[2][0] = -sin_alpha;
    mx_rotate_yw[2][2] = cos_alpha;
    mx_rotate_yw
}

fn rotate_xw(view: ThreeDPos, targ: ThreeDPos) -> [[f64; 4]; 4] {
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

    let mut mx_rotate_xw = cal_mx_unit();
    mx_rotate_xw[1][1] = cos_beta;
    mx_rotate_xw[1][2] = -sin_beta;
    mx_rotate_xw[2][1] = sin_beta;
    mx_rotate_xw[2][2] = cos_beta;

    mx_rotate_xw
}

fn rotate_zw(view: ThreeDPos, targ: ThreeDPos) -> [[f64; 4]; 4] {
    let mut mx_rotate_zw = cal_mx_unit();

    let gamma = 0f64;

    mx_rotate_zw[0][0] = gamma.to_radians().cos();
    mx_rotate_zw[0][1] = -gamma.to_radians().sin();
    mx_rotate_zw[1][0] = gamma.to_radians().sin();
    mx_rotate_zw[1][1] = gamma.to_radians().cos();

    mx_rotate_zw
}

fn reverse_zv() -> [[f64; 4]; 4] {
    let mut mx_reverse_zv = cal_mx_unit();
    mx_reverse_zv[2][2] = -1f64;
    mx_reverse_zv
}

fn cal_mx_unit() -> [[f64; 4]; 4] {
    let mut mx_unit = [[0f64; 4]; 4];
    mx_unit
        .iter_mut()
        .enumerate()
        .for_each(|(i, row)| row[i] = 1f64);
    mx_unit
}

fn cal_matrix(mx_a: &[[f64; 4]; 4], mx_b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut mx_result = [[0f64; 4]; 4];

    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                mx_result[i][j] += mx_a[i][k] * mx_b[k][j];
            }
        }
    }

    mx_result
}


fn cal_view_pos(mx_view_trans: &[[f64; 4]; 4], world: &ThreeDPos) -> ThreeDPos {
    let mx_world = [world.x, world.y, world.z, world.w];
    let mut mx_result: [f64; 4] = [0f64; 4];
    for i in 0..4 {
        for j in 0..4 {
            mx_result[i] += mx_view_trans[i][j] * mx_world[j];
        }
    }

    ThreeDPos {
        x: mx_result[0],
        y: mx_result[1],
        z: mx_result[2],
        w: mx_result[3],
    }
}

fn cal_screen_pos(mx_screen_trans: &[[f64; 4]; 4], view: &ThreeDPos) -> TwoDPos {
    let mx_view = [view.x, view.y, view.z, view.w];
    let mut mx_result: [f64; 4] = [0f64; 4];
    for i in 0..4 {
        for j in 0..4 {
            mx_result[i] += mx_screen_trans[i][j] * mx_view[j];
        }
    }

    TwoDPos {
        x: mx_result[0],
        y: mx_result[1],
    }
}

fn cal_display_pos(screen_trans: &ScreenTrans, pos: &TwoDPos) -> TwoDPos {
    TwoDPos { x: screen_trans.width as f64 / 2f64 + pos.x, y: screen_trans.height as f64 / 2f64 - pos.y }
}