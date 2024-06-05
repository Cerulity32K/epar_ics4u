use jut::extensions::Boxed;
use macroquad::math::DVec2;

pub trait Provider<T> {
    fn box_clone(&self) -> Box<dyn Provider<T>>;
    fn get(&self, beat: f64) -> T;
}
pub struct ProviderOffset<T>(pub Box<dyn Provider<T>>, pub f64);
impl<T: 'static> Provider<T> for ProviderOffset<T> {
    fn box_clone(&self) -> Box<dyn Provider<T>> {
        Self(self.0.box_clone(), self.1).boxed()
    }
    fn get(&self, beat: f64) -> T {
        self.0.get(beat - self.1)
    }
}
#[derive(Clone)]
pub struct Constant<T: Clone>(pub T);
impl<T: Clone + 'static> Provider<T> for Constant<T> {
    fn box_clone(&self) -> Box<dyn Provider<T>> {
        self.clone().boxed()
    }
    fn get(&self, _beat: f64) -> T {
        self.0.clone()
    }
}

#[derive(Clone)]
pub struct FnWrap<O: Clone + 'static, F: Fn(f64) -> O + Clone + 'static>(pub F);
impl<O: Clone + 'static, F: Fn(f64) -> O + Clone + 'static> Provider<O> for FnWrap<O, F> {
    fn box_clone(&self) -> Box<dyn Provider<O>> {
        self.clone().boxed()
    }
    fn get(&self, beat: f64) -> O {
        self.0(beat)
    }
}
#[derive(Clone, Copy)]
pub struct Velocity {
    pub start: DVec2,
    pub velocity: DVec2,
}
impl Velocity {
    pub fn new(start: DVec2, velocity: DVec2) -> Self {
        Self { start, velocity }
    }
}
impl Provider<DVec2> for Velocity {
    fn box_clone(&self) -> Box<dyn Provider<DVec2>> {
        (*self).boxed()
    }
    fn get(&self, beat: f64) -> DVec2 {
        self.start + self.velocity * beat
    }
}
