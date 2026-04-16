use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::RenderUiSet;
use crate::AppState;

const TOAST_DURATION_SECS: f32 = 4.0;

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub timer: f32,
}

#[derive(Clone, Debug)]
pub enum ToastKind {
    Success,
    Warning,
    Error,
}

impl ToastKind {
    fn color(&self) -> egui::Color32 {
        match self {
            ToastKind::Success => egui::Color32::from_rgb(50, 180, 80),
            ToastKind::Warning => egui::Color32::from_rgb(210, 160, 40),
            ToastKind::Error => egui::Color32::from_rgb(200, 60, 60),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            ToastKind::Success => "OK",
            ToastKind::Warning => "Warning",
            ToastKind::Error => "Error",
        }
    }
}

#[derive(Resource, Default)]
pub struct Notifications {
    toasts: Vec<Toast>,
}

impl Notifications {
    pub fn success(&mut self, message: impl Into<String>) {
        self.toasts.push(Toast {
            message: message.into(),
            kind: ToastKind::Success,
            timer: TOAST_DURATION_SECS,
        });
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.toasts.push(Toast {
            message: message.into(),
            kind: ToastKind::Error,
            timer: TOAST_DURATION_SECS,
        });
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.toasts.push(Toast {
            message: message.into(),
            kind: ToastKind::Warning,
            timer: TOAST_DURATION_SECS,
        });
    }
}

fn tick_notifications(time: Res<Time>, mut notifications: ResMut<Notifications>) {
    let dt = time.delta_secs();
    notifications.toasts.retain_mut(|toast| {
        toast.timer -= dt;
        toast.timer > 0.0
    });
}

fn render_notifications(mut contexts: EguiContexts, notifications: Res<Notifications>) {
    if notifications.toasts.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::new("toast_notifications"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -40.0))
        .order(egui::Order::Foreground)
        .show(contexts.ctx_mut(), |ui| {
            for toast in notifications.toasts.iter().rev() {
                let alpha = if toast.timer < 1.0 {
                    (toast.timer * 255.0) as u8
                } else {
                    255
                };
                let bg = toast.kind.color();
                let bg = egui::Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), alpha);
                let text_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(toast.kind.label())
                                    .color(text_color)
                                    .strong(),
                            );
                            ui.label(egui::RichText::new(&toast.message).color(text_color));
                        });
                    });
                ui.add_space(4.0);
            }
        });
}

pub struct NotificationsPlugin;

impl Plugin for NotificationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Notifications>()
            .add_systems(Update, tick_notifications)
            .add_systems(
                Update,
                render_notifications
                    .after(RenderUiSet)
                    .run_if(AppState::in_displaying_mode()),
            );
    }
}
