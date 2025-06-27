use godot::prelude::*;
use godot::classes::ProgressBar;
use godot::classes::IProgressBar;
use godot::classes::StyleBoxFlat;

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
		self.set_color();
		self.timer = 3.0;
	}
	
	pub fn set_color(&mut self) {
		let mut stylebox : Gd<StyleBoxFlat> = self.base().get_theme_stylebox("fill").unwrap().cast();
		
		let color = match self.base().get_value() {
			0.0..33.0 => Color::from_rgb(255.0, 0.0, 0.0),
			33.0..66.0 => Color::from_rgb(255.0, 255.0, 0.0),
			66.0..=100.0 => Color::from_rgb(0.0, 255.0, 0.0),
			_ => panic!()
		};
		
		stylebox.set_bg_color(color);
	}
}
