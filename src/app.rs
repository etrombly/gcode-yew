use yew::prelude::*;
use stdweb::{
    traits::*,
    unstable::TryInto,
    console,
    web::{
        document,
        html_element::CanvasElement,
        CanvasRenderingContext2d,
    },
};
use gcode::Mnemonic;

#[derive(Debug)]
struct State {
  input: String,
}

pub struct App {
    state: State,
}

pub enum Msg {
    ProcessGcode,
    UpdateInput(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let state = State{input: "".to_string()};
        App {state}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ProcessGcode => {
                {
                    self.state.draw_map(&self.state.input);
                }
                self.state.input = "".to_string();
            }
            Msg::UpdateInput(val) => {
                self.state.input = val;
            }
        }
        true
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="row">
                { self.view_input() }
                { self.view_canvas() }
            </div>
        }
    }
}

impl App {
    fn view_input(&self) -> Html<App> {
        html! {
            <div class="column left">
                <textarea id="gcode_input" rows="10" cols="30" value=&self.state.input oninput=|e| Msg::UpdateInput(e.value)/>
                <br/>
                <button onclick=|_| Msg::ProcessGcode>{"Process Gcode"}</button>
            </div>
        }
    }

    fn view_canvas(&self) -> Html<App> {
        html! {
            <div class="column middle">
                <canvas id="gcode_canvas" width="640" height="480"></canvas>
            </div>
        }
    }
}

impl State {
    fn draw_map(&self, gcode: &str) {
        let canvas: CanvasElement = document()
            .query_selector("#gcode_canvas")
            .unwrap()
            .expect("Didn't find the map canvas.")
            .try_into() // Element -> CanvasElement
            .unwrap(); // cannot be other than a canvas
        let context: CanvasRenderingContext2d = canvas.get_context().unwrap();

        context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
        context.set_line_width(1.0);
        context.begin_path();
        context.set_fill_style_color("black");
        // fix for blurry lines
        context.translate(0.5, 0.5);
        context.move_to(0.0, 0.0);

        let lines = gcode::parse(gcode);
        for line in lines {
            for code in line.gcodes() {
                match code.mnemonic() {
                    Mnemonic::General => {
                        match code.major_number() {
                            0 | 1 => {
                                if let Some(x) = code.value_for('x') {
                                    if let Some(y) = code.value_for('y') {
                                        //console!(log, format!("line to {} {}", x, y));
                                        context.line_to(x.into(),y.into());
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }
        }
        context.stroke();
    }
}