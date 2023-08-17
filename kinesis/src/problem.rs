use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

trait Dynamic {
    type Ctx: Any + ?Sized;

    fn mount(&mut self);
    fn detach(&mut self);

    fn update(&mut self, context: &Self::Ctx, changed: &[usize]);
}

type AnyComponent = dyn Component<Ctx = dyn Any, Child = dyn Any>;

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

struct Fragment<Ctx>
where
    Ctx: Any + ?Sized,
{
    renderables: Vec<Box<dyn Dynamic<Ctx = Ctx>>>,
    controllers: Vec<ControllerChild<Ctx, AnyComponent>>,
}
impl<Ctx> Dynamic for Fragment<Ctx>
where
    Ctx: 'static + ?Sized,
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

trait Component {
    type Ctx: Any + ?Sized;
    type Child: Any + ?Sized;

    fn handle_event(&mut self) -> Option<Vec<usize>>;

    fn render(&self) -> FragmentBuilder<Self::Ctx>;

    fn get_context(&self) -> &Self::Ctx;

    fn update_child(&self, child: &mut Self::Child);
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

enum ChildEnum {
    Count(Count),
}

struct Count(usize);
impl From<Count> for ChildEnum {
    fn from(count: Count) -> Self {
        ChildEnum::Count(count)
    }
}

struct App(usize);
impl Component for App {
    type Ctx = Self;
    type Child = ChildEnum;

    fn handle_event(&mut self) -> Option<Vec<usize>> {
        todo!()
    }

    fn render(&self) -> FragmentBuilder<Self::Ctx> {
        todo!()
    }

    fn get_context(&self) -> &Self::Ctx {
        self
    }

    fn update_child(&self, child: &mut Self::Child) {
        // Perform prop bindings here
        match child {
            Self::Child::Count(count) => count.0 = self.0 * 2,
        }
    }
}
