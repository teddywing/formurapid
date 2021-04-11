use pdf_forms::{Form, FieldType};

fn main() {
    let mut form = Form::load("./f1040.pdf").unwrap();

    for i in 0..form.len() {
        let field_type = form.get_type(i);

        if let FieldType::Text = field_type {
            form.set_text(i, format!("{}", i)).unwrap();
        }
    }

    form.save("./f1040-new.pdf").unwrap();
}
