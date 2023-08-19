use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

trait Dynamic {
    type Ctx: Any + ?Sized;

    fn mount(&mut self);
    fn detach(&mut self);

    fn update(&mut self, context: &Self::Ctx, changed: &[usize]);
}

type AnyComponent = dyn Component<Ctx = dyn Any>;

struct ControllerChild<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: Component + ?Sized,
{
    controller: Rc<RefCell<Controller<C>>>,
    update: Box<dyn Fn(&Ctx, &mut C)>,
}
impl<Ctx, C> ControllerChild<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: Component + ?Sized,
{
    fn update(&self, context: &Ctx) {
        (self.update)(
            context,
            &mut self.controller.borrow().component.borrow_mut(),
        );
    }
}

struct ComponentChild<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: Component + ?Sized,
{
    component: RefCell<Box<C>>,
    update: Box<dyn Fn(&Ctx, &mut C)>,
}
impl<Ctx, C> ComponentChild<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: 'static + Component + ?Sized,
{
    fn to_controller(self) -> ControllerChild<Ctx, C> {
        let controller = Controller::<C>::new_unsized(self.component);

        ControllerChild {
            controller,
            update: self.update,
        }
    }
}

enum Node {
    Text(String),
    Element(String),
}

struct Fragment<Ctx>
where
    Ctx: Any + ?Sized,
{
    nodes: Vec<Node>,
    renderables: Vec<Box<dyn Dynamic<Ctx = Ctx>>>,
    controllers: Vec<ControllerChild<Ctx, AnyComponent>>,
}
impl<Ctx> Dynamic for Fragment<Ctx>
where
    Ctx: Any + 'static + ?Sized,
{
    type Ctx = Ctx;

    fn mount(&mut self) {
        self.renderables.iter_mut().for_each(|renderable| {
            renderable.mount();
        });

        self.controllers.iter_mut().for_each(|child| {
            child.controller.borrow_mut().mount();
        });
    }

    fn detach(&mut self) {
        todo!()
    }

    fn update(&mut self, context: &Self::Ctx, changed: &[usize]) {
        self.controllers.iter_mut().for_each(|child| {
            child.update(context);
        });
    }
}

struct FragmentBuilder<Ctx>
where
    Ctx: Any + ?Sized,
{
    builder: Box<dyn Fn(&Ctx) -> Fragment<Ctx>>,
    children: Vec<ComponentChild<Ctx, AnyComponent>>,
}
impl<Ctx> FragmentBuilder<Ctx>
where
    Ctx: Any + ?Sized,
{
    pub fn build(self) -> Fragment<Ctx> {
        todo!();
    }
}

struct Controller<C>
where
    C: Component + ?Sized,
{
    component: Rc<RefCell<Box<C>>>,
    fragment: Fragment<C::Ctx>,
}

impl<C> Controller<C>
where
    C: Component + ?Sized + 'static,
{
    pub fn new<SizedC>(component: SizedC) -> Rc<RefCell<Controller<SizedC>>>
    where
        SizedC: Component,
    {
        Self::new_unsized(RefCell::new(Box::new(component)))
    }

    pub fn new_unsized<Comp>(component: RefCell<Box<Comp>>) -> Rc<RefCell<Controller<Comp>>>
    where
        Comp: Component + ?Sized,
    {
        let controller_reference = Rc::new(RefCell::new(None));

        let component = Rc::new(component);

        // Render component
        let fragment = component.borrow().render().build();

        let controller = Rc::new(RefCell::new(Controller::<Comp> {
            component,
            fragment,
        }));
        *controller_reference.borrow_mut() = Some(Rc::clone(&controller));

        controller
    }
}

impl<C> Dynamic for Controller<C>
where
    C: Component + ?Sized,
{
    // TODO: Maybe try make this the parent component somehow?
    type Ctx = ();

    fn mount(&mut self) {
        self.fragment.mount();
        let component = self.component.borrow();
        self.fragment.update(component.get_context(), &[]);
    }

    fn detach(&mut self) {
        todo!()
    }

    fn update(&mut self, context: &Self::Ctx, changed: &[usize]) {
        todo!()
    }
}

trait Component {
    type Ctx: Any + ?Sized + 'static;

    fn handle_event(&mut self) -> Option<Vec<usize>>;

    fn render(&self) -> FragmentBuilder<Self::Ctx>;

    fn get_context(&self) -> &Self::Ctx;
}

trait ComponentBB: Component {
    fn handle_event(&mut self) -> Option<Vec<usize>>;
    fn render(&self) -> FragmentBuilder<dyn Any>;
}

struct App(usize);
impl Component for App {
    type Ctx = Self;

    fn handle_event(&mut self) -> Option<Vec<usize>> {
        todo!()
    }

    fn render(&self) -> FragmentBuilder<Self::Ctx> {
        todo!()
    }

    fn get_context(&self) -> &Self::Ctx {
        self
    }
}

fn test() {
    let c = Box::new(App(0));

    // Can cast App to Any (required for a cast)
    // c as Box<dyn RealComponent<Ctx = App>>;

    // Can cast to component with a type for the context
    // c as Box<dyn Component<Ctx = App>>;

    // c as Box<dyn Component<Ctx = dyn Any>>;

    // TODO: Somehow cast this to Box<dyn Component<Ctx = dyn Any>>

    // let c = ComponentChild::<(), App> {
    //     component: RefCell::new(Box::new(c)),
    //     update: Box::new(|&ctx, c| {}),
    // };

    // c as ComponentChild<(), dyn Component<Ctx = App>>;

    // let c = ComponentWrapper { component: c };

    // let a = Box::new(c) as Box<ComponentWrapper<dyn Component<Ctx = <App as Component>::Ctx>>>;
}
