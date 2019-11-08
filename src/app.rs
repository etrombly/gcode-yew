use gcode::{GCode, Mnemonic};
use stdweb::{
    console,
    traits::*,
    unstable::TryInto,
    web::{document, html_element::CanvasElement, CanvasRenderingContext2d},
};
use yew::prelude::*;

// TODO: a lot of gcode has 0,0 as the center and has negative x and y, canvas has 0,0 as top left. Need to handle it
// TODO: currently all movement is absolute, need to add relative movement
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
    draw_moves: bool,
}

pub struct App {
    state: State,
}

pub enum Msg {
    Clear,
    DrawMove,
    ProcessGcode,
    UpdateInput(String),
    UpdateZ(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let state = State {
            input: "".to_string(),
            location: Location {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            display_z: 0.,
            draw_moves: true,
        };
        App { state }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ProcessGcode => {
                self.state.draw_map();
            }
            Msg::DrawMove => self.state.draw_moves = !self.state.draw_moves,
            Msg::UpdateInput(val) => self.state.input = val,
            Msg::UpdateZ(val) => {
                self.state.display_z = val.parse::<f64>().unwrap();
                self.state.draw_map();
            }
            Msg::Clear => {
                self.state.input = "".to_string();
            }
        }
        true
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="columns is-centered">
                { self.view_input() }
                { self.view_canvas() }
            </div>
        }
    }
}

impl App {
    fn view_input(&self) -> Html<App> {
        html! {
            <div class="column is-one-quarter">
                <p class="message-header">
                    {"GCode"}
                </p>
                <textarea id="gcode_input" class="textarea" rows="20" placeholder="Enter GCode here" value=&self.state.input oninput=|e| Msg::UpdateInput(e.value)/>
                <div class="buttons is-grouped is-right">
                    <p class="control"><button class="button is-dark" onclick=|_| Msg::Clear>{"Clear"}</button></p>
                    <p class="control"><button class="button is-dark" onclick=|_| Msg::ProcessGcode>{"Process"}</button></p>
                </div>
                <p class="message is-dark">
                    <p class="message-header">
                        {"Options"}
                    </p>
                    <div class="message-body">
                        <label class="checkbox">
                            <input type="checkbox" id="draw_move" class="checkbox" checked=self.state.draw_moves onclick=|_| Msg::DrawMove/>
                            {"Draw Moves"}
                        </label>
                    </div>
                </p>
            </div>
        }
    }

    fn view_canvas(&self) -> Html<App> {
        html! {
            <div class="column is-two-thirds">
                <p class="message-header">
                    {"Output"}
                </p>
                <canvas id="gcode_canvas" class="box" style="width:100%;height:80%;"></canvas>
                <div class="field is-grouped is-grouped-centered">
                    <p class="control"><label class="label">{"Z layer "}</label></p>
                    <input type="range" id="display_z_slider" class="control is-expanded" value=&self.state.display_z oninput=|e| Msg::UpdateZ(e.value)/>
                    <input type="text" class="tag is-dark" size="4" value=&self.state.display_z oninput=|e| Msg::UpdateZ(e.value)/>
                </div>
            </div>
        }
    }
}

impl State {
    fn draw_map(&mut self) {
        let canvas: CanvasElement = document()
            .query_selector("#gcode_canvas")
            .unwrap()
            .expect("Didn't find the map canvas.")
            .try_into() // Element -> CanvasElement
            .unwrap(); // cannot be other than a canvas
 
        // size the canvas to match the actual width and height, gets rid of blurriness
        canvas.set_width(canvas.offset_width() as u32);
        canvas.set_height(canvas.offset_height() as u32);


        let context: CanvasRenderingContext2d = canvas.get_context().unwrap();

        context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
        self.location = Location {
            x: 0.,
            y: 0.,
            z: 0.,
        };
        // another fix for blurry lines
        let translate_x = (canvas.offset_height() as f64 / 2.) + 0.5;
        let translate_y = (canvas.offset_width() as f64 / 2.) + 0.5;
        // flip the y axis
        context.transform(1., 0., 0., -1., 0., canvas.height() as f64);
        context.translate(translate_x, translate_x);
        context.move_to(0.0, 0.0);

        let gcode = self.input.clone();
        let lines = gcode::parse(&gcode);
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
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        context.translate(-translate_x, -translate_y);
    }

    #[allow(non_snake_case)]
    fn parse_G0(&mut self, code: &GCode, context: &CanvasRenderingContext2d) {
        console!(log, format!("processing {} {:?}", code, self.location));
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
        context.set_line_width(1.0);
        context.begin_path();
        context.set_stroke_style_color(color);
        context.move_to(self.location.x, self.location.y);
        if let Some(x) = code.value_for('x') {
            self.location.x = x.into();
        }
        if let Some(y) = code.value_for('y') {
            self.location.y = y.into();
        }

        // TODO: figure out how wide the z drawing should be (above and below current z)
        // if the code isn't on the display layer don't draw
        if self.location.z - self.display_z > 0.1 || self.location.z - self.display_z < -0.1 {
            console!(
                log,
                "skipping code, not on display z layer {}",
                self.location.z - self.display_z
            );
            draw = false;
        }

        if draw {
            context.line_to(self.location.x, self.location.y);
        }
        context.stroke();
    }

    // TODO: currently only handling a G2, need to reverse x1, y1 for G3
    // TODO: need to calculate x1,y1 if you only have r
    #[allow(non_snake_case)]
    fn parse_G2(&mut self, code: &GCode, context: &CanvasRenderingContext2d) {
        console!(log, format!("processing {} {:?}", code, self.location));
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
        context.set_line_width(1.0);
        context.begin_path();
        context.set_stroke_style_color(color);
        context.move_to(self.location.x, self.location.y);
        let mut x1 = 0.0;
        let mut y1 = 0.0;

        if let Some(x) = code.value_for('x') {
            self.location.x = x.into();
            if let Some(i) = code.value_for('i') {
                x1 = self.location.x - i as f64;
            }
        }
        if let Some(y) = code.value_for('y') {
            self.location.y = y.into();
            if let Some(j) = code.value_for('j') {
                y1 = self.location.y - j as f64;
            }
        }
        let r = if let Some(r) = code.value_for('r') {
            r as f64
        } else {
            ((x1 - self.location.x).powf(2.) + (y1 - self.location.y).powf(2.)).sqrt()
        };
        if draw {
            context
                .arc_to(x1, y1, self.location.x, self.location.y, r)
                .expect("failed to draw arc");
        }
        context.stroke();
    }
}
