#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Route {
    #[default]
    Search,
    Import,
}

#[derive(Default)]
pub struct Router {
    pub c_route: Route,
}

impl Router {
    pub fn set(&mut self, r: Route) {
        self.c_route = r;
    }
}
