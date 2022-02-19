use file_monitor::model::set_up_model;

pub fn main() {
    let model = set_up_model(0.5);
    model.run();
}