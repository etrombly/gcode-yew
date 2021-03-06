use gcode::{GCode, Mnemonic, Parser, Nop, buffers::DefaultBuffers};
use js_sys::Array;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, WheelEvent};
use yew::prelude::*;

#[derive(Debug)]
struct Location {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug)]
struct State {
    input: String,
    location: Location,
    display_z: f64,
    absolute: bool,
    draw_moves: bool,
    zoom: f64,
    translate: (f64, f64),
    dragging: Option<(f64, f64, f64, f64)>,
}

pub struct App {
    state: State,
    link: ComponentLink<Self>,
}

pub enum Msg {
    Clear,
    DrawMove,
    ProcessGcode,
    UpdateInput(String),
    UpdateZ(String),
    Scroll(f64),
    DragStart(f64, f64),
    Dragging(f64, f64),
    DragStop,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = State {
            input: "".to_string(),
            location: Location {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            display_z: 0.,
            absolute: true,
            draw_moves: true,
            zoom: 1.0,
            translate: (0., 0.),
            dragging: None,
        };
        App { state, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ProcessGcode => {
                self.state.zoom = 1.0;
                self.state.translate = (0., 0.);
                self.state.draw_map();
            }
            Msg::DrawMove => self.state.draw_moves = !self.state.draw_moves,
            Msg::UpdateInput(val) => self.state.input = val,
            Msg::UpdateZ(val) => {
                self.state.display_z = val.parse::<f64>().unwrap();
                self.state.draw_map();
            }
            Msg::Clear => {
                self.state.zoom = 1.0;
                self.state.input = "".to_string();
                self.state.translate = (0., 0.);
            }
            Msg::Scroll(x) => self.state.zoom(x),
            Msg::DragStart(x, y) => {
                self.state.dragging = Some((x, y, self.state.translate.0, self.state.translate.1))
            }
            Msg::Dragging(x, y) => {
                if let Some((start_x, start_y, translate_x, translate_y)) = self.state.dragging {
                    self.state.translate = (translate_x + x - start_x, translate_y + y - start_y);
                    self.state.draw_map();
                }
            }
            Msg::DragStop => self.state.dragging = None,
        }
        true
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        todo!()
    }

    fn view(&self) -> Html {
        html! {
            <div class="columns is-centered">
                { self.view_input(&self.link) }
                { self.view_canvas(&self.link) }
            </div>
        }
    }
}

impl App {
    fn view_input(&self, link: &ComponentLink<App>) -> Html {
        let clear_handler = link.callback(|_| Msg::Clear);
        let gcode_handler = link.callback(|_| Msg::ProcessGcode);
        let draw_handler = link.callback(|_| Msg::DrawMove);
        let input_handler = link.callback(|e: InputData| Msg::UpdateInput(e.value));
        html! {
            <div class="column is-one-quarter">
                <p class="message-header">
                    {"GCode"}
                </p>
                <textarea id="gcode_input" class="textarea" rows="20" placeholder="Enter GCode here" value=&self.state.input oninput=input_handler/>
                <div class="buttons is-grouped is-right">
                    <p class="control"><button class="button is-dark" onclick=clear_handler>{"Clear"}</button></p>
                    <p class="control"><button class="button is-dark" onclick=gcode_handler>{"Process"}</button></p>
                </div>
                <p class="message is-dark">
                    <p class="message-header">
                        {"Options"}
                    </p>
                    <div class="message-body">
                        <label class="checkbox">
                            <input type="checkbox" id="draw_move" class="checkbox" checked=self.state.draw_moves onclick=draw_handler/>
                            {"Draw Moves"}
                        </label>
                    </div>
                </p>
            </div>
        }
    }

    fn view_canvas(&self, link: &ComponentLink<App>) -> Html {
        let wheel_handler = link.callback(|e: WheelEvent| Msg::Scroll(e.delta_y()));
        let down_handler =
            link.callback(|e: MouseEvent| Msg::DragStart(e.offset_x().into(), e.offset_y().into()));
        let move_handler =
            link.callback(|e: MouseEvent| Msg::Dragging(e.offset_x().into(), e.offset_y().into()));
        let up_handler = link.callback(|_| Msg::DragStop);
        let input_handler = link.callback(|e: InputData| Msg::UpdateZ(e.value));
        let input_handler2 = link.callback(|e: InputData| Msg::UpdateZ(e.value));
        html! {
            <div class="column is-two-thirds">
                <p class="message-header">
                    {"Output"}
                </p>
                <canvas id="gcode_canvas" class="box" style="width:100%;height:80%;"
                    onmousewheel=wheel_handler
                    onmousedown=down_handler
                    onmousemove=move_handler
                    onmouseup=up_handler></canvas>
                <div class="field is-grouped is-grouped-centered">
                    <p class="control"><label class="label">{"Z layer "}</label></p>
                    <input type="range" id="display_z_slider" class="control is-expanded" value=&self.state.display_z oninput=input_handler/>
                    <input type="text" class="tag is-dark" size="4" value=&self.state.display_z oninput=input_handler2/>
                </div>
            </div>
        }
    }
}

impl State {
    fn draw_map(&mut self) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas: HtmlCanvasElement = document
            .query_selector("#gcode_canvas")
            .unwrap()
            .expect("Didn't find the map canvas.")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        // size the canvas to match the actual width and height, gets rid of blurriness
        canvas.set_width(canvas.offset_width() as u32);
        canvas.set_height(canvas.offset_height() as u32);

        self.absolute = true;

        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
        self.location = Location {
            x: 0.,
            y: 0.,
            z: 0.,
        };
        // another fix for blurry lines
        let translate_x = (canvas.width() as f64 / 2.) + 0.5;
        let translate_y = (canvas.height() as f64 / 2.) + 0.5;
        // flip the y axis
        context
            .transform(1., 0., 0., -1., 0., canvas.height() as f64)
            .unwrap();

        // draw x and y axis lines
        context
            .translate(self.translate.0, -self.translate.1)
            .unwrap();
        context.begin_path();
        context.set_stroke_style(&"grey".into());
        let array = Array::new();
        array.push(&"3".into());
        array.push(&"2".into());
        context.set_line_dash(&array).unwrap();
        context.move_to(0., translate_y);
        context.line_to(canvas.width() as f64, translate_y);
        context.move_to(translate_x, 0.);
        context.line_to(translate_x, canvas.height() as f64);
        context.stroke();
        let array2 = Array::new();
        context.set_line_dash(&array2).unwrap();

        context.translate(translate_x, translate_y).unwrap();

        context.scale(self.zoom, self.zoom).unwrap();

        let gcode = self.input.clone();
        let lines: Parser<Nop, DefaultBuffers> = Parser::new(&gcode, Nop);
        for line in lines {
            for code in line.gcodes() {
                match code.mnemonic() {
                    Mnemonic::General => match code.major_number() {
                        0 | 1 => {
                            self.parse_G0(code, &context);
                        }
                        2 | 3 => {
                            self.parse_G2(code, &context);
                        }
                        90 => self.absolute = true,
                        91 => self.absolute = false,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        context.translate(-translate_x, -translate_y).unwrap();
    }

    #[allow(non_snake_case)]
    fn parse_G0(&mut self, code: &GCode, context: &CanvasRenderingContext2d) {
        //fn parse_G0(&mut self, code: &GCode<ArrayVec<[Word; 5]>>, context: &CanvasRenderingContext2d) {
        if let Some(z) = code.value_for('z') {
            self.location.z = z as f64;
        }

        let mut draw = self.draw_moves;
        let mut color = "black";

        match code.value_for('e') {
            Some(e) => {
                draw = true;
                if e < 0. {
                    // color retracts red
                    color = "red";
                }
            }
            // if moves are drawn color them green
            None => color = "green",
        }
        context.set_line_width(1.0 / self.zoom);
        context.begin_path();
        context.set_stroke_style(&color.into());
        context.move_to(self.location.x, self.location.y);
        if let Some(x) = code.value_for('x') {
            if self.absolute {
                self.location.x = x.into();
            } else {
                self.location.x += x as f64;
            }
        }
        if let Some(y) = code.value_for('y') {
            if self.absolute {
                self.location.y = y.into();
            } else {
                self.location.y += y as f64;
            }
        }

        // TODO: figure out how wide the z drawing should be (above and below current z)
        // if the code isn't on the display layer don't draw
        if self.location.z - self.display_z > 0.1 || self.location.z - self.display_z < -0.1 {
            draw = false;
        }

        if draw {
            context.line_to(self.location.x, self.location.y);
        }
        context.stroke();
    }

    #[allow(non_snake_case)]
    fn parse_G2(
        &mut self,
        code: &GCode,
        //code: &GCode<ArrayVec<[Word; 5]>>,
        context: &CanvasRenderingContext2d,
    ) -> Option<bool> {
        if let Some(z) = code.value_for('z') {
            self.location.z = z as f64;
        }

        let mut draw = self.draw_moves;
        let mut color = "black";

        match code.value_for('e') {
            Some(e) => {
                draw = true;
                if e < 0. {
                    // color retracts red
                    color = "red";
                }
            }
            // if moves are drawn color them green
            None => color = "green",
        }
        context.set_line_width(1.0 / self.zoom);
        context.begin_path();
        context.set_stroke_style(&color.into());
        context.move_to(self.location.x, self.location.y);

        let x = if self.absolute {
            code.value_for('x')? as f64
        } else {
            code.value_for('x')? as f64 + self.location.x
        };
        let y = if self.absolute {
            code.value_for('y')? as f64
        } else {
            code.value_for('y')? as f64 + self.location.x
        };

        // TODO: handle i or j not being entered
        let (center_x, center_y, radius) = if let (Some(i), Some(j)) =
            (code.value_for('i'), code.value_for('j'))
        {
            let x1 = self.location.x + i as f64;
            let y1 = self.location.y + j as f64;
            (
                x1,
                y1,
                ((x1 - self.location.x).powi(2) + (y1 - self.location.y).powi(2)).sqrt(),
            )
        } else if let Some(r) = code.value_for('r') {
            let radius = r as f64;

            let q = ((x - self.location.x).powi(2) + (y - self.location.y).powi(2)).sqrt();

            let y3 = (self.location.y + y) / 2.;
            let x3 = (self.location.x + x) / 2.;

            let basex = (radius.powi(2) - (q / 2.).powi(2)).sqrt() * ((self.location.y - y) / q);
            let basey = (radius.powi(2) - (q / 2.).powi(2)).sqrt() * ((x - self.location.x) / q);

            // TODO: center may be at -basex -basey, need to figure out how to pick
            let centerx1 = x3 + basex;
            let centery1 = y3 + basey;

            (centerx1, centery1, radius)
        } else {
            return None;
        };

        let angle1 = (self.location.y - center_y).atan2(self.location.x - center_x);
        let angle2 = (y - center_y).atan2(x - center_x);

        self.location.x = x;
        self.location.y = y;

        // TODO: figure out how wide the z drawing should be (above and below current z)
        // if the code isn't on the display layer don't draw
        if self.location.z - self.display_z > 0.1 || self.location.z - self.display_z < -0.1 {
            draw = false;
        }

        if draw {
            if code.major_number() == 2 {
                context
                    .arc_with_anticlockwise(center_x, center_y, radius, angle1, angle2, true)
                    .unwrap();
            } else {
                context
                    .arc_with_anticlockwise(center_x, center_y, radius, angle1, angle2, false)
                    .unwrap();
            }
        }
        context.stroke();
        Some(true)
    }

    fn zoom(&mut self, delta: f64) {
        let clicks = delta as f64 / -40.;
        let scale_factor = 1.1_f64;
        let factor = scale_factor.powf(clicks);
        self.zoom *= factor;
        self.draw_map();
    }
}
