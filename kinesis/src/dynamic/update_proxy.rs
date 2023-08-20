use web_sys::console;

use crate::fragment::Location;

use super::Dynamic;

pub type UpdateProxyFn = dyn Fn(&[usize]) -> Vec<usize>;

/// A proxy for [`Dynamic`] things, allowing for a custom `update` function to be called before the
/// original `update` is called.
pub struct UpdateProxy {
    /// The [`Dynamic`] to be rendered with the proxy.
    dynamic: Box<dyn Dynamic>,

    /// The update function to run before the original update function. This can include re-writing
    /// the changed dependencies, which can be passed onwards.
    proxy_update: Box<UpdateProxyFn>,
}

impl UpdateProxy {
    /// Create a new proxy with the provided dynamic and update function.
    pub fn new<D, U>(dynamic: D, proxy_update: U) -> Self
    where
        D: 'static + Dynamic,
        U: 'static + Fn(&[usize]) -> Vec<usize>,
    {
        Self {
            dynamic: Box::new(dynamic) as Box<dyn Dynamic>,
            proxy_update: Box::new(proxy_update) as Box<UpdateProxyFn>,
        }
    }
}

impl Dynamic for UpdateProxy {
    fn mount(&mut self, location: &Location) {
        self.dynamic.mount(location);
    }

    fn detach(&mut self, top_level: bool) {
        self.dynamic.detach(top_level);
    }

    fn update(&mut self, changed: &[usize]) {
        console::log_1(&"update called".into());

        // Run the update function
        let changed = (self.proxy_update)(changed);

        // Run the original dynamic update
        self.dynamic.update(&changed);
    }
}
