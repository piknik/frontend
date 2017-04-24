mod acquire;
mod color;
mod data;
mod generator;
mod graph;
mod trigger;

use gtk::{
    BoxExt,
    ContainerExt,
    RangeExt,
    WidgetExt,
};
use relm::ContainerWidget;

trait Panel {
    fn draw(&self, context: &::cairo::Context, scales: Scales);
}

#[derive(Copy, Clone)]
struct Scales {
    h: (f64, f64),
    v: (f64, f64),
}

impl Scales {
    pub fn get_width(&self) -> f64 {
        self.h.1 - self.h.0
    }

    pub fn get_height(&self) -> f64 {
        self.v.1 - self.v.0
    }
}

#[derive(Clone)]
pub struct Application {
    window: ::gtk::Window,
    graph: ::relm::Component<graph::Widget>,
    acquire: ::relm::Component<acquire::Widget>,
    generator: ::relm::Component<generator::Widget>,
    trigger: ::relm::Component<trigger::Widget>,
    redpitaya: ::redpitaya_scpi::Redpitaya,
    data: data::Widget,
    scales: Scales,
}

impl Application {
    pub fn run() {
        ::relm::run::<Self>()
            .unwrap();
    }

    pub fn draw(&self) {
        let graph = self.graph.widget();
        let width = graph.get_width();
        let height = graph.get_height();
        let context = self.graph.widget().create_context();

        self.transform(&context, width, height);

        context.set_line_width(0.01);

        graph.draw(&context, self.scales);
        self.trigger.widget().draw(&context, self.scales);

        if self.redpitaya.acquire.is_started() {
            context.set_line_width(0.05);
            self.data.draw(&context, self.scales);
        }
    }

    fn transform(&self, context: &::cairo::Context, width: f64, height: f64) {
        context.set_matrix(::cairo::Matrix {
            xx: width / self.scales.get_width(),
            xy: 0.0,
            yy: -height / self.scales.get_height(),
            yx: 0.0,
            x0: self.scales.h.0 * width / self.scales.get_width(),
            y0: self.scales.v.1 * height / self.scales.get_height(),
        });
    }
}

#[derive(Clone)]
pub enum Signal {
    AcquireStart,
    AcquireStop,
    GeneratorAmplitude(::redpitaya_scpi::generator::Source, f32),
    GeneratorOffset(::redpitaya_scpi::generator::Source, f32),
    GeneratorFrequency(::redpitaya_scpi::generator::Source, u32),
    GeneratorDutyCycle(::redpitaya_scpi::generator::Source, f32),
    GeneratorStart(::redpitaya_scpi::generator::Source),
    GeneratorStop(::redpitaya_scpi::generator::Source),
    GeneratorSignal(::redpitaya_scpi::generator::Source, ::redpitaya_scpi::generator::Form),
    GraphDraw,
    TriggerDelay(u16),
    TriggerLevel(f32),
    Quit,
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::AcquireStart => "Signal::AcquireStart",
            Signal::AcquireStop => "Signal::AcquireStop",
            Signal::GeneratorAmplitude(_, _) => "Signal::GeneratorAmplitude",
            Signal::GeneratorOffset(_, _) => "Signal::GeneratorOffset",
            Signal::GeneratorFrequency(_, _) => "Signal::GeneratorFrequency",
            Signal::GeneratorDutyCycle(_, _) => "Signal::GeneratorDutyCycle",
            Signal::GeneratorStart(_) => "Signal::GeneratorStart",
            Signal::GeneratorStop(_) => "Signal::GeneratorStop",
            Signal::GeneratorSignal(_, _) => "Signal::GeneratorSignal",
            Signal::TriggerDelay(_) => "Signal::TriggerDelay",
            Signal::TriggerLevel(_) => "Signal::TriggerLevel",
            Signal::GraphDraw => "Signal::GraphDraw",
            Signal::Quit => "Signal::Quit",
        }
    }
}

impl ::relm::Widget for Application {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Window;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::AcquireStart => self.redpitaya.acquire.start(),
            Signal::AcquireStop => self.redpitaya.acquire.stop(),

            Signal::GeneratorAmplitude(source, value) => self.redpitaya.generator.set_amplitude(source, value),
            Signal::GeneratorOffset(source, value) => self.redpitaya.generator.set_offset(source, value),
            Signal::GeneratorFrequency(source, value) => self.redpitaya.generator.set_frequency(source, value),
            Signal::GeneratorDutyCycle(source, value) => self.redpitaya.generator.set_duty_cycle(source, value),
            Signal::GeneratorStart(source) => self.redpitaya.generator.start(source),
            Signal::GeneratorStop(source) => self.redpitaya.generator.stop(source),
            Signal::GeneratorSignal(source, form) => self.redpitaya.generator.set_form(source, form),

            Signal::GraphDraw => {
                self.data.data = self.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN1);
                self.draw();
            },

            Signal::TriggerDelay(value) => self.redpitaya.trigger.set_delay(value),
            Signal::TriggerLevel(value) => self.redpitaya.trigger.set_level(value),

            Signal::Quit => {
                self.redpitaya.acquire.stop();
                self.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT1);
                self.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT2);
                ::gtk::main_quit();
            },
        };
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        // @TODO use program arguments
        let redpitaya = ::redpitaya_scpi::Redpitaya::new("192.168.1.5:5000");

        let main_box = ::gtk::Box::new(::gtk::Orientation::Horizontal, 0);

        let graph_page = ::gtk::EventBox::new();
        main_box.pack_start(&graph_page, true, true, 0);

        let graph = graph_page.add_widget::<graph::Widget, _>(&relm);
        connect!(graph@graph::Signal::Draw, relm, Signal::GraphDraw);

        let notebook = ::gtk::Notebook::new();

        let acquire_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        acquire_page.set_border_width(10);
        let acquire = acquire_page.add_widget::<acquire::Widget, _>(&relm);
        connect!(acquire@acquire::Signal::Start, relm, Signal::AcquireStart);
        connect!(acquire@acquire::Signal::Stop, relm, Signal::AcquireStop);

        notebook.append_page(
            &acquire_page,
            Some(&::gtk::Label::new(Some("Acquire")))
        );

        let generator_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        generator_page.set_border_width(10);
        let generator = generator_page.add_widget::<generator::Widget, _>(&relm);
        connect!(generator@generator::Signal::Start(source), relm, Signal::GeneratorStart(source));
        connect!(generator@generator::Signal::Stop(source), relm, Signal::GeneratorStop(source));
        connect!(generator@generator::Signal::Amplitude(source, value), relm, Signal::GeneratorAmplitude(source, value));
        connect!(generator@generator::Signal::Offset(source, value), relm, Signal::GeneratorOffset(source, value));
        connect!(generator@generator::Signal::Frequency(source, value), relm, Signal::GeneratorFrequency(source, value));
        connect!(generator@generator::Signal::DutyCycle(source, value), relm, Signal::GeneratorDutyCycle(source, value));
        connect!(generator@generator::Signal::Signal(source, value), relm, Signal::GeneratorSignal(source, value));

        notebook.append_page(
            &generator_page,
            Some(&::gtk::Label::new(Some("Generator")))
        );

        let window = ::gtk::Window::new(::gtk::WindowType::Toplevel);
        window.add(&main_box);
        connect!(relm, window, connect_destroy(_), Signal::Quit);

        let trigger_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        trigger_page.set_border_width(10);
        let trigger = trigger_page.add_widget::<trigger::Widget, _>(&relm);
        connect!(trigger@trigger::Signal::Delay(value), relm, Signal::TriggerDelay(value));
        connect!(trigger@trigger::Signal::Level(value), relm, Signal::TriggerLevel(value));

        notebook.append_page(
            &trigger_page,
            Some(&::gtk::Label::new(Some("Trigger")))
        );

        main_box.pack_start(&notebook, false, true, 0);

        Application {
            window: window,
            graph: graph,
            acquire: acquire,
            generator: generator,
            trigger: trigger,
            data: data::Widget::new(),
            redpitaya: redpitaya,
            scales: Scales {
                h: (0.0, 16384.0),
                v: (-5.0, 5.0),
            },
        }
    }

    fn init_view(&self) {
        self.generator.widget().amplitude_scale.set_value(
            self.redpitaya.generator.get_amplitude(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.generator.widget().offset_scale.set_value(
            self.redpitaya.generator.get_offset(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.generator.widget().frequency_scale.set_value(
            self.redpitaya.generator.get_frequency(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.generator.widget().duty_cycle_scale.set_value(
            self.redpitaya.generator.get_duty_cycle(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.trigger.widget().delay_scale.set_value(
            self.redpitaya.trigger.get_delay() as f64
        );

        self.trigger.widget().level_scale.set_value(
            self.redpitaya.trigger.get_level() as f64
        );

        self.window.show_all();
        self.generator.widget().duty_cycle_frame.set_visible(false);
    }
}
