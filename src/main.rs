use eframe::egui::*;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn main() {
    
    // let model: Model = Model::init(read_stl("./stl/modeling_robo.stl"));
    // let view = ThreeDPos::init(10f64, 15f64, 10f64);
    // let targ = ThreeDPos::init(0f64, 0f64, 0f64);
    // let gamma = 0f64;
    // let screen = ScreenTrans::init(640, 480, 50f64);
    // screen.cal_mx_screen(&view);
    // // 不変化
    // let screen = screen;

    // let mut view_param = ViewTrans::init(
    //     shift(&view),
    //     rotate_yw(&view, &targ),
    //     rotate_xw(&view, &targ),
    //     rotate_zw(&gamma),
    // );

    // view_param.cal_mx_view_trans();
    // // 不変化
    // let view_param = view_param;

    // model.cal_view_pos(&view_param.mx_view_trans);
    // model.cal_screen_pos(&screen.mx_screen_trans);
    // model.cal_display_pos(&screen);

    // // 不変化
    // let model = model;

    //以下描画
    let native_options = eframe::NativeOptions {
        initial_window_size: Some((1280f32, 720f32).into()),
        resizable: false,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "RoboWire",
        native_options,
        Box::new(|cc| {
            Box::new(DisplayRobo::init(
                cc,
                Model::init(read_stl()),
                ThreeDPos::init(0f64, 20f64, 0f64),
                ThreeDPos::init(0f64, 0f64, 0f64),
                0f64,
                ScreenTrans::init(1280, 720),
            ))
        }),
    );
}

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

    fn init(x: f64, y: f64, z: f64) -> Self {
        ThreeDPos { x, y, z, w: 1f64 }
    }
}

struct Stl {
    pos: [ThreeDPos; 3],
    _normal_vec: ThreeDPos,
}

impl Stl {
    fn new() -> Self {
        Stl {
            pos: [ThreeDPos::new(); 3],
            _normal_vec: ThreeDPos::new(),
        }
    }
}

#[derive(Copy, Clone)]
struct TwoDPos {
    x: f64,
    y: f64,
}

impl TwoDPos {
    fn new() -> Self {
        TwoDPos { x: 0f64, y: 0f64 }
    }

    fn to_pos2(&self) -> Pos2 {
        pos2(self.x as f32, self.y as f32)
    }
}

struct Model {
    stl: Vec<Stl>,
    view: Vec<[ThreeDPos; 3]>,
    screen: Vec<[TwoDPos; 3]>,
    display: Vec<[TwoDPos; 3]>,
}

impl Model {
    fn new() -> Self {
        Model {
            stl: Vec::new(),
            view: Vec::new(),
            screen: Vec::new(),
            display: Vec::new(),
        }
    }

    fn init(stl: Vec<Stl>) -> Self {
        Model {
            stl,
            view: Vec::new(),
            screen: Vec::new(),
            display: Vec::new(),
        }
    }

    fn cal_view_pos(&mut self, mx_view_trans: &[[f64; 4]; 4]) {
        self.stl.iter().for_each(|s| {
            let mut view_pos = [ThreeDPos::new(); 3];
            for i in 0..view_pos.len() {
                let mx_pos = [s.pos[i].x, s.pos[i].y, s.pos[i].z, s.pos[i].w];
                let mx_result = cal_pos(&mx_view_trans, &mx_pos);
                // let mut mx_result = [0f64; 4];
                // for j in 0..mx_pos.len() {
                //     for k in 0..mx_pos.len() {
                //         mx_result[j] += mx_view_trans[j][k] * mx_pos[k];
                //     }
                // }
                view_pos[i] = ThreeDPos {
                    x: mx_result[0],
                    y: mx_result[1],
                    z: mx_result[2],
                    w: mx_result[3],
                };
            }
            self.view.push(view_pos);
        })
    }


    fn cal_screen_pos(&mut self, depth: f64) {
        self.view.iter().for_each(|p| {
            let mut screen_pos = [TwoDPos::new(); 3];
            for i in 0..screen_pos.len() {
                let mx_pos = [p[i].x, p[i].y, p[i].z, p[i].w];


                let mx_result = cal_pos(&cal_screen_trans(&depth, &p[i]), &mx_pos);
                // for j in 0..mx_pos.len() {
                //     for k in 0..mx_pos.len() {
                //         mx_result[j] += cal_screen_trans(&depth, &p[i])[j][k] * mx_pos[k];
                //     }
                // }
                
                screen_pos[i] = TwoDPos {
                    x: mx_result[0],
                    y: mx_result[1],
                };
            }
            self.screen.push(screen_pos);
        })
    }

    fn cal_display_pos(&mut self, screen_trans: &ScreenTrans) {
        self.screen.iter().for_each(|p| {
            let mut screen_pos: [TwoDPos; 3] = [TwoDPos::new(); 3];
            for i in 0..screen_pos.len() {
                screen_pos[i] = TwoDPos {
                    x: screen_trans.width as f64 / 2f64 + p[i].x,
                    y: screen_trans.height as f64 / 2f64 - p[i].y,
                };
            }
            self.display.push(screen_pos);
        });
    }

    fn clear_trans_pos(&mut self) {
        self.view = Vec::new();
        self.screen = Vec::new();
        self.display = Vec::new();
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
        ViewTrans {
            mx_shift: cal_mx_unit(),
            mx_rotate_yw: cal_mx_unit(),
            mx_rotate_xw: cal_mx_unit(),
            mx_rotate_zw: cal_mx_unit(),
            mx_reverse_zv: cal_mx_unit(),
            mx_view_trans: cal_mx_unit(),
        }
    }

    fn init(
        mx_shift: [[f64; 4]; 4],
        mx_rotate_yw: [[f64; 4]; 4],
        mx_rotate_xw: [[f64; 4]; 4],
        mx_rotate_zw: [[f64; 4]; 4],
    ) -> Self {
        ViewTrans {
            mx_shift,
            mx_rotate_yw,
            mx_rotate_xw,
            mx_rotate_zw,
            mx_reverse_zv: reverse_zv(),
            mx_view_trans: cal_mx_unit(),
        }
    }

    fn cal_mx_view_trans(&mut self) {
        let params = [
            self.mx_reverse_zv,
            self.mx_rotate_zw,
            self.mx_rotate_xw,
            self.mx_rotate_yw,
            self.mx_shift,
        ];
        self.mx_view_trans = params
            .iter()
            .copied()
            .reduce(|a, b| cal_matrix(&a, &b))
            .unwrap();
    }
}

struct ScreenTrans {
    height: usize,
    width: usize,
    depth: f64,
}

impl ScreenTrans {
    fn init(width: usize, height: usize) -> Self {
        ScreenTrans {
            height,
            width,
            depth: 0f64,
        }
    }
}

fn cal_screen_trans(depth: &f64,  pv: &ThreeDPos) -> [[f64; 4]; 4]{
    
    let ratio = depth / pv.z;
    println!("{}", ratio);
    [
        [ratio, 0f64, 0f64, 0f64],
        [0f64, ratio, 0f64, 0f64],
        [0f64, 0f64, ratio, 0f64],
        [0f64, 0f64, 0f64, 1f64],
    ]
}

fn cal_distance_2d(pos_a: &TwoDPos, pos_b: &TwoDPos) -> f64 {
    let delta_x = pos_a.x - pos_b.x;
    let delta_y = pos_a.y - pos_b.y;
    (delta_x.powi(2) + delta_y.powi(2)).sqrt()
}

fn cal_distance_3d(pos_a: &ThreeDPos, pos_b: &ThreeDPos) -> f64 {
    let delta_x = pos_a.x - pos_b.x;
    let delta_y = pos_a.y - pos_b.y;
    let delta_z = pos_a.z - pos_b.z;
    (delta_x.powi(2) + delta_y.powi(2) +  delta_z .powi(2)).sqrt()
}


fn shift(view: &ThreeDPos) -> [[f64; 4]; 4] {
    let mut mx_shift = cal_mx_unit();
    mx_shift[0][3] = -view.x;
    mx_shift[1][3] = -view.y;
    mx_shift[2][3] = -view.z;
    mx_shift
}

fn rotate_yw(view: &ThreeDPos, targ: &ThreeDPos) -> [[f64; 4]; 4] {
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

fn rotate_xw(view: &ThreeDPos, targ: &ThreeDPos) -> [[f64; 4]; 4] {
    let delta_x = targ.x - view.x;
    let delta_y = targ.y - view.y;
    let delta_z = targ.z - view.z;

    let (cos_beta, sin_beta) = if delta_x == 0f64 && delta_y == 0f64 && delta_z == 0f64 {
        (1f64, 0f64)
    } else {
        (
            (delta_x.powi(2) + delta_z.powi(2)).sqrt()
                / (delta_x.powi(2) + delta_y.powi(2) + delta_z.powi(2)).sqrt(),
            -delta_y / (delta_x.powi(2) + delta_y.powi(2) + delta_z.powi(2)).sqrt(),
        )
    };

    let mut mx_rotate_xw = cal_mx_unit();
    mx_rotate_xw[1][1] = cos_beta;
    mx_rotate_xw[1][2] = -sin_beta;
    mx_rotate_xw[2][1] = sin_beta;
    mx_rotate_xw[2][2] = cos_beta;

    mx_rotate_xw
}

fn rotate_zw(gamma: &f64) -> [[f64; 4]; 4] {
    let mut mx_rotate_zw = cal_mx_unit();

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

fn cal_pos(mx_a: &[[f64; 4]; 4], mx_pos: &[f64; 4]) -> [f64; 4] {
    let mut mx_result = [0f64; 4];
    for i in 0..mx_pos.len() {
        for j in 0..mx_pos.len() {
            mx_result[i] += mx_a[i][j] * mx_pos[j];
        }
    }
    mx_result
}

fn read_stl() -> Vec<Stl> {

    println!("input path > ");

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();

    let path = buf.trim().split(">").nth(0).unwrap().trim().clone();

    let mut stl_model: Vec<Stl> = Vec::new();
    println!("{}", &path);

    let file_to_read = File::open(path).expect("ファイルオープンに失敗");
    let mut file_reader = BufReader::new(file_to_read);

    let mut buf = String::new();

    loop {
        file_reader.read_line(&mut buf).unwrap();
        buf.clear();
        // 単位法線ベクトル
        file_reader.read_line(&mut buf).unwrap();
        if buf.trim().split(" ").nth(0).unwrap() == "endsolid" {
            break;
        };
        let tmp_n_vec = buf.trim().split(" ").collect::<Vec<&str>>()[2..]
            .iter()
            .map(|s| s.trim().parse::<f64>().unwrap())
            .collect::<Vec<f64>>();
        buf.clear();
        // ゴミ("outer loop")
        file_reader.read_line(&mut buf).unwrap();
        buf.clear();
        // 3ポリゴン
        let mut tmp_poly: [ThreeDPos; 3] = [ThreeDPos::new(); 3];
        for i in 0..3 {
            file_reader.read_line(&mut buf).unwrap();
            let tmp = buf.trim().split(" ").collect::<Vec<&str>>()[1..]
                .iter()
                .map(|s| s.trim().parse::<f64>().unwrap())
                .collect::<Vec<f64>>();
            tmp_poly[i].x = tmp[0];
            tmp_poly[i].y = tmp[1];
            tmp_poly[i].z = tmp[2];
            tmp_poly[i].w = 1f64;
            buf.clear();
        }
        // 不変化
        let tmp_poly = tmp_poly;

        // ゴミ("endloop", "endfacet")
        file_reader.read_line(&mut buf).unwrap();
        buf.clear();

        stl_model.push(Stl {
            pos: tmp_poly,
            _normal_vec: ThreeDPos {
                x: tmp_n_vec[0],
                y: tmp_n_vec[1],
                z: tmp_n_vec[2],
                w: 1f64,
            },
        })
    }

    stl_model
}

pub struct DisplayRobo {
    model: Model,
    view: ThreeDPos,
    targ: ThreeDPos,
    gamma: f64,
    screen: ScreenTrans,
    view_param: ViewTrans,
}

impl DisplayRobo {
    fn init(
        _cc: &eframe::CreationContext<'_>,
        model: Model,
        view: ThreeDPos,
        targ: ThreeDPos,
        gamma: f64,
        screen: ScreenTrans,
    ) -> Self {
        DisplayRobo {
            model,
            view,
            targ,
            gamma,
            screen,
            view_param: ViewTrans::new(),
        }
    }
}

impl eframe::App for DisplayRobo {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
    fn update(&mut self, _ctx: &Context, _frame: &mut eframe::Frame) {

        // let x_angle: i32 = if self.view.x == 0f64 {
        //     if self.view.y > 0f64 {
        //         180
        //     } else if self.view.y < 0f64 {
        //         0
        //     // } else {
        //     //     panic!("視点と目標点が一致しています");
        //     }
        // } else {
        //     (self.view.x / cal_distance_2d(&TwoDPos { x: self.view.x, y: self.view.y }, &TwoDPos { x: self.targ.x, y: self.targ.y })).acos().to_degrees() as i32
        // };

        // println!("{}", x_angle);

        // (cal_distance_2d(&TwoDPos { x: self.view.x, y: self.view.y }, &TwoDPos { x: self.targ.x, y: self.targ.y }) / self.view.x).acos();
        // println!("{}, {}, {}", if self.view.x == 0f64 {
        //     if self.view.y > 0f64 {
        //         180
        //     } else if self.view.y < 0f64 {
        //         0
        //     } else {
        //         panic!("視点と目標点が一致しています");
        //     }
        // } else {
        //     (cal_distance_2d(&TwoDPos { x: self.view.x, y: self.view.y }, &TwoDPos { x: self.targ.x, y: self.targ.y }) / self.view.x).acos() as i32
        // }, self.view.x, x_angle);
        
        println!("{}", self.view.y);
        self.view.y = match self.view.y as isize {
            1 => 20f64,
            _ => self.view.y + -1f64,
        };

        self.screen.depth = cal_distance_3d(&self.view, &self.targ);

        self.model.clear_trans_pos();

        self.view_param = ViewTrans::init(
            shift(&self.view),
            rotate_yw(&self.view, &self.targ),
            rotate_xw(&self.view, &self.targ),
            rotate_zw(&self.gamma),
        );
        self.view_param.cal_mx_view_trans();

        self.model.cal_view_pos(&self.view_param.mx_view_trans);
        self.model.cal_screen_pos(self.screen.depth);
        self.model.cal_display_pos(&self.screen);

        println!("{}, {}, {}", self.model.view[9][1].x, self.model.view[9][1].y, self.model.view[9][1].z);

        println!("{}, {}", self.model.screen[9][1].x, self.model.screen[9][1].y);

        CentralPanel::default().show(_ctx, |ui| {
            self.model.display.iter().for_each(|p| {
                ui.painter().line_segment(
                    [p[1].to_pos2(), p[0].to_pos2()],
                    Stroke::new(1f32, Color32::YELLOW),
                );
                ui.painter().line_segment(
                    [p[2].to_pos2(), p[1].to_pos2()],
                    Stroke::new(1f32, Color32::YELLOW),
                );
                ui.painter().line_segment(
                    [p[0].to_pos2(), p[2].to_pos2()],
                    Stroke::new(1f32, Color32::YELLOW),
                );
            })
        });
    }
}
