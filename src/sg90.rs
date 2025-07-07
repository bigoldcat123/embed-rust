use defmt::error;
use embassy_stm32::timer::{GeneralInstance4Channel, simple_pwm::SimplePwmChannel};

static FACTOR: f32 = 100_f32 / 180_f32;

pub struct SG90<'a, T: GeneralInstance4Channel> {
    pwm: SimplePwmChannel<'a, T>,
}
impl<'a, T: GeneralInstance4Channel> SG90<'a, T> {
    pub fn new(mut pwm: SimplePwmChannel<'a, T>) -> Self {
        pwm.enable();
        Self { pwm }
    }
    /// 0-180
    pub fn turn(&mut self, deg: u16) {
        if deg > 180 {
            error!("bigger than 180");
            return;
        }
        // 25 - 125 = (125 - 25 / 180)
        // 0  - 180
        self.pwm
            .set_duty_cycle_fraction(25 + (deg as f32 * FACTOR) as u16, 1000);
    }
}
