use application::Panel;
use color::Colorable;
use gtk::{
    self,
    BoxExt,
    OrientableExt,
    WidgetExt,
};
use super::level::Widget as LevelWidget;
use super::level::placeholder::Widget as Placeholder;
use super::level::Signal::Level as LevelSignal;
use super::level::widget::Orientation;
use super::Signal;
use relm_attributes::widget;

impl Widget {
    pub fn level_left<'a>(&'a self) -> &'a ::relm::Component<LevelWidget> {
        &self.level_left
    }

    pub fn level_right<'a>(&'a self) -> &'a ::relm::Component<LevelWidget> {
        &self.level_right
    }

    pub fn level_top<'a>(&'a self) -> &'a ::relm::Component<LevelWidget> {
        &self.level_top
    }

    pub fn get_width(&self) -> f64 {
        self.drawing_area.get_allocated_width() as f64
    }

    pub fn get_height(&self) -> f64 {
        self.drawing_area.get_allocated_height() as f64
    }

    pub fn set_image(&self, image: &::cairo::ImageSurface) {
        let context = self.create_context(&self.drawing_area);

        context.set_source_surface(image, 0.0, 0.0);
        context.paint();
    }

    pub fn invalidate(&self) {
        self.drawing_area.queue_draw_area(
            0,
            0,
            self.get_width() as i32,
            self.get_height() as i32,
        );

        self.level_left.widget().invalidate();
        self.level_right.widget().invalidate();
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        let width = model.scales.get_width();
        let height = model.scales.get_height();

        context.set_color(::color::BACKGROUND);
        context.rectangle(model.scales.h.0, model.scales.v.0, width, height);
        context.fill();

        context.set_color(::color::MAIN_SCALE);

        context.rectangle(model.scales.h.0, model.scales.v.0, width, height);
        context.stroke();

        for i in 0..11 {
            if i % 5 == 0 {
                context.set_color(::color::MAIN_SCALE);
            } else {
                context.set_color(::color::SECONDARY_SCALE);
            }

            let x = width / 10.0 * (i as f64);

            context.set_line_width(width / 1000.0);
            context.move_to(model.scales.h.0 + x, model.scales.v.0);
            context.line_to(model.scales.h.0 + x, model.scales.v.1);
            context.stroke();

            let y = height / 10.0 * (i as f64);

            context.set_line_width(height / 1000.0);
            context.move_to(model.scales.h.0, model.scales.v.0 + y);
            context.line_to(model.scales.h.1, model.scales.v.0 + y);
            context.stroke();
        }

        self.level_left.widget().draw(context, model);
        self.level_right.widget().draw(context, model);
    }
}

#[widget]
impl ::relm::Widget for Widget {
    fn model(_: ()) -> () {
    }

    fn update(&mut self, _: Signal, _: &mut Self::Model) {
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                packing: {
                    expand: false,
                    fill: true,
                },
                Placeholder {
                    packing: {
                        expand: false,
                        fill: true,
                    },
                },
                #[name="level_left"]
                LevelWidget(Orientation::Left) {
                    packing: {
                        expand: true,
                        fill: true,
                    },
                    LevelSignal(name, offset) => Signal::Level(name, offset),
                },
            },
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                packing: {
                    expand: true,
                    fill: true,
                },
                #[name="level_top"]
                LevelWidget(Orientation::Top) {
                    packing: {
                        expand: false,
                        fill: true,
                    },
                    LevelSignal(name, offset) => Signal::Level(name, offset),
                },
                #[name="drawing_area"]
                gtk::DrawingArea {
                    packing: {
                        expand: true,
                        fill: true,
                    },
                    draw(_, _) => (Signal::Draw, ::gtk::Inhibit(false)),
                },
            },
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                packing: {
                    expand: false,
                    fill: true,
                },
                Placeholder {
                    packing: {
                        expand: false,
                        fill: true,
                    },
                },
                #[name="level_right"]
                LevelWidget(Orientation::Right) {
                    packing: {
                        expand: true,
                        fill: true,
                    },
                    LevelSignal(name, offset) => Signal::Level(name, offset),
                },
            },
        },
    }
}
