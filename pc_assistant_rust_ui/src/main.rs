use eframe::{egui, App, Frame, NativeOptions};
use std::time::Instant;

struct PetApp {
    last_click: Option<Instant>,
    message: String,
    show_message: bool,
    is_dragging: bool,
    drag_offset_x: f32,
    drag_offset_y: f32,
    blink_timer: Instant,
    blink_state: bool,
    float_timer: Instant,
    float_offset: f32,
}

impl Default for PetApp {
    fn default() -> Self {
        Self {
            last_click: None,
            message: "你好！我是你的PC个人助手".to_string(),
            show_message: true,
            is_dragging: false,
            drag_offset_x: 0.0,
            drag_offset_y: 0.0,
            blink_timer: Instant::now(),
            blink_state: false,
            float_timer: Instant::now(),
            float_offset: 0.0,
        }
    }
}

impl App for PetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // 处理点击事件
        if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                if pos.x >= 0.0 && pos.x <= 200.0 && pos.y >= 0.0 && pos.y <= 200.0 {
                    self.last_click = Some(Instant::now());
                    self.message = "有什么可以帮助你的吗？".to_string();
                    self.show_message = true;
                } else {
                    // 开始拖动
                    self.is_dragging = true;
                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        self.drag_offset_x = pos.x;
                        self.drag_offset_y = pos.y;
                    }
                }
            }
        }

        // 结束拖动
        if ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
            self.is_dragging = false;
        }

        // 处理眨眼动画
        if self.blink_timer.elapsed().as_millis() > 3000 {
            self.blink_state = !self.blink_state;
            self.blink_timer = Instant::now();
        }

        // 处理浮动动画
        let float_time = self.float_timer.elapsed().as_secs_f32();
        self.float_offset = (float_time * 2.0).sin() * 5.0;

        // 自动隐藏消息
        if self.show_message && self.last_click.is_some() && self.last_click.unwrap().elapsed().as_secs() > 3 {
            self.show_message = false;
        }

        // 绘制桌宠
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_width(200.0);
            ui.set_height(200.0);

            // 绘制消息气泡
            if self.show_message {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(255, 255, 255))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                        .rounding(8.0)
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(self.message.clone()).wrap(true));
                        });
                    ui.add_space(10.0);
                });
            }

            // 绘制桌宠主体
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(255, 215, 0))
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 180, 0)))
                .rounding(100.0)
                .show(ui, |ui| {
                    ui.set_width(160.0);
                    ui.set_height(160.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);

                        // 绘制眼睛
                        ui.horizontal(|ui| {
                            ui.add_space(30.0);
                            egui::Frame::none()
                                .fill(egui::Color32::BLACK)
                                .rounding(10.0)
                                .show(ui, |ui| {
                                    ui.set_width(20.0);
                                    if self.blink_state {
                                        ui.set_height(2.0);
                                    } else {
                                        ui.set_height(20.0);
                                    }
                                });
                            ui.add_space(40.0);
                            egui::Frame::none()
                                .fill(egui::Color32::BLACK)
                                .rounding(10.0)
                                .show(ui, |ui| {
                                    ui.set_width(20.0);
                                    if self.blink_state {
                                        ui.set_height(2.0);
                                    } else {
                                        ui.set_height(20.0);
                                    }
                                });
                            ui.add_space(30.0);
                        });

                        ui.add_space(20.0);

                        // 绘制嘴巴
                        egui::Frame::none()
                            .fill(egui::Color32::BLACK)
                            .rounding(10.0)
                            .show(ui, |ui| {
                                ui.set_width(40.0);
                                ui.set_height(20.0);
                            });
                    });
                });
        });
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([200.0, 200.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "PC个人助手",
        options,
        Box::new(|_cc| Box::new(PetApp::default())),
    )
    .unwrap();
}
