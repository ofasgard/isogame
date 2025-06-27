use godot::prelude::*;
use godot::classes::ProgressBar;
use godot::classes::IProgressBar;

#[derive(GodotClass)]
#[class(base=ProgressBar,init)]
pub struct HealthBar {
	timer: f64,
	base: Base<ProgressBar>
}

#[godot_api]
impl IProgressBar for HealthBar {
	fn process(&mut self, delta: f64) {
		self.timer = f64::max(0.00, self.timer - delta);
		if self.timer == 0.00 { self.base_mut().hide(); }
	}
}

impl HealthBar {
	pub fn update(&mut self, value: f64) {
		self.base_mut().set_value(value);
		self.base_mut().show();
		self.timer = 3.0;
	}
}
